use diesel::*;
use parking_lot::Mutex;
use url::Url;

pub mod stated_url;
use stated_url::StatedUrl;
mod wikipedia;

use crate::{
  bots::{util::*, *},
  models, schema,
};
use wikipedia::wikipedia_enhancer;

pub trait LinkEnhancer {
  fn enhance(&self, arg: &(StatedUrl, Vec<String>)) -> Result<(StatedUrl, Vec<String>)>;
}
fn simple_enhancer(
  f: impl Fn(&(StatedUrl, Vec<String>)) -> Result<(StatedUrl, Vec<String>)>,
) -> impl LinkEnhancer {
  struct SimpleEnhancer<A>(A);
  impl<A: Fn(&(StatedUrl, Vec<String>)) -> Result<(StatedUrl, Vec<String>)>> LinkEnhancer
    for SimpleEnhancer<A>
  {
    fn enhance(&self, arg: &(StatedUrl, Vec<String>)) -> Result<(StatedUrl, Vec<String>)> {
      (self.0)(arg)
    }
  }
  SimpleEnhancer(f)
}

pub fn rubenbot(conn: &Mutex<PgConnection>) -> impl Bot + '_ {
  simple_bot(move |recv_post| {
    let enhancers: Vec<(_, Box<dyn LinkEnhancer + Send + Sync>)> = vec![
      ("wikipedia_enhancer", Box::new(wikipedia_enhancer())),
      //("qedchat_link_encode", Box::new(qedchat_link_encode())),
      ("youtube_enhancer", Box::new(youtube_link_enhancer(conn))),
      ("qedgallery", Box::new(qedgallery())),
    ];

    let send_posts = {
      if !recv_post.post.message.starts_with("!") {
        recv_post
          .post
          .message
          .split_whitespace()
          //.split("")
          .map(|url_str| {
            log::debug!("{}", &url_str);
            Url::parse(url_str)
          })
          .flatten()
          .map(|url| StatedUrl::new(url))
          .filter(|su| vec!["http", "https"].contains(&su.get_url().to_owned().scheme()))
          .map(|su| (su, vec![]))
          .map(|(su, et)| {
            enhancers.iter().fold((su, et), |(su, et), enhancer| {
              enhancer
                .1
                .enhance(&(su.clone(), et.clone()))
                .unwrap_or((su, et))
            })
          })
          .collect::<Vec<(StatedUrl, Vec<String>)>>()
      } else {
        vec![]
      }
    };

    let enhanced_urls = send_posts
      .clone()
      .iter()
      .filter(|(su, _)| su.is_modified())
      .map(|(su, _)| su.get_url().to_string())
      .dedup()
      .collect::<Vec<String>>();
    let extra_texts = send_posts
      .clone()
      .iter()
      .map(|(_, et)| et.to_owned())
      .flatten()
      .dedup()
      .join("\n");

    Ok({
      let mut message = vec![];
      if !enhanced_urls.is_empty() {
        message.push("better: ".to_owned() + &enhanced_urls.join("\n"));
      }
      if !extra_texts.is_empty() {
        message.push(extra_texts);
      }
      let message = message.join("\n");

      if !message.is_empty() {
        Some(("Ruben".to_string(), message))
      } else {
        None
      }
    })
  })
}

fn qedgallery() -> impl LinkEnhancer {
  simple_enhancer(|(stated_url, extra_texts)| {
    let mut stated_url = stated_url.clone();

    let mut url = stated_url.get_url();
    if url.host_str().unwrap_or("") == "qedgallery.qed-verein.de" && url.path() == "/image.php" {
      url.set_path("image_view.php");

      let pairs = &url
        .query_pairs()
        .filter(|(key, value)| !(key == "type" && value == "original"))
        .map(|(key, value)| key + "=" + value)
        .join("&");
      url.set_query(Some(pairs));

      stated_url.set_url(url);
    }

    Ok((stated_url, extra_texts.clone()))
  })
}

pub fn youtube_link_enhancer(conn: &Mutex<PgConnection>) -> impl LinkEnhancer + '_ {
  simple_enhancer(move |(stated_url, extra_texts)| {
    let cc = conn.lock();
    let conn = cc.deref();

    let mut stated_url = stated_url.clone();
    let mut extra_texts = extra_texts.clone();

    let s_url = &mut stated_url.get_url();

    match schema::url__::table
      .filter(schema::url__::dsl::url.eq(s_url.to_string()))
      .load::<models::Urls>(conn)
      .catch_notfound()?
    {
      Some(url_row) =>
      //
      //
      {
        Ok((stated_url, extra_texts))
      }
      None =>
      // read last_modified from database
      // if recent {
      //    read from database
      // } else {
      //   create new youtube object
      // }
      {
        Ok((stated_url, extra_texts))
      }
    }

    //if let Some(y) = crate::youtube::Youtube::from_url(s_url) {
    //  // filter playlist tags out
    //  //let query_filtered = s_url
    //  //  .query_pairs()
    //  //  .filter(|(key, _value)| !vec!["list", "index"].contains(&&key.to_owned().to_string()[..]))
    //  //  .map(|(key, value)| key + "=" + value)
    //  //  .join("&");
    //  //if s_url.query().unwrap_or("") != query_filtered {
    //  //  s_url.set_query(Some(&query_filtered));
    //  //}
    //
    //  let y_to_url = y.to_url();
    //  if y_to_url.to_string() != stated_url.get_url().to_string() {
    //    stated_url.set_url(y_to_url);
    //  }
    //  dbg!(&y);
    //
    //  if let Some(a) = y.annotation() {
    //    extra_texts.push(a);
    //  }
    //}
    //(stated_url, extra_texts)
  })
}
