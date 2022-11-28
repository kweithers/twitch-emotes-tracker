use redis::Commands;
use std::collections::HashSet;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
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

    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut redis_con = redis_client.get_connection().unwrap();

    // consume incoming Privmsg(s) (standard individual twitch chat messages)
    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(message) => {
                    let mut print = false;
                    for token in message.message_text.split(" ") {
                        if emote_set.contains(token) {
                            let key = format!(
                                "{}:{}:{}",
                                message.channel_login,
                                message.server_timestamp.format("%Y%m%d"),
                                token
                            );
                            let _: () = redis_con.incr(key, 1).unwrap();
                            print = true;
                        }
                    }
                    if print {
                        println!(
                            "{} {} - {}: {}",
                            message.server_timestamp,
                            message.channel_login,
                            message.sender.name,
                            message.message_text
                        );
                    }
                }
                _ => (),
            }
        }
    });

    // join channels
    for channel in streamers.lines() {
        // assert!(client.join(channel.to_owned()) == Ok(()));
        client.join(channel.to_owned()).unwrap();
    }

    // keep the tokio executor alive.
    // If you return instead of waiting the background task will exit.
    join_handle.await.unwrap();
}
