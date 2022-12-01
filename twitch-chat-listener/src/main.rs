mod db_write;

use redis::Commands;
use std::collections::HashSet;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use tokio::time::{sleep, Duration};

#[tokio::main]
pub async fn main() {
    let streamers = include_str!("streamers.txt"); // Top 10,000 from https://sullygnome.com/channels/365/peakviewers
    let emotes = include_str!("emotes.txt");
    let mut emote_set = HashSet::new();
    for emote in emotes.lines() {
        emote_set.insert(emote.to_owned());
    }

    // default configuration joins chat as anonymous.
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let redis_client = redis::Client::open("redis://redis").unwrap();
    let mut redis_con = redis_client.get_connection().unwrap();

    let _postgres_handle = tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(86400)).await; // Run every 24 hours
            db_write::db_write_and_redis_clear().await.unwrap();
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
                            let _: () = redis_con.incr(key, 1).unwrap();
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
