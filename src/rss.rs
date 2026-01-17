use reqwest::Client;
use feed_rs::parser;
use std::error::Error;

pub async fn fetch_feed(client: &Client, url: &str) -> Result<feed_rs::model::Feed, Box<dyn Error + Send + Sync>> {
    let resp = client.get(url).send().await?;
    let content = resp.bytes().await?;
    let feed = parser::parse(&content[..])?;
    Ok(feed)
}
