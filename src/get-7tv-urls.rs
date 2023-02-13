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
    let mut url = "https://7tv.app/emotes?page=1".to_owned();
    let mut file = File::create("emote-urls/7tv-urls.txt").unwrap();

    let mut emote_set = std::collections::HashSet::new();

    for i in 1..=50 {
        c.goto(url.as_str()).await?;
        sleep(Duration::from_secs(10)).await;
        let xpath = c.find_all(Locator::Css(".emote-card")).await?;
        for item in xpath.iter() {
            let emote_name = item
                .find(Locator::Css("span"))
                .await?
                .text()
                .await?
                .to_owned();

            if emote_set.contains(&emote_name) {
                continue;
            }
            emote_set.insert(emote_name.clone());

            let emote_url = item
                .find(Locator::Css("img"))
                .await?
                .attr("src")
                .await?
                .unwrap();

            write!(file, "{emote_name}:{emote_url}\n",)?;
        }
        url = url
            .strip_suffix(format!("{index}", index = i).as_str())
            .unwrap()
            .to_owned();

        url.push_str(format!("{index}", index = i + 1).as_str());
    }
    c.close().await
}
