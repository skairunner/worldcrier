/*/
CREATE TABLE rss_msgs (
    title TEXT,
    description TEXT,
    link TEXT,
    guid TEXT,
    pub_date TEXT
);
CREATE INDEX idx_rss_msgs_guid ON rss_msgs (guid);
CREATE INDEX idx_rss_msgs_pub_date ON rss_msgs (pub_date);

-- Store the last time the rss feed was checked.
CREATE TABLE rss_scan (
    time TEXT,
);

CREATE TABLE sent_msgs (
    guid TEXT,
    sent_date TEXT,
);
CREATE INDEX idx_sent_msgs_guid ON sent_msgs(guid);
CREATE INDEX idx_sent_msgs_sent_date ON sent_msgs(sent_date);
*/

use chrono::prelude::{DateTime, FixedOffset};

use crate::rsstypes;

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
