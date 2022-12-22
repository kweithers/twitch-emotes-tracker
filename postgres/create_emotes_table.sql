CREATE TABLE emotes (
	streamer varchar,
  	date DATE,
	hour INTEGER,
	emote varchar,
	n INTEGER,
	PRIMARY KEY (streamer, date, hour, emote)
);