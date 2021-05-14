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
	duration	BigInt
);

CREATE TABLE chatuser (
	userid		Integer PRIMARY KEY,
	username	Text NOT NULL,
	  UNIQUE(username)

);
CREATE OR REPLACE RULE ignore_duplicate_inserts_on_chatuser AS ON INSERT TO chatuser
  WHERE (EXISTS (SELECT 1 FROM chatuser WHERE new.userid = chatuser.userid))
  DO INSTEAD NOTHING;


CREATE TABLE nickname__ (
	userid		Integer NOT NULL REFERENCES chatuser(userid),
	nickname		Text PRIMARY KEY
);

CREATE TABLE nickname_preferred (
	userid		Integer PRIMARY KEY REFERENCES chatuser(userid),
	preferred	Text REFERENCES nickname__(nickname)
);
