use super::schema::*;

use chrono::NaiveDateTime;
use diesel::pg::data_types::PgInterval;

use diesel::*;

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "sing"]
pub struct Sing {
  pub url: String,
  pub added: NaiveDateTime,
  pub added_by: String,
  pub last_access: NaiveDateTime,
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "urls"]
pub struct Urls {
  pub url: String,
  pub last_updated: NaiveDateTime,
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "url_metadata"]
pub struct UrlMetadata {
  pub url: String,
  pub title: String,
  pub author: String,
  pub duration: PgInterval,
  pub start_time: PgInterval,
}
