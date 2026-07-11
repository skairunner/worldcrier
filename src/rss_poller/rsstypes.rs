use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Rss {
    #[serde(rename = "channel")]
    pub channels: Option<Vec<Channel>>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Channel {
    pub title: Option<String>,
    pub link: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "item")]
    pub items: Option<Vec<Item>>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Item {
    pub title: String,
    pub description: String,
    pub link: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub guid: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_example() {
        use quick_xml::de::from_str;
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
    <channel>
        <title>world title</title>
        <link>world link</link>
        <language>en</language>
        <description>world description</description>
        <item>
            <title>title 1</title>
            <description>desc1</description>
            <link>link1</link>
            <pubDate>Fri, 03 Jul 2026 08:23:02 +0100</pubDate>
            <guid isPermaLink="false">80561711-a26e-4beb-bb2c-4b34691bdc99</guid>
        </item>
        <item>
            <title>title 2</title>
            <description>desc2</description>
            <link>link2</link>
            <pubDate>Thu, 02 Jul 2026 09:55:18 +0100</pubDate>
            <guid isPermaLink="false">9bf30cd5-3519-476c-a71a-a43d67bf66eb</guid>
        </item>
    </channel>
</rss>
        "#;
        let result: Rss = from_str(xml).unwrap();
        assert_eq!(
            result,
            Rss {
                channels: Some(vec![Channel {
                    title: Some("world title".to_string()),
                    link: Some("world link".to_string()),
                    language: Some("en".to_string()),
                    description: Some("world description".to_string()),
                    items: Some(vec![
                        Item {
                            title: "title 1".to_string(),
                            description: "desc1".to_string(),
                            link: "link1".to_string(),
                            pub_date: "Fri, 03 Jul 2026 08:23:02 +0100".to_string(),
                            guid: "80561711-a26e-4beb-bb2c-4b34691bdc99".to_string(),
                        },
                        Item {
                            title: "title 2".to_string(),
                            description: "desc2".to_string(),
                            link: "link2".to_string(),
                            pub_date: "Thu, 02 Jul 2026 09:55:18 +0100".to_string(),
                            guid: "9bf30cd5-3519-476c-a71a-a43d67bf66eb".to_string(),
                        },
                    ])
                }])
            }
        )
    }
}
