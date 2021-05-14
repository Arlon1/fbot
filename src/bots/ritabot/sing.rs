use crate::{lib::Youtube, models, schema};

use chrono::offset::Local;
use clap::Clap;
use diesel::{prelude::*, result::DatabaseErrorKind, PgConnection};
use qedchat::RecvPost;
use url::Url;

#[derive(Clap, Debug)]
pub enum SingMode {
  Sing,

  #[clap(aliases = &["a", "l", "add"])]
  Learn {
    url: String,
  },
  Replace {
    oldurl: String,
    newurl: String,
  },
  Remove {
    url: String,
  },

  Count,
}

fn annotation(url: Url) -> Option<String> {
  let y = Youtube::from_url(&url)?;
  Some(y.annotation()?)
}

fn sing_url(post: &RecvPost, conn: &PgConnection) -> String {
  let url_res = diesel::dsl::sql_query("SELECT * FROM sing ORDER BY random() LIMIT 1;")
    .get_result::<models::Sing>(conn);
  if let Err(ref err) = url_res {
    if err == &diesel::result::Error::NotFound {
      return "Ich kenne keinen Song".to_owned();
    }
  }
  let url = url_res.expect("Error loading random url from `sing`").url;

  diesel::update(schema::sing::table.filter(schema::sing::dsl::url.eq(&url)))
    .set(schema::sing::dsl::last_access.eq(post.date))
    .execute(conn)
    .expect("Updating last access for url failed");

  if let Some(url_obj) = Url::parse(&url).ok() {
    match annotation(url_obj) {
      Some(text) => format!("{}\n{}", url, text),
      None => url,
    }
  } else {
    url
  }
}

fn learn_url(url: &str, post: &RecvPost, conn: &PgConnection) -> String {
  if let Some(url_obj) = Url::parse(&url).ok() {
    match diesel::insert_into(schema::sing::table)
      .values(models::Sing {
        url: url.to_owned(),
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
        let urls_list = schema::url__::table
          .filter(schema::url__::dsl::url.eq(&url))
          .load::<models::Urls>(conn)
          .expect("Error loading urls");
        dbg!(&urls_list);
        let url_value = match urls_list.len() {
          0 => diesel::insert_into(schema::url__::table)
            .values(models::Urls {
              url: url.to_owned(),
              last_updated: Local::now().naive_local(),
            })
            .get_result::<models::Urls>(conn)
            .expect("Error creating an metadata entry for `url`"),

          _ => urls_list[0].clone(),
        };

        if url_value
          .last_updated
          .signed_duration_since(Local::now().naive_local())
          .num_weeks()
          >= 4
        {
          let y = Youtube::from_url(&url_obj).unwrap(); // todo (unwrap)

          let metadata = &schema::url_metadata::table
            .filter(schema::url_metadata::url.eq(&url))
            .load::<models::UrlMetadata>(conn)
            .expect("Error querying url_metadata")[0];

          if y != Youtube::from_metadata(metadata) {
            diesel::update(
              schema::url_metadata::table.filter(schema::url_metadata::dsl::url.eq(&url)),
            )
            .set((
              schema::url_metadata::dsl::title.eq(y.title()),
              schema::url_metadata::dsl::author.eq(y.channel()),
            ))
            .execute(conn)
            .expect("");
          }

          diesel::update(schema::url__::table.filter(schema::url__::dsl::url.eq(&url)))
            .set(schema::url__::dsl::last_updated.eq(Local::now()))
            .execute(conn)
            .expect("Error updating `last_updated` in table `urls`");
        }

        format!(
          "Ich kann was Neues singen:{}",
          annotation(url_obj)
            .map(|s| "\n".to_owned() + &s)
            .unwrap_or("".to_owned())
        )
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

fn forget_url(url: &str, conn: &PgConnection) -> String {
  match diesel::delete(schema::sing::table.filter(schema::sing::dsl::url.eq(url)))
    .execute(conn)
    .expect("Error deleting sing url")
  {
    0 => "Kenn ich nicht".to_owned(),
    _ => format!("ich habe vergessen was vergessen: {}", url),
  }
}

fn sing_count(conn: &PgConnection) -> usize {
  let count = schema::sing::table
    .count()
    .execute(conn)
    .expect("could not load table");
  count
}

pub fn sing(mode: Option<SingMode>, post: &RecvPost, conn: &PgConnection) -> String {
  let m = mode.unwrap_or(SingMode::Sing);
  match m {
    SingMode::Sing => sing_url(post, conn),

    SingMode::Learn { url } => learn_url(&url, post, conn),
    SingMode::Remove { url } => forget_url(&url, conn),

    SingMode::Replace { oldurl, newurl } => {
      forget_url(&oldurl, conn);
      learn_url(&newurl, post, conn);
      "ich habe x durch y ersetzt".to_owned()
    }
    SingMode::Count => format!("Ich kann schon {} Lieder singen.", sing_count(conn)),
  }
}
