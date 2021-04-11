use regex::Regex;
use url::Url;
use urlencoding::encode;

pub mod stated_url;
use stated_url::StatedUrl;

use crate::bots::*;

trait LinkEnhancer {
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

fn wikipedia_enhancer() -> impl LinkEnhancer {
  simple_enhancer(|stated_url| {
    let mut stated_url = stated_url.clone();
    let wp_re = Regex::new(
      r"(?x)
([a-z]{2}\.)
m\.
(wikipedia)
(\.org)",
    )
    .expect("invalid regex");
    let mut new_url = stated_url.get_url();
    if let Some(caps) = new_url
      .clone()
      .host_str()
      .and_then(|host| wp_re.captures(host))
    {
      let new_host = caps
        .iter()
        .skip(1)
        .flatten()
        .fold(String::new(), |a, c| a + c.as_str());
      new_url
        .set_host(Some(&new_host))
        .expect("error setting host");
    }
    stated_url.set_url(new_url);

    stated_url
  })
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
      ("wikipedia_enhancer", Box::new(wikipedia_enhancer())),
      ("qedchat_link_encode", Box::new(qedchat_link_encode())),
    ];

    let urls = post
      .post
      .message
      .split_whitespace()
      .map(|url_str| Url::parse(url_str))
      .flatten();

    let posts: Vec<Url> = urls
      .map(|url| StatedUrl::new(url))
      .filter(|su| vec!["http", "https"].contains(&su.get_url().to_owned().scheme()))
      .map(|su| enhancers.iter().fold(su, |su, e| e.1.enhance(&su)))
      .filter(|post| post.is_modified())
      .map(|statedurl| statedurl.get_url())
      .collect();

    Ok(if posts.is_empty() {
      None
    } else {
      let message =
        "better: ".to_owned() + &posts.into_iter().map(|url| url.to_string()).join("\n");
      Some(("Ruben".to_string(), message))
    })
  })
}
