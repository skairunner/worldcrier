CREATE TABLE rss_msg (
    title TEXT,
    description TEXT,
    link TEXT,
    guid TEXT,
    pub_date TEXT,
    sent INTEGER
);
CREATE UNIQUE INDEX idx_rss_msg_guid ON rss_msg (guid);
CREATE INDEX idx_rss_msg_sent ON rss_msg (sent);

-- Store info about the last time the rss feed was checked.
CREATE TABLE rss_scan (
    time TEXT
);

INSERT INTO rss_scan(time) VALUES ('2026-07-10T09:23:05Z');