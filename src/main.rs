use simplelog::TermLogger;

use crate::{
    data::FEEDS,
    db::{get_migrator, open_db},
    rss_poller::{
        poll_rss_if_needed,
        poller::{PollEntry, do_poll_rss},
    },
};

mod data;
mod db;
mod dbtypes;
mod rss_poller;
mod sqliteacquire;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    TermLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Always,
    )?;

    let migrator = get_migrator().await?;

    let mut entries = vec![];
    for entry in &FEEDS {
        let db = open_db(entry.author, entry.name, &migrator)
            .await
            .expect("create database success");
        entries.push(PollEntry { target: entry, db });
    }
    do_poll_rss(entries).await
}
