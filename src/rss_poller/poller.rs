use sqlx::SqlitePool;
use tracing::instrument;

use crate::{
    data::WatchTarget,
    rss_poller::{
        constants::{LOOP_INTERVAL_STD, REQUEST_INTERVAL_STD},
        poll_rss_if_needed,
    },
};

use super::rss::RssPoll;

#[derive(Debug, Clone)]
pub struct PollTarget {
    pub target: &'static WatchTarget,
    pub db: SqlitePool,
}

impl PollTarget {
    /// Whether the author and world name matches this poll target
    pub fn matches(&self, author: &str, world_name: &str) -> bool {
        return author == self.target.author && world_name == self.target.name;
    }
}

/// Infinite task to poll for rss periodically.
#[instrument]
pub async fn do_poll_rss(entries: Vec<PollTarget>) -> anyhow::Result<()> {
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
