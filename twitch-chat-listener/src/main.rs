mod db_write;

use multiset::HashMultiSet;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_postgres::{Error, NoTls};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // Read in twitch streamers and emotes
    let streamers = include_str!("streamers_long.txt"); // Top 10,000 from https://sullygnome.com/channels/365/peakviewers
    let emotes = include_str!("emotes_long.txt");
    let mut emote_set = HashSet::new();
    for emote in emotes.lines() {
        emote_set.insert(emote.to_owned());
    }
    
    // Connect to the database.
    let (mut postgres_client, postgres_connection) =
        tokio_postgres::connect("host=db user=kmw password=bald", NoTls).await?;
    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = postgres_connection.await {
            println!("connection error: {}", e);
        }
    });

    // default configuration joins chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let data = Arc::new(Mutex::new(HashMultiSet::new()));
    let clone1 = Arc::clone(&data);

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(3600)).await; //Every hour, write to postgres
            db_write::set_write(&mut postgres_client, &clone1)
                .await
                .unwrap();
        }
    });

    // consume incoming Privmsg(s) (standard individual twitch chat messages)
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(message) => {
                    for token in message.message_text.split(" ") {
                        if emote_set.contains(token) {
                            let key = format!("{}:{}", message.channel_login, token);
                            data.lock().await.insert(key);
                        }
                    }
                }
                _ => (),
            }
        }
    });

    // join channels
    for channel in streamers.lines() {
        client.join(channel.to_owned()).unwrap()
    }

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
    Ok(())
}
