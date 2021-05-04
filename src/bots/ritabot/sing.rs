use crate::{models, schema::sing::dsl::*};

use clap::Clap;
use diesel::{prelude::*, sql_types::Text, PgConnection};

#[derive(Clap, Debug)]
pub enum SingMode {
  Sing,
  Learn { url: String },
  Count,
}

pub fn sing_count(conn: &PgConnection) -> usize {
  let count = sing.count().execute(conn).expect("could not load table");
  count
}
