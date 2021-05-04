use anyhow::{Context, Result};
use std::sync::Mutex;

use super::dual_error::*;
use crate::lib::*;

pub fn ud_lookup(
  term: String,
  execution_last: &Mutex<InstantWaiter>,
) -> Result<String, anyhow::Error> {
  if term == "" {
    anyhow::bail!("Du musst schon einen Begriff angeben");
  }

  execution_last
    .lock()
    .map_err(|error| DualError::new("internal error. try again".to_owned(), error.to_string()))?
    .wait_for_permission();

  let obj = reqwest::blocking::get(format!(
    "http://api.urbandictionary.com/v0/define?term={term}",
    term = term
  ))
  .map_err(|e| {
    DualError::new(
      "ud: connection_error".to_owned(),
      format!("ud api error: {}", e.to_string()),
    )
  })?
  .text()
  .map_err(|e| DualError::new("error: no text received".to_owned(), e.to_string()))?;
  let obj: serde_json::value::Value = serde_json::from_str(&obj)
    .map_err(|e| DualError::new("parsing error".to_owned(), e.to_string()))?;
  Ok(
    obj
      .get("list")
      .context("parsing error")?
      .as_array()
      .context("parsing error")?
      .iter()
      .nth(0)
      .context("kenne ich nicht")?
      .get("definition")
      .context("no definitions available")?
      .as_str()
      .context("parsing error")?
      .to_owned(),
  )
}
