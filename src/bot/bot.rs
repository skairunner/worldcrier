use poise::{Context, Framework, FrameworkOptions};
use serenity::{all::ClientBuilder, client::FullEvent};
use std::{
    env,
    sync::atomic::{AtomicBool, Ordering},
};
use tracing::instrument;

use crate::{
    bot::{error::BotError, send_updates::send_updates},
    rss_poller::poller::PollTarget,
};

#[derive(Debug)]
pub struct UserData {
    pub superadmin_id: Option<u64>,
    pub is_loop_running: AtomicBool,
    pub poll_targets: Vec<PollTarget>,
}

#[instrument]
pub async fn start_bot(poll_targets: Vec<PollTarget>) -> anyhow::Result<()> {
    let framework: Framework<UserData, BotError> = poise::Framework::builder()
        .options(FrameworkOptions {
            commands: vec![],
            command_check: Some(|ctx: Context<UserData, BotError>| {
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
