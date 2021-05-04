use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Config {
  pub bots: HashMap<String, Bot>,
  pub account: Account,
  pub db: Db,
}

#[derive(Deserialize)]
pub struct Bot {
  pub channels: Vec<String>,
}

#[derive(Deserialize)]
pub struct Account {
  pub user: String,
  pub pass: String,
}

#[derive(Deserialize)]
pub struct Db {
  pub user: String,
  pub pass: String,
  pub hostname: String,
  pub database: String,
}
impl Db {
  pub fn database_url(&self) -> String {
    format!(
      "postgres://{}:{}@{}/{}",
      self.user, self.pass, self.hostname, self.database
    )
  }
}
