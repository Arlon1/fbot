use crate::{bots::*, lib::*, models, schema::sing as table_sing /*::dsl::**/};
use clap::Clap;
use diesel::{prelude::*, PgConnection};
use log::error;
use sha1::{Digest, Sha1};
use std::sync::Mutex;
use url::Url;

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

        fn annotation(url: Url) -> Option<String> {
          let y = Youtube::from_url(&url)?;
          Some(y.annotation()?)
        }

        let m = mode.unwrap_or(SingMode::Sing);
        match m {
          SingMode::Sing => {
            let url: String =
              diesel::dsl::sql_query("SELECT * FROM sing ORDER BY random() LIMIT 1;")
                .get_result::<models::Sing>(cc)?
                .url;
            if let Some(url_obj) = Url::parse(&url).ok() {
              match annotation(url_obj) {
                Some(text) => format!("{}\n{}", url, text),
                None => url,
              }
            } else {
              url
            }
          }
          SingMode::Learn { url } => {
            if let Some(url_obj) = Url::parse(&url).ok() {
              diesel::insert_into(table_sing::table)
                .values(models::Sing { url: url.clone() })
                .get_result::<models::Sing>(cc)?;
              format!(
                "Ich kann was Neues singen.{}",
                annotation(url_obj)
                  .map(|s| "\n".to_owned() + &s)
                  .unwrap_or("".to_owned())
              )
            } else {
              "Gib eine URL an".to_owned()
            }
          }
          SingMode::Count => format!("Ich kann schon {} Lieder singen.", sing_count(cc)),
        }
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
