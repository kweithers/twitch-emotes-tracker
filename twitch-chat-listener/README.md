Uses the [twitch-irc](https://crates.io/crates/twitch-irc) crate to join the twitch chats of the top 10,000 streamers and listen for hundreds of the top twitch and bttv emotes.

Stores the results in a [redis](https://redis.io/) data store with keys of the format "streamer:date:emote".

Future goals:

1. Use the redis data store to do daily inserts into a postgresql database. This database will support a web server that can be used to query historical data, showing graphs and comparisons across users/emotes.

2. Create a 'state-of-twitch' or 'twitch-right-now' web server that uses websockets to show a left-to-right scrolling stream of recent emotes for many rows of streamers.