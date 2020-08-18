use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Config {
  pub bots: HashMap<String, Bot>,
  pub account: Account,
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
