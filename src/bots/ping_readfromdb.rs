use diesel::*;
use parking_lot::Mutex;

use super::nickname;
use crate::{
  bots::{util::*, *},
  models, schema,
};

pub fn ping_readfromdb(conn: &Mutex<PgConnection>) -> impl Bot + '_ {
  simple_bot(move |recv_post| {
    let cc = conn.lock();
    let conn = cc.deref();

    let mut ping_list = vec![];

    let meta_recv_post = match recv_post.user_id.zip(recv_post.username.as_ref().cloned()) {
      Some((user_id, username)) => Some((user_id, username)),
      None => nickname::userid_and_nickname(Some(recv_post.post.name.clone()), conn)?,
    };

    if let Some(meta_recv_post) = meta_recv_post {
      if let Some(pings) = delete(schema::ping::table)
        .filter(schema::ping::dsl::receiver.eq(meta_recv_post.1))
        .returning(schema::ping::all_columns)
        .load::<models::PingQuery>(conn)
        .catch_notfound()?
      {
        for ping in pings {
          ping_list.push(ping);
        }
      }
    }

    if let Some(pings) = diesel::delete(schema::ping::table)
      .filter(schema::ping::dsl::receiver.eq(recv_post.post.name.clone()))
      .returning(schema::ping::all_columns)
      .get_results(conn)
      .catch_notfound()?
    {
      for ping in pings {
        ping_list.push(ping);
      }
    }

    Ok(if ping_list.is_empty() {
      None
    } else {
      Some(("Navi".to_owned(), ping_list.iter().join("\n")))
    })
  })
}
