use crate::{lib::Youtube, models, schema};

use anyhow::Result;
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
  #[clap(aliases = &["repl"])]
  Replace {
    oldurl: String,
    newurl: String,
  },
  #[clap(aliases = &["r", "del", "delete", "forget"])]
  Remove {
    url: String,
  },

  #[clap(alias = "c")]
  Count,
}

fn annotation_from_metadata(metadata: Option<models::UrlMetadata>) -> Option<String> {
  Youtube::from_metadata(&metadata?)?.annotation()
}

fn insert_or_update_metadata(
  url: &Url,
  post: &RecvPost,
  conn: &PgConnection,
) -> Result<Option<models::UrlMetadata>> {
  let url_obj = url;
  let url = url_obj.to_string();

  let urls_list = schema::url__::table
    .filter(schema::url__::dsl::url.eq(&url))
    .load::<models::Urls>(conn)
    .expect("Error loading urls");

  let update_metadata = match urls_list.len() {
    0 => {
      diesel::insert_into(schema::url__::table)
        .values(models::Urls {
          url: url.to_owned(),
          last_updated: post.date.naive_local(),
        })
        .execute(conn)
        .expect("Error creating an metadata entry for `url`");
      true
    }

    _ => {
      urls_list[0]
        .last_updated
        .signed_duration_since(Local::now().naive_local())
        .num_weeks()
        >= 12
    }
  };

  let old_metadata = &schema::url_metadata::table
    .filter(schema::url_metadata::url.eq(&url))
    .load::<models::UrlMetadata>(conn)
    .expect("Error querying url_metadata")
    .last()
    .to_owned()
    .cloned();

  diesel::update(schema::url__::table.filter(schema::url__::dsl::url.eq(&url)))
    .set(schema::url__::dsl::last_updated.eq(Local::now()))
    .execute(conn)
    .expect("Error updating `last_updated` in table `urls`");

  let y = Youtube::from_url(&url_obj).map(|y| y.to_metadata());

  Ok(if update_metadata {
    if old_metadata != &y {
      if let Some(new_metadata) = y {
        match old_metadata {
          None => {
            let query = diesel::insert_into(schema::url_metadata::table).values(&new_metadata);
            query
              .execute(conn)
              .expect("Could not insert into url_metadata");
            Some(new_metadata)
          }
          Some(_) => {
            diesel::update(
              schema::url_metadata::table.filter(schema::url_metadata::dsl::url.eq(&url)),
            )
            .set(&new_metadata)
            .execute(conn)
            .expect("Could not update url_metadata");
            Some(new_metadata)
          }
        }
      } else {
        diesel::delete(schema::url_metadata::table)
          .filter(schema::url_metadata::dsl::url.eq(&url))
          .execute(conn)
          .expect("Could not delete from url_metadata");
        None
      }
    } else {
      old_metadata.to_owned()
    }
  } else {
    old_metadata.to_owned()
  })
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

  match schema::url_metadata::table
    .filter(schema::url_metadata::url.eq(&url))
    .get_result::<models::UrlMetadata>(conn)
  {
    Ok(metadata) => {
      format!("{}{}", url, {
        annotation_from_metadata(Some(metadata))
          .map(|s| format!("\n{}", s))
          .unwrap_or("".to_owned())
      })
    }
    Err(err) => match err {
      diesel::result::Error::NotFound => url,
      _ => {
        log::error!("{}", err);
        url
      }
    },
  }
}

fn learn_url(url: &str, post: &RecvPost, conn: &PgConnection) -> String {
  let mut url = url.to_owned();
  if !url.starts_with("http") {
    url = "https://".to_owned() + &url;
  }

  if let Some(url_obj) = Url::parse(&url).ok() {
    // inserts url into table `sing`
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
        let metadata = insert_or_update_metadata(&url_obj, post, conn)
          .ok()
          .flatten();

        format!("Ich kann jetzt was Neues singen:{}", {
          match annotation_from_metadata(metadata) {
            Some(annotation) => format!("\n{}", &annotation),
            None => format!(" {}", url),
          }
        })
      }
      Err(err) => match err {
        diesel::result::Error::DatabaseError(e, _) => match e {
          DatabaseErrorKind::UniqueViolation => "kenn ich schon".to_owned(),
          _ => "Datenbank-Fehler. Tritt Franz und probier es später nochmal.".to_owned(),
        },
        _ => "diesel-Fehler. Tritt Franz und probier es später nochmal.".to_owned(),
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
    _ => format!("Ich habe was vergessen: {}", url),
  }
}

fn sing_count(conn: &PgConnection) -> Result<usize> {
  Ok(
    schema::sing::table
      .select(diesel::dsl::count(schema::sing::dsl::url))
      .execute(conn)?,
  )
}

pub fn sing(mode: Option<SingMode>, post: &RecvPost, conn: &PgConnection) -> String {
  let m = mode.unwrap_or(SingMode::Sing);
  match m {
    SingMode::Sing => sing_url(post, conn),

    SingMode::Learn { url } => learn_url(&url, post, conn),
    SingMode::Remove { url } => forget_url(&url, conn),

    SingMode::Replace { oldurl, newurl } => {
      format!(
        "{}\n{}",
        forget_url(&oldurl, conn),
        learn_url(&newurl, post, conn)
      )
    }
    SingMode::Count => match sing_count(conn) {
      Ok(count) => format!("Ich kann schon {} Lieder singen.", count),
      Err(err) => format!("Datenbank-Fehler: {}", err),
    },
  }
}
