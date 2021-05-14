-- This file should undo anything in `up.sql`
DROP TABLE sing;

DROP TABLE url_metadata;
DROP TABLE url__;

DROP TABLE nickname_preferred;
DROP TABLE nickname__;
DROP RULE ignore_duplicate_inserts_on_chatuser ON chatuser; 
DROP TABLE chatuser;
