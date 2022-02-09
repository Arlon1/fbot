//use clap::Parser;
use diesel::*;
use parking_lot::Mutex;

use super::nickname;
use crate::{
  bots::*,
  // bots::util::*,
  models,
  schema,
};

pub fn freiepunkte(conn: &Mutex<PgConnection>) -> impl Bot + '_ {
  #[derive(Clone, Debug, clap::ArgEnum)]
  enum FreiepunkteMode {
    Show,
    Add,
    Remove,
  }

  #[derive(Parser, Clone, Debug)]
  struct Opt {
    name_with_hash: String,
    nickname: Option<String>,
    #[clap(arg_enum)] // default_value = "show"
    mode: Option<FreiepunkteMode>,
    #[clap(default_value = "1")]
    delta: i64,
  }

  clap_bot_notrigger("freiepunkte", move |opt: Opt, recv_post| {
    dbg!(&opt);
    let cc = conn.lock();
    let conn = cc.deref();

    let userid = nickname::userid_from_recv_post(recv_post, conn)?;
    if let None = userid {
      return Ok(None);
    }

    //let nickname = opt.nickname.unwrap_or();

    Ok(if !opt.name_with_hash.starts_with("#") {
      {
        None
      }
    } else {
      dbg!("freiepunkte running\n{:#?}", &opt);

      let punktname = Some(opt.name_with_hash); //.strip_prefix("#");

      let mode = opt.mode.unwrap_or(FreiepunkteMode::Add);
      match mode {
        FreiepunkteMode::Show => {
          let punktestand = punktestand(
            punktid(punktname, conn)?,
            nickname::userid(opt.nickname, conn)?,
            conn,
          )?;

          Some("freiepunkte running show".to_owned())
        }
        _ => Some("freiepunkte running".to_owned()),
      }
    })
  })
}

fn punktestand(
  punktid: Option<i32>,
  userid: Option<i32>,
  conn: &PgConnection,
) -> Result<Option<i32>> {
  if let Some((punktid, _)) = punktid.zip(userid) {
    //match schema::freiepunkte_values::table
    //  .filter(schema::freiepunkte_values::dsl::id.eq(punktid))
    //  .filter(schema::freiepunkte_values::dsl::userid.eq(userid))
    //  .load::<models::FreiePunkteValues>(conn)?
    //  .first()
    //{
    //  Some(freiepunkte_value) => Ok(Some(freiepunkte_value.wert)),
    //  None => Ok(None),
    //}
    Ok(Some(0)) // todo
  } else {
    Ok(None)
  }
}

fn punktid(punktname: Option<String>, conn: &PgConnection) -> Result<Option<i32>> {
  if let Some(punktname) = punktname {
    Ok(Some(
      schema::freiepunkte::table
        .filter(schema::freiepunkte::dsl::name.eq(punktname))
        .load::<models::FreiePunkte>(conn)?[0]
        .id,
    ))
  } else {
    Ok(None)
  }
}
