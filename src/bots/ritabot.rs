use crate::bots::*;
use crate::instant_waiter::*;

use clap::Clap;
use log::error;
use std::sync::Mutex;

pub fn ritabot(execution_last: Mutex<InstantWaiter>) -> impl Bot + 'static {
  #[derive(Clap)]
  enum Opt {
    Ping {},
    Ud { term: String },
  }
  use Opt::*;
  clap_bot("rita", "        Dr. Ritarost", move |opt: Opt, _post| {
    Ok(Some(match opt {
      Ping {} => "hallu".to_owned(),
      Ud { term } => ud_lookup(term, &execution_last).unwrap_or("weiß nicht".to_owned()),
    }))
  })
}

fn ud_lookup(term: String, execution_last: &Mutex<InstantWaiter>) -> Result<String, String> {
  if term == "" {
    Err("Du musst schon einen Begriff angeben")?
  }

  execution_last
    .lock()
    .map_err(|error| {
      error!("{}", error);
      "internal error"
    })?
    .wait_for_permission();

  let obj = reqwest::blocking::get(format!(
    "http://api.urbandictionary.com/v0/define?term={term}",
    term = term
  ))
  .map_err(|e| {
    error!("ud api error: {}", e.to_string());
    "ud: connection_error"
  })?
  .text()
  .map_err(|e| {
    error!("{}", e);
    "error: no text received"
  })?;
  let obj: serde_json::value::Value = serde_json::from_str(&obj).map_err(|e| {
    error!("{}", e);
    "parsing error"
  })?;
  Ok(
    obj
      .get("list")
      .ok_or("parsing error")?
      .as_array()
      .ok_or("parsing error")?[0]
      .get("definition")
      .ok_or("no definitions available")?
      .as_str()
      .ok_or("parsing error")?
      .to_owned(),
  )
}
