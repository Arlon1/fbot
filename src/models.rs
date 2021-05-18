use super::schema::*;

use chrono::NaiveDateTime;
use diesel::*;
use diesel_chrono_duration::*;
use serde::Deserialize;
use std::cmp::PartialEq;

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "sing"]
pub struct Sing {
  pub url: String,
  pub added: NaiveDateTime,
  pub added_by: String,
  pub last_access: NaiveDateTime,
}

#[derive(Clone, Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "url__"]
pub struct Urls {
  pub url: String,
  pub last_updated: NaiveDateTime,
}

#[derive(AsChangeset, Clone, Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "url_metadata"]
pub struct UrlMetadata {
  pub url: String,
  pub title: Option<String>,
  pub author: Option<String>,
  pub duration: Option<ChronoDurationProxy>,
  pub start_time: Option<ChronoDurationProxy>,
}
impl PartialEq for UrlMetadata {
  fn eq(&self, other: &Self) -> bool {
    self.title == other.title
      && self.author == other.author
      && self.duration == other.duration
      && self.start_time == other.start_time
  }
}

#[derive(Debug, Deserialize, Insertable, Queryable, QueryableByName)]
#[table_name = "chatuser"]
pub struct Chatuser {
  #[serde(alias = "Id")]
  pub userid: i32,
  #[serde(alias = "Benutzername")]
  pub username: String,
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "nickname__"]
pub struct Nickname {
  pub userid: i32,
  pub nickname: String,
}
