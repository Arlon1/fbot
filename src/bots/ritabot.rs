use crate::{bots::*, instant_waiter::*, string_storage::*};

use anyhow::{Context, Result};
use clap::Clap;
use log::error;
use sha1::{Digest, Sha1};
use std::sync::Mutex;

pub fn ritabot(
  execution_last: Mutex<InstantWaiter>,
  name_being: Mutex<StringStorage>,
) -> impl Bot + 'static {
  #[derive(Clap)]
  enum Opt {
    Ping {},
    Ud { term: String },
    Decide { _terms: Vec<String> },
    Sing { _a: Vec<String> },

    //Be { name: String },
    Say { a: Vec<String> },
    Slap { targets: Vec<String> },
    Featurerequest { features: Vec<String> },
  }
  use Opt::*;

  clap_bot("rita", "        Dr. Ritarost", move |opt: Opt, post| {
    Ok(Some(match opt {
      Ping {} => "hallu".to_owned(),
      Ud { term } => match ud_lookup(term, &execution_last) {
        Ok(description) => description.replace("\n\n", "\n"),
        Err(e) => {
          if let Some(e) = e.downcast_ref::<DualError>() {
            error!("{}", e.underlying());
          }
          e.to_string()
        }
      },
      Decide { _terms } => {
        let seed =  b"tpraR4gin8XHk_t3bGHZTJ206qc9vyV7LlUMTf655LNJDKGciVXKRLijqGkHgkpW <= Manfreds schlimmstes Geheimnis";
        let mut text = post.post.message.clone().into_bytes();
        text.extend_from_slice(seed);
        let hash = Sha1::digest(&text);

        if format!("{:x}", hash).chars().nth(0).unwrap() as i64 % 2 == 1 {
          "+".to_owned()
        } else {
          "-".to_owned()
        }
      }
      /*Be { name } => {
        //name_storage.set_s(name.to_owned());
        //format!("Ich bin jetzt {}", name)
      }*/
      Sing { _a } => return Ok(None),
      Say { a } => {
        format!("{:?}", a)
      }
      Slap { targets } => {
        let mut targets = targets.join(" ");
        if !targets.ends_with(".") {
          targets += ".";
        }
        format!("Rita schlägt {}", targets)
      }
      Featurerequest { features } => {
        let mut features = features.join(" ");
        if !features.ends_with(".") {
          features += ".";
        }
        format!("Ich will {}", features)
      }
    }))
  })
}

#[derive(Debug, thiserror::Error)]
#[error("{display_error}")]
struct DualError {
  display_error: String,
  underlying: String,
}
impl DualError {
  pub fn new(display_error: String, underlying: String) -> Self {
    Self {
      display_error,
      underlying,
    }
  }
  pub fn underlying(&self) -> &str {
    &self.underlying
  }
}

fn ud_lookup(term: String, execution_last: &Mutex<InstantWaiter>) -> Result<String, anyhow::Error> {
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
