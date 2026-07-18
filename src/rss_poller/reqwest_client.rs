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
