use clap::Parser;
use diesel::*;
use parking_lot::Mutex;

use super::nickname;
use crate::{bots::*, models, schema};

pub fn ping_sendtodb(conn: &Mutex<PgConnection>) -> impl Bot + '_ {
  #[derive(Clone, Debug, Parser)]
  struct Opt {
    receiver: String,
    //schedule: Date
    message: Vec<String>,
  }
  clap_bot("ping", "__", move |opt: Opt, post| {
    let cc = conn.lock();
    let conn = cc.deref();

    if opt.message.len() > 0 {
      let sender = nickname::userid(Some(post.post.name.clone()), conn)?;
      let receiver = match nickname::userid(Some(opt.receiver.clone()), conn)? {
        Some(userid) => nickname::username(Some(userid), conn)?.unwrap_or(opt.receiver),
        None => opt.receiver,
      };

      diesel::insert_into(schema::ping::table)
        .values(models::PingInsert {
          sender,
          receiver,
          sent: post.date.naive_utc(),
          scheduled: None,
          message: opt.message.join(" "),
        })
        .execute(conn)?;
    }
    Ok(None)
  })
}
