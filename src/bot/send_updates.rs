use std::time::Duration;

use lazy_static::lazy_static;
use rand::{
    distr::Uniform,
    prelude::{Distribution, StdRng},
};
use serenity::{
    builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage},
    model::{
        Colour,
        channel::GuildChannel,
        id::{ChannelId, GuildId},
    },
    prelude::{Context},
};

use crate::{
    db::{get_discord_channels, get_unsent_message, set_message_sent},
    rss_poller::poller::PollTarget,
};

fn color_tuple_from_hex(hex: &'static str) -> (u8, u8, u8) {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
    (r, g, b)
}

static LOOP_INTERVAL_STD: Duration = Duration::new(5 * 60, 0);
static COLORS: [&str; 12] = [
    "#fafafa",
    "#00a2e8",
    "#7092be",
    "#99D9ea",
    "#b5e61d",
    "#ed1c24",
    "#ff7f27",
    "#ffc90e",
    "#ffaec9",
    "#be82be",
    "#523462",
    "#211d32",
];

lazy_static! {
    static ref COLOR_TUPLES: [(u8, u8, u8); 10] = [
        color_tuple_from_hex(COLORS[0]),
        color_tuple_from_hex(COLORS[1]),
        color_tuple_from_hex(COLORS[2]),
        color_tuple_from_hex(COLORS[3]),
        color_tuple_from_hex(COLORS[4]),
        color_tuple_from_hex(COLORS[5]),
        color_tuple_from_hex(COLORS[6]),
        color_tuple_from_hex(COLORS[7]),
        color_tuple_from_hex(COLORS[8]),
        color_tuple_from_hex(COLORS[9]),
    ];
}

fn get_channel_from_cache(
    context: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Option<GuildChannel> {
    let guild = context.cache.guild(guild_id)?;
    let channels = &guild.channels;
    channels.get(&channel_id).cloned()
}

async fn get_channel_from_http(
    context: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Option<GuildChannel> {
    let guild = match context.http.get_guild(guild_id).await {
        Ok(guild) => guild,
        Err(e) => {
            tracing::error!("Could not find guild: {e:?}");
            return None;
        }
    };

    let mut channels = match guild.channels(&context.http).await {
        Ok(channels) => channels,
        Err(e) => {
            tracing::error!("Encountered API error: {e:?}");
            return None;
        }
    };

    channels.remove(&channel_id)
}

/// Poll databases for unsent messages and send them
pub async fn send_updates(context: &Context, poll_targets: &[PollTarget]) -> anyhow::Result<()> {
    let max_i = poll_targets.len();
    let mut i = 0;
    let mut rng = rand::make_rng::<StdRng>();
    let uniform = Uniform::new(0, COLOR_TUPLES.len() - 1)?;
    loop {
        i += 1;
        if i >= max_i {
            i -= max_i;
        }
        let mut conn = poll_targets[i].db.acquire().await?;
        let msg = match get_unsent_message(&mut conn).await? {
            Some(msg) => msg,
            None => {
                i += 1;
                tokio::time::sleep(LOOP_INTERVAL_STD).await;
                continue;
            }
        };
        tracing::info!(
            "Sending notifications for {} - {}",
            msg.title,
            poll_targets[i].target.name
        );
        let tuple = COLOR_TUPLES[uniform.sample(&mut rng)];
        let embed = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(poll_targets[i].target.author)
                    .url(poll_targets[i].target.url),
            )
            .title(msg.title)
            .url(msg.link)
            .description(msg.description)
            .color(Colour::from_rgb(tuple.0, tuple.1, tuple.2));

        let targets = get_discord_channels(&mut conn).await?;

        for target in targets {
            let (channel_id, guild_id) = (
                ChannelId::new(target.channel_id),
                GuildId::new(target.guild_id),
            );
            let channel = match get_channel_from_cache(context, guild_id, channel_id) {
                Some(channel) => channel,
                None => match get_channel_from_http(context, guild_id, channel_id).await {
                    Some(c) => c,
                    None => {
                        tracing::error!("Could not get channel {channel_id:?} from cache");
                        continue;
                    }
                },
            };
            if let Err(e) = channel
                .send_message(&context, CreateMessage::new().add_embed(embed.clone()))
                .await
            {
                tracing::error!("Problem while sending message: {e:?}");
                continue;
            }
        }
        set_message_sent(&msg.guid, &mut conn).await?;
        tracing::info!("Sent messages");

        tokio::time::sleep(LOOP_INTERVAL_STD).await;
    }
}
