use chrono::prelude::{DateTime, FixedOffset};

use crate::rss_poller::rsstypes;

/// An RSS message from the database with the date parsed.
pub struct RssMsg {
    pub title: String,
    pub description: String,
    pub link: String,
    pub guid: String,
    pub pub_date_raw: String,
    pub pub_date: Option<DateTime<FixedOffset>>,
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
        }
    }
}
