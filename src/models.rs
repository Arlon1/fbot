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
#[table_name = "qedmitglied"]
pub struct Qedmitglied {
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

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "nickname_preferred"]
pub struct NicknamePreferred {
  pub userid: i32,
  pub preferred: String,
}

#[derive(Debug, Queryable, QueryableByName)]
#[table_name = "ping"]
pub struct PingQuery {
  pub id: i32,
  pub sender: Option<i32>,
  pub receiver: String,
  pub sent: NaiveDateTime,
  pub scheduled: Option<NaiveDateTime>,
  pub message: String,
}
impl std::fmt::Display for PingQuery {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} sagte: {}", "", self.message)
  }
}

#[derive(Debug, Insertable)]
#[table_name = "ping"]
pub struct PingInsert {
  pub sender: Option<i32>,
  pub receiver: String,
  pub sent: NaiveDateTime,
  pub scheduled: Option<NaiveDateTime>,
  pub message: String,
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "freiepunkte"]
pub struct FreiePunkte {
  pub id: i32,
  pub name: String,
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "freiepunkte_values"]
pub struct FreiePunkteValues {
  pub id: i32,
  pub userid: i32,
  pub wert: i32,
}
