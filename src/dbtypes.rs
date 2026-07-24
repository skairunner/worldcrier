use chrono::prelude::{DateTime, FixedOffset};
use sqlx::{
    prelude::{FromRow, Row},
    sqlite::SqliteRow,
};

use crate::rss_poller::rsstypes;

/// An RSS message from the database with the date parsed.
pub struct RssMsg {
    pub title: String,
    pub description: String,
    pub link: String,
    pub guid: String,
    pub pub_date_raw: String,
    pub pub_date: Option<DateTime<FixedOffset>>,
    pub image_url: String,
}

impl RssMsg {
    /// Serialize the date into a standard format.
    pub fn serialize_date(&self) -> String {
        if let Some(date) = &self.pub_date {
            date.to_rfc3339()
        } else {
            self.pub_date_raw.clone()
        }
    }
}

impl FromRow<'_, SqliteRow> for RssMsg {
    fn from_row(row: &'_ SqliteRow) -> sqlx::Result<Self> {
        let title = row.try_get("title")?;
        let description = row.try_get("description")?;
        let link = row.try_get("link")?;
        let guid = row.try_get("guid")?;
        let pub_date_raw: String = row.try_get("pub_date")?;
        let pub_date = DateTime::parse_from_rfc3339(&pub_date_raw).ok();
        let image_url = row.try_get("image_url")?;

        Ok(Self {
            title,
            description,
            link,
            guid,
            pub_date_raw,
            pub_date,
            image_url,
        })
    }
}

impl From<rsstypes::Item> for RssMsg {
    fn from(value: rsstypes::Item) -> Self {
        // We need to try parsing the pub date!
        let pub_date_parsed = DateTime::parse_from_rfc2822(&value.pub_date).ok();
        Self {
            title: value.title,
            description: value.description,
            link: value.link,
            guid: value.guid,
            pub_date_raw: value.pub_date,
            pub_date: pub_date_parsed,
            image_url: String::new(),
        }
    }
}

pub struct TargetChannel {
    pub guild_id: u64,
    pub channel_id: u64,
}

impl FromRow<'_, SqliteRow> for TargetChannel {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        let guild_id = row
            .try_get::<String, _>("guild_id")?
            .parse::<u64>()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let channel_id = row
            .try_get::<String, _>("channel_id")?
            .parse::<u64>()
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        Ok(Self {
            guild_id,
            channel_id,
        })
    }
}
