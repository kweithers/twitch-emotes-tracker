Uses the [twitch-irc](https://crates.io/crates/twitch-irc) crate to join the twitch chats of the top 10,000 streamers and listen for hundreds of the top twitch and bttv emotes.

Stores the results in a multihashset and writes to postgres once per day.