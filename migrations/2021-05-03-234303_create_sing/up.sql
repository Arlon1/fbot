-- Your SQL goes here
CREATE TABLE sing (
	url		Text	NOT NULL PRIMARY KEY,
	added		TIMESTAMP WITH TIME ZONE NOT NULL,
	added_by 	Text	NOT NULL,
	last_access	TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE urls (
	url		Text	NOT NULL PRIMARY KEY,
	last_updated	TIMESTAMP WITH TIME ZONE NOT NULL
);
CREATE TABLE url_metadata (
	url		Text 	PRIMARY KEY REFERENCES urls(url),
	title		Text	NOT NULL,
	author		Text	NOT NULL,
	duration	Interval NOT NULL,
	start_time	Interval NOT NULL
);
