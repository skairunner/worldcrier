use poise::{Context as PoiseContext, CreateReply, Framework, FrameworkOptions};
use serenity::{all::ClientBuilder, client::FullEvent};
use std::{
    env,
    sync::atomic::{AtomicBool, Ordering},
};
use tracing::instrument;

use crate::{
    bot::{error::BotError, send_updates::send_updates},
    db::add_discord_channel,
    rss_poller::poller::PollTarget,
};

#[derive(Debug)]
pub struct UserData {
    pub superadmin_id: Option<u64>,
    pub is_loop_running: AtomicBool,
    pub poll_targets: Vec<PollTarget>,
}

/// Check the parameter and return an error if it is inappropriate.
fn _check_parameter(param: Option<&str>, name: &str) -> Result<u64, String> {
    let param = if let Some(param) = param {
        param
    } else {
        return Err(format!("No {name} found"));
    };
    param
        .parse::<u64>()
        .map_err(|_e| format!("The {name} was not a valid ID"))
}

#[poise::command(slash_command)]
async fn register(
    ctx: PoiseContext<'_, UserData, BotError>,
    #[description = "The author"] author_name: String,
    #[description = "The world name"] world_name: String,
    #[description = "Channel to register"] channel_link: String,
) -> Result<(), BotError> {
    // First, check if the target exists.
    let target = ctx
        .data()
        .poll_targets
        .iter()
        .filter(|target| target.matches(&author_name, &world_name))
        .next()
        .cloned();
    let target = if let Some(target) = target {
        target
    } else {
        ctx.send(CreateReply {
            content: Some(format!("Error: the specified world {world_name} by {author_name} is not known to Worldcrier.")),
            ..Default::default()
        }).await?;
        return Ok(());
    };
    // Example link:
    // https://discord.com/channels/768578800681222214/768580352737542165
    let channel_link = channel_link.replace("https://discord.com/channels/", "");
    let mut intermediate = channel_link.split(",").take(2);
    let (guild_id, channel_id) = (intermediate.next(), intermediate.next());
    let guild_id = match _check_parameter(guild_id, "guild id") {
        Ok(guild_id) => guild_id,
        Err(msg) => {
            ctx.send(CreateReply {
                content: Some(msg),
                ..Default::default()
            })
            .await?;
            return Ok(());
        }
    };
    let channel_id = match _check_parameter(channel_id, "channel id") {
        Ok(channel_id) => channel_id,
        Err(msg) => {
            ctx.send(CreateReply {
                content: Some(msg),
                ..Default::default()
            })
            .await?;
            return Ok(());
        }
    };

    // Check that the command was invoked in the guild the channel is being added to
    let channel = if let Some(channel) = ctx.guild_channel().await {
        channel
    } else {
        ctx.send(CreateReply {
            content: Some(
                "This command can only be invoked from a server, not in DMs.".to_string(),
            ),
            ..Default::default()
        })
        .await?;
        return Ok(());
    };
    if channel.guild_id.get() != guild_id {
        ctx.send(CreateReply {
            content: Some("The target channel does not exist in the guild".to_string()),
            ..Default::default()
        })
        .await?;
        return Ok(());
    }
    // Next, need to check that the provided channel exists in the server

    Ok(())
}

#[poise::command(slash_command)]
async fn unregister(
    ctx: PoiseContext<'_, UserData, BotError>,
    #[description = "The author"] author_name: String,
    #[description = "The world name"] world_name: String,
    #[description = "Channel to unregister"] channel_link: String,
) -> Result<(), BotError> {
    Ok(())
}

#[instrument]
pub async fn start_bot(poll_targets: Vec<PollTarget>) -> anyhow::Result<()> {
    let framework: Framework<UserData, BotError> = poise::Framework::builder()
        .options(FrameworkOptions {
            commands: vec![],
            command_check: Some(|ctx: PoiseContext<UserData, BotError>| {
                Box::pin(async move {
                    // Always allow superadmin
                    if let Some(superadmin_id) = &ctx.data().superadmin_id
                        && ctx.author().id.get() == *superadmin_id
                    {
                        return Ok(true);
                    }
                    // Also allow administrator of the current guild, if it exists.
                    if let Some(member) = ctx.author_member().await
                        && let Some(permissions) = member.permissions
                        && permissions.administrator()
                    {
                        return Ok(true);
                    }
                    Ok(false)
                })
            }),
            event_handler: |ctx, event, _framework, user_data| {
                Box::pin(async move {
                    // As in the Serenity examples
                    if let FullEvent::CacheReady { guilds: _ } = event
                        && !user_data.is_loop_running.load(Ordering::Relaxed)
                    {
                        // Immediately try to set the is running value to prevent race condition.
                        // If we successfully set the value, we should order other operations after this.
                        // Otherwise we can assume someone else set the value and we should stop handling this event.
                        if user_data
                            .is_loop_running
                            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                            .is_err()
                        {
                            return Ok(());
                        }
                        // Now let's do the polling.
                        send_updates(ctx, &user_data.poll_targets)
                            .await
                            .map_err(BotError::OtherError)?;
                    }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|_ctx, _ready, _framework| {
            Box::pin(async move {
                let superadmin_id = Some(env::var("SUPERADMIN")?.parse()?);
                Ok(UserData {
                    superadmin_id,
                    is_loop_running: AtomicBool::new(false),
                    poll_targets,
                })
            })
        })
        .build();
    let client = ClientBuilder::new(
        env::var("APP_TOKEN")?,
        serenity::all::GatewayIntents::non_privileged(),
    )
    .framework(framework)
    .await;
    client?.start().await?;
    Ok(())
}
