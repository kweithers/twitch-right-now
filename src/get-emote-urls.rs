use fantoccini::{ClientBuilder, Locator};
use std::fs::File;
use std::io::prelude::Write;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
    let c = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");
    let url = "https://betterttv.com/emotes/top".to_owned();
    let mut file = File::create("emote-urls/emote-urls.txt").unwrap();

    let mut emote_set = std::collections::HashSet::new();
    c.goto(url.as_str()).await?;
    sleep(Duration::from_secs(5)).await;
    let xpath = c.find_all(Locator::Css("img")).await?;
    for item in xpath.iter() {
        let emote_name = item.attr("alt").await?.unwrap().to_owned();

        if emote_set.contains(&emote_name) {
            continue
        }

        emote_set.insert(emote_name.clone());
        let mut emote_url = item
            .attr("src")
            .await?
            .unwrap()
            .strip_suffix("/3x.webp")
            .unwrap()
            .to_owned();
        emote_url.push_str("/2x");

        write!(file, "{emote_name}:{emote_url}\n",)?;
    }
    c.close().await
}
