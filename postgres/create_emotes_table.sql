CREATE TABLE emotes (
	stream varchar,
  date DATE,
	emote varchar,
	n INTEGER,
	PRIMARY KEY (stream, date, emote)
);