use anyhow::Context;
use clap::Parser;
use diesel::*;
use parking_lot::Mutex;

use crate::{bots::*, models, schema};

pub fn nickname(conn: &Mutex<PgConnection>) -> impl Bot + '_ {
  #[derive(Parser)]
  struct Opt {
    name: Option<String>,
    #[clap(subcommand)]
    mode: Option<NicknameMode>,
  }
  #[derive(Parser, Clone, Debug)]
  enum NicknameMode {
    Show,
    #[clap(aliases = &["a", "-a", "--add"])]
    Add {
      nickname_to_add: String,
    },
    #[clap(aliases = &["r", "-r", "rm", "--remove", "del", "--delete"])]
    Remove {
      nickname_to_remove: String,
    },
  }

  let _botname = "nickname";
  clap_bot(_botname, _botname, move |opt: Opt, post| {
    let c = conn.lock();
    let cc = c.deref();

    let nicknames = |userid: i32| -> Result<Vec<models::Nickname>, diesel::result::Error> {
      schema::nickname__::table
        .filter(schema::nickname__::dsl::userid.eq(&userid))
        .load::<models::Nickname>(cc)
    };

    let userid_thispost = || -> Result<Option<i32>, diesel::result::Error> {
      if let Some(user_id) = post.user_id {
        Ok(Some(user_id))
      } else {
        userid(Some(post.post.name.to_owned()), cc)
      }
    };
    let nickname_thispost = || -> String {
      if let Some(username) = post.username.as_ref().cloned() {
        username
      } else {
        post.post.name.clone()
      }
    };

    let nickname_show = || -> Result<String> {
      let (userid, nickname) = {
        if let Some(n) = opt.name.as_ref() {
          let n = n.to_owned();
          (
            userid(Some(n.clone()), cc).context(format!("Ich kenne {} nicht.", n))?,
            n.clone(),
          )
        } else {
          (
            userid_thispost().context(format!("Ich kenne {} nicht.", &post.post.name))?,
            nickname_thispost(),
          )
        }
      };

      Ok(if let Some(userid) = userid {
        let mut nicknames_list = nicknames(userid)?
          .iter()
          .map(|nick| nick.nickname.to_owned())
          .collect::<Vec<String>>();
        nicknames_list.sort_unstable();
        match nicknames_list.len() {
          0 => format!("{} hat keine nicknames.", nickname),
          _ => format!(
            "{} hat die Nicknames\n{}",
            nickname,
            nicknames_list.join("\n")
          ),
        }
      } else {
        format!("Ich kenne {} nicht.", nickname)
      })
    };

    fn add_nickname(
      post: RecvPost,
      name: Option<String>,
      nickname_to_add: String,
      conn: &PgConnection,
    ) -> Result<Option<String>, diesel::result::Error> {
      let mut name = name;
      if let None = name {
        name = post.username;
      }

      if let Some(name) = name {
        let userid = userid(Some(name.clone()), conn)?;

        if let Some(userid) = userid {
          diesel::insert_into(schema::nickname__::table)
            .values((
              schema::nickname__::dsl::userid.eq(userid),
              schema::nickname__::dsl::nickname.eq(nickname_to_add.clone()),
            ))
            .execute(conn)?;

          Ok(Some(format!(
            "{} hat jetzt den nickname {}",
            name, nickname_to_add
          )))
        } else {
          Ok(Some(format!("Ich kenne {} nicht.", name)))
        }
      } else {
        Ok(None)
      }
    }
    fn remove_nickname(
      name: Option<String>,
      nickname: String,
      conn: &PgConnection,
    ) -> Result<Option<String>> {
      if let Some(_) = userid(Some(nickname.clone()), conn)? {
        let diesel = diesel::delete(
          schema::nickname__::table.filter(schema::nickname__::dsl::nickname.eq(nickname.clone())),
        )
        .execute(conn);
        match &diesel {
          Err(diesel::result::Error::NotFound) => {
            return Ok(Some(format!("Ich kenne {} nicht", nickname)));
          }
          _ => {
            diesel?;
          }
        }

        Ok(Some({
          if let Some(name) = name {
            format!("{} hat jetzt den nickname {} nicht mehr.", name, nickname)
          } else {
            format!("Ich habe jetzt den nickname {} vergessen.", nickname)
          }
        }))
      } else {
        Ok(Some(format!("Ich kenne {} nicht.", nickname)))
      }
    }

    let mode = opt.mode.as_ref().cloned().unwrap_or(NicknameMode::Show);
    Ok(match mode {
      NicknameMode::Show => Some(nickname_show().unwrap_or_else(|s| s.to_string())),
      NicknameMode::Add { nickname_to_add } => {
        add_nickname(post.to_owned(), opt.name, nickname_to_add, cc).unwrap()
      }
      NicknameMode::Remove { nickname_to_remove } => {
        remove_nickname(opt.name, nickname_to_remove, cc).unwrap_or_else(|s| Some(s.to_string()))
      }
    })
  })
}

pub fn username(userid: Option<i32>, conn: &PgConnection) -> Result<Option<String>> {
  if let Some(userid) = userid {
    let diesel_result = schema::qedmitglied::table
      .filter(schema::qedmitglied::dsl::userid.eq(userid))
      .load::<models::Qedmitglied>(conn);

    match diesel_result {
      Err(diesel::result::Error::NotFound) => Ok(None),
      _ => {
        let inner = diesel_result?;
        Ok(Some(inner[0].username.clone()))
      }
    }
  } else {
    Ok(None)
  }
}
pub fn userid(
  nickname: Option<String>,
  conn: &PgConnection,
) -> Result<Option<i32>, diesel::result::Error> {
  if let Some(nickname) = nickname {
    let diesel_result = diesel::dsl::sql_query(
          "WITH real_and_nicknames (userid, nickname) AS (SELECT * FROM nickname__ UNION ALL SELECT * FROM qedmitglied) SELECT * FROM real_and_nicknames WHERE LOWER(nickname) = LOWER($1);",
        )
        .bind::<diesel::sql_types::Text, _>(nickname)
        .get_result::<models::Nickname>(conn)
          .map(|nickname| Some(nickname.userid));

    match diesel_result {
      Err(diesel::result::Error::NotFound) => Ok(None),
      _ => {
        let diesel_inner = diesel_result?;
        Ok(diesel_inner)
      }
    }
  } else {
    Ok(None)
  }
}

pub fn userid_and_nickname(
  nickname: Option<String>,
  conn: &PgConnection,
) -> Result<Option<(i32, String)>> {
  if let Some(nickname) = nickname {
    if let Some(userid) = userid(Some(nickname), conn)? {
      /* sql guarantees us that nickname.userid references qedmitglied.userid
       * so we will unwrap()
       */
      Ok(Some((userid, username(Some(userid), conn)?.unwrap())))
    } else {
      Ok(None)
    }
  } else {
    Ok(None)
  }
}

pub fn userid_from_recv_post(post: &RecvPost, conn: &PgConnection) -> Result<Option<i32>> {
  match post.user_id {
    Some(user_id) => Ok(Some(user_id)),
    None => match userid(Some(post.post.name.clone()), conn)? {
      Some(user_id) => Ok(Some(user_id)),
      None => Ok(None),
    },
  }
}
