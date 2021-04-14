use url::Url;
use urlencoding::encode;

pub mod stated_url;
use stated_url::StatedUrl;
mod wikipedia;

use crate::bots::*;

pub trait LinkEnhancer {
  fn enhance(&self, stated_url: &StatedUrl) -> StatedUrl;
}
fn simple_enhancer(f: impl Fn(&StatedUrl) -> StatedUrl) -> impl LinkEnhancer {
  struct SimpleEnhancer<A>(A);
  impl<A: Fn(&StatedUrl) -> StatedUrl> LinkEnhancer for SimpleEnhancer<A> {
    fn enhance(&self, stated_url: &StatedUrl) -> StatedUrl {
      (self.0)(stated_url)
    }
  }
  SimpleEnhancer(f)
}

fn qedchat_link_encode() -> impl LinkEnhancer {
  simple_enhancer(|stated_url| {
    let mut stated_url = stated_url.clone();

    let chars = ['(', ')'];

    let mut url = stated_url.get_url();
    let mut path_str = stated_url.get_url().path().to_owned();
    for c in chars.iter() {
      path_str = path_str.replace(c.to_owned(), &encode(&c.to_string()).to_owned());
    }
    url.set_path(&path_str);
    stated_url.set_url(url);

    stated_url
  })
}

pub fn rubenbot() -> impl Bot {
  simple_bot(move |post| {
    let enhancers: Vec<(_, Box<dyn LinkEnhancer + Send + Sync>)> = vec![
      (
        "wikipedia_enhancer",
        Box::new(wikipedia::wikipedia_enhancer()),
      ),
      ("qedchat_link_encode", Box::new(qedchat_link_encode())),
    ];

    let urls = post
      .post
      .message
      .split_whitespace()
      .map(|url_str| Url::parse(url_str))
      .flatten();

    let posts = urls
      .map(|url| StatedUrl::new(url))
      .filter(|su| vec!["http", "https"].contains(&su.get_url().to_owned().scheme()))
      .map(|su| enhancers.iter().fold(su, |su, e| e.1.enhance(&su)))
      .filter(|post| post.is_modified())
      .map(|stated_url| (stated_url.get_url(), stated_url.get_extra_urls()))
      .collect::<Vec<(Url, Vec<Url>)>>();

    let enhanced_urls = posts
      .clone()
      .iter()
      .map(|pair| pair.clone().0.to_string())
      .collect::<Vec<String>>();
    let extra_urls = posts
      .iter()
      .map(|pair| pair.clone().1)
      .flatten()
      .map(|url| url.to_string())
      .collect::<Vec<String>>();

    Ok(if enhanced_urls.is_empty() && extra_urls.is_empty() {
      None
    } else {
      let mut message = vec![];
      if !enhanced_urls.is_empty() {
        message.push("better: ".to_owned() + &enhanced_urls.join("\n"));
      }
      if !extra_urls.is_empty() {
        //message.push("even better: ".to_owned() + &extra_urls.join("\n"));
      }
      Some(("Ruben".to_string(), message.join("\n")))
    })
  })
}
