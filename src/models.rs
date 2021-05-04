use super::schema::*;

use diesel::{sql_types::*, *};

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[table_name = "sing"]
pub struct Sing {
  pub url: String,
}
