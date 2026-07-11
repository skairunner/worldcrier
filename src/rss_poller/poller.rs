use sqlx::{SqlitePool, migrate::Migrator};

use crate::{
    data::WatchTarget,
    rss_poller::{
        constants::{LOOP_INTERVAL_STD, REQUEST_INTERVAL_STD},
        poll_rss_if_needed,
    },
};

use super::rss::RssPoll;

pub struct PollEntry {
    pub target: &'static WatchTarget,
    pub db: SqlitePool,
}

/// Infinite task to poll for rss periodically.
pub async fn do_poll_rss(entries: Vec<PollEntry>) -> anyhow::Result<()> {
    loop {
        for entry in entries.iter() {
            let mut tx = entry.db.begin().await?;
            let did_poll = poll_rss_if_needed(entry.target, &mut tx).await?;
            tx.commit().await?;
            match did_poll {
                RssPoll::DidPoll => {
                    tokio::time::sleep(REQUEST_INTERVAL_STD).await;
                }
                RssPoll::DidNotPoll => {}
            }
        }
        tokio::time::sleep(LOOP_INTERVAL_STD).await;
    }
}
