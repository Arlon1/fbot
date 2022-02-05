-- Your SQL goes here
CREATE TABLE sing (
	url		Text	NOT NULL PRIMARY KEY,
	added		TIMESTAMP WITH TIME ZONE NOT NULL,
	added_by 	Text	NOT NULL,
	last_access	TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE url__ (
	url		Text	NOT NULL PRIMARY KEY,
	last_updated	TIMESTAMP WITH TIME ZONE NOT NULL
);
CREATE TABLE url_metadata (
	url		Text 	PRIMARY KEY REFERENCES url__(url),
	title		Text,
	author		Text,
	duration	BigInt,
	start_time	BigInt
);

CREATE TABLE qedmitglied (
	userid		Integer PRIMARY KEY,
	username	Text NOT NULL UNIQUE
);

CREATE TABLE nickname__ (
	userid		Integer NOT NULL REFERENCES qedmitglied(userid),
	nickname	Text PRIMARY KEY
);

CREATE TABLE nickname_preferred (
	userid		Integer PRIMARY KEY REFERENCES qedmitglied(userid),
	preferred	Text REFERENCES nickname__(nickname)
);

CREATE TABLE ping (
	id		SERIAL PRIMARY KEY, 
	sender		Integer REFERENCES qedmitglied(userid),
	receiver		Text   NOT NULL,
	sent		TIMESTAMP WITH TIME ZONE NOT NULL,
	scheduled	TIMESTAMP WITH TIME ZONE,
	message 		Text   NOT NULL
);


CREATE TABLE freiepunkte (
	id		SERIAL PRIMARY KEY,
	name		Text NOT NULL
);

CREATE TABLE freiepunkte_values (
	id		Integer PRIMARY KEY REFERENCES freiepunkte(id),
	userid		Integer REFERENCES qedmitglied(userid),
	wert		Integer NOT NULL
);
