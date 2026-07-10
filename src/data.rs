pub struct WatchTarget {
    pub name: &'static str,
    pub author: &'static str,
    pub url: &'static str,
}

pub static FEEDS: [WatchTarget; 1] = [WatchTarget {
    name: "Solaris",
    author: "nnie",
    url: "https://www.worldanvil.com/w/solaris-nnie",
}];

pub const RSS_SUFFIX: &str = "/opendata/rss";
