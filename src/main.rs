use simplelog::TermLogger;

use crate::{
    data::FEEDS,
    db::{get_migrator, open_db},
    rss::poll_rss_if_needed,
};

mod data;
mod db;
mod dbtypes;
mod rss;
mod rsstypes;
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
    let pool = open_db("nnie", "solaris", &migrator).await?;
    let mut tx = pool.begin().await?;
    poll_rss_if_needed(&FEEDS[0], &mut tx).await?;
    tx.commit().await?;

    Ok(())
}
