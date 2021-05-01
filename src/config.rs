use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Config {
  pub bots: HashMap<String, Bot>,
  pub account: Account,
  pub db: DB,
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
pub struct DB {
  pub url: String,
}
