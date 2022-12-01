CREATE TABLE emotes (
	streamer varchar,
  date DATE,
	emote varchar,
	n INTEGER,
	PRIMARY KEY (streamer, date, emote)
);