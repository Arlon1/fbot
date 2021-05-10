use crate::{bots::*, lib::*};

use clap::Clap;
use diesel::PgConnection;
use log::error;
use sha1::{Digest, Sha1};
use std::sync::Mutex;

mod dual_error;
mod sing;
mod ud;

use dual_error::*;
use sing::*;
use ud::*;

pub fn ritabot(
  execution_last: Mutex<InstantWaiter>,
  conn: Mutex<PgConnection>,
) -> impl Bot + 'static {
  #[derive(Clap)]
  enum Opt {
    Ping {},
    Ud {
      term: String,
    },
    Decide {
      _terms: Vec<String>,
    },
    Sing {
      #[clap(subcommand)]
      mode: Option<SingMode>,
    },

    //Be { name: String },
    Say {
      a: Vec<String>,
    },
    Slap {
      targets: Vec<String>,
    },
    Featurerequest {
      features: Vec<String>,
    },
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
      Sing { mode } => {
        let c = conn.lock().unwrap();
        let cc = c.deref();

        sing(mode, post, cc)
      }
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
