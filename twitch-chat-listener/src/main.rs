mod db_write;

// use redis::Commands;
use std::collections::HashSet;
use std::sync::{Arc};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use tokio::time::{Duration,interval};
use tokio::sync::{Mutex};
use multiset::HashMultiSet;

#[tokio::main]
pub async fn main() {
    let streamers = include_str!("streamers_long.txt"); // Top 10,000 from https://sullygnome.com/channels/365/peakviewers
    let emotes = include_str!("emotes_long.txt");
    let mut emote_set = HashSet::new();
    for emote in emotes.lines() {
        emote_set.insert(emote.to_owned());
    }

    // default configuration joins chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let data = Arc::new(Mutex::new(HashMultiSet::new()));
    let clone1 = Arc::clone(&data);

    let _postgres_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(86400)); //24 hrs
        interval.tick().await; //First tick happens immediately
        loop {
            interval.tick().await; //Every 24 hrs, write to postgres from redis
            db_write::set_write(&clone1).await.unwrap();
        }
    });

    // consume incoming Privmsg(s) (standard individual twitch chat messages)
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(message) => {
                    for token in message.message_text.split(" ") {
                        if emote_set.contains(token) {
                            let key = format!(
                                "{}:{}",
                                message.channel_login,
                                token
                            );
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
        client.join(channel.to_owned()).unwrap();
    }

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}
