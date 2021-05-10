use crate::{lib::Youtube, models, schema /* schema::sing::dsl::* */};

use clap::Clap;
use diesel::{prelude::*, result::DatabaseErrorKind, PgConnection};
use url::Url;

#[derive(Clap, Debug)]
pub enum SingMode {
  Sing,

  Learn {
    url: String,
  },
  Replace {
    oldurl: String,
    with: String,
    newurl: String,
  },
  Remove {
    url: String,
  },

  Count,
}

pub fn sing_count(conn: &PgConnection) -> usize {
  let count = schema::sing::table
    .count()
    .execute(conn)
    .expect("could not load table");
  count
}

pub fn sing(mode: Option<SingMode>, post: &qedchat::RecvPost, conn: &PgConnection) -> String {
  fn annotation(url: Url) -> Option<String> {
    let y = Youtube::from_url(&url)?;
    Some(y.annotation()?)
  }

  let m = mode.unwrap_or(SingMode::Sing);
  match m {
    SingMode::Sing => {
      let url_res = diesel::dsl::sql_query("SELECT * FROM sing ORDER BY random() LIMIT 1;")
        .get_result::<models::Sing>(conn);
      if let Err(ref err) = url_res {
        if err == &diesel::result::Error::NotFound {
          return "ich kenne keinen Song".to_owned();
        }
      }
      let url = url_res.expect("Error loading random url from `sing`").url;

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
      if let Some(_) = Url::parse(&url).ok() {
        match diesel::insert_into(schema::sing::table)
          .values(models::Sing {
            url: url.clone(),
            added: post.date.naive_local(),
            added_by: post
              .username
              .as_ref()
              .cloned()
              .unwrap_or(post.post.name.clone()),
            last_access: post.date.naive_local(),
          })
          .execute(conn)
        {
          Ok(_) => {
            let _metadata = schema::urls::table
              //.filter(url.eq(true))
              .load::<models::Urls>(conn)
              .expect("Error loading urls");
            /*dbg!(metadata);
            match metadata.len() {
              0 =>{},
              1=>{},
              _ => {},
            }*/
            //format!(
            //  "Ich kann was Neues singen.{}",
            //  annotation(url_obj)
            //    .map(|s| "\n".to_owned() + &s)
            //    .unwrap_or("".to_owned())
            //)
            "ich kann jetzt singen".to_owned()
          }
          Err(err) => match err {
            diesel::result::Error::DatabaseError(e, _) => match e {
              DatabaseErrorKind::UniqueViolation => "kenn ich schon".to_owned(),
              _ => "anderer Datenbank-Fehler".to_owned(),
            },
            _ => "was ist das überhaupt für ein Fehler?".to_owned(),
          },
        }
      } else {
        "ungültiger Link".to_owned()
      }
    }
    SingMode::Remove { url } => "todo".to_owned(),
    SingMode::Replace {
      oldurl,
      with,
      newurl,
    } => "todo".to_owned(),
    SingMode::Count => format!("Ich kann schon {} Lieder singen.", sing_count(conn)),
  }
}
