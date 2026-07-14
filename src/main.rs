use dotenvy::dotenv;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{
    bot::start_bot,
    data::FEEDS,
    db::{get_migrator, open_db},
    rss_poller::poller::{PollTarget, do_poll_rss},
};

mod bot;
mod data;
mod db;
mod dbtypes;
mod rss_poller;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::new("worldcrier=debug").add_directive(LevelFilter::WARN.into());
    tracing_subscriber::fmt().with_env_filter(filter).init();
    dotenv()?;

    let migrator = get_migrator().await?;

    let mut entries = vec![];
    for entry in &FEEDS {
        let db = open_db(entry.author, entry.name, &migrator)
            .await
            .expect("create database success");
        entries.push(PollTarget { target: entry, db });
    }

    let poll_handle = tokio::spawn(Box::pin(do_poll_rss(entries.clone())));
    let bot_handle = tokio::spawn(Box::pin(start_bot(entries)));
    let (r1, r2) = tokio::try_join!(poll_handle, bot_handle)?;

    r1?;
    r2?;
    Ok(())
}
