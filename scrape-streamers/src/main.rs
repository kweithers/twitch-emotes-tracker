use fantoccini::{ClientBuilder, Locator};
use std::fs::File;
// use std::fs::OpenOptions;
use std::io::prelude::Write;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
    let c = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");
    let mut url = "https://twitchtracker.com/channels/ranking".to_owned();
    let mut file = File::create("twitch-chat-listener/src/streamers.txt").unwrap();
    // let mut file = OpenOptions::new()
    //     .append(true)
    //     .open("twitch-chat-listener/src/streamers.txt")
    //     .unwrap();

    for i in 1..=200 {
        c.goto(url.as_str()).await?;
        sleep(Duration::from_secs(5)).await;
        let xpath = c.find_all(Locator::Css("tr > td > a")).await?;
        for item in xpath.iter().step_by(2) {
            let n = item
                .attr("href")
                .await?
                .unwrap()
                .strip_prefix("/")
                .unwrap()
                .to_owned();
            write!(file, "{}\n", n)?;
        }
        if i > 1 {
            url = url
                .strip_suffix(format!("?page={index}", index = i).as_str())
                .unwrap()
                .to_owned();
        }
        url.push_str(format!("?page={index}", index = i + 1).as_str());
    }

    c.close().await
}
