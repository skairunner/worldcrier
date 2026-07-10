use crate::data::{RSS_SUFFIX, WatchTarget};
use crate::db::{get_last_run_date, update_run_date, upsert_msgs};
use crate::dbtypes::RssMsg;
use crate::rsstypes::Rss;
use crate::sqliteacquire::SqliteAcquire;
use bytes::Buf;
use chrono::Duration;
use chrono::prelude::{FixedOffset, Utc};
use quick_xml::de::from_reader;
use quick_xml::encoding::DecodingReader;
use reqwest;

const HOUR_S: i64 = 3600;

/// The duration between scanning a given RSS feed
static POLL_INTERVAL: Duration = Duration::new(HOUR_S * 12, 0).unwrap();

pub fn get_client() -> reqwest::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-clacks-overhead", "GNU Gorkam Worka".parse().unwrap());
    headers.insert(
        "user-agent",
        "worldcrier/1.0 reqwest 0.13.4 (contact: ki539@nyu.edu)"
            .parse()
            .unwrap(),
    );

    reqwest::Client::builder().default_headers(headers).build()
}

/// Poll the specified RSS feed.
pub async fn poll_rss_feed(url: &str) -> anyhow::Result<Vec<RssMsg>> {
    let client = get_client()?;
    let res = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let xml: Rss = from_reader(DecodingReader::new(res.reader()))?;

    Ok(xml
        .channels
        .unwrap_or_default()
        .into_iter()
        .flat_map(|channel| channel.items.unwrap_or_default())
        .map(|item| item.into())
        .collect())
}

pub enum RssPoll {
    DidPoll,
    DidNotPoll,
}

/// Fetch the last scan date and poll if needed.
pub async fn poll_rss_if_needed<'a, A: SqliteAcquire<'a>>(
    info: &WatchTarget,
    conn: A,
) -> anyhow::Result<RssPoll> {
    log::debug!("Fetching date");
    let mut conn = conn.acquire().await?;
    let date = get_last_run_date(&mut *conn).await?;
    log::debug!("Date: {date}");
    if Utc::now().fixed_offset() - date < POLL_INTERVAL {
        log::debug!("Skipping.");
        return Ok(RssPoll::DidNotPoll);
    }

    log::debug!("Fetching");
    let items = poll_rss_feed(&format!("{}{}", info.url, RSS_SUFFIX)).await?;
    upsert_msgs(items, &mut *conn).await?;
    update_run_date(&mut *conn).await?;

    log::debug!("OK");
    Ok(RssPoll::DidPoll)
}
