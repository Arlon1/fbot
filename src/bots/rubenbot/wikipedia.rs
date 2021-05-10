use regex::Regex;
use url::Url;

use super::{simple_enhancer, LinkEnhancer};

pub fn wikipedia_enhancer() -> impl LinkEnhancer {
  simple_enhancer(|(stated_url, extra_texts)| {
    let mut stated_url = stated_url.clone();
    let extra_texts = extra_texts.clone();

    if let Some(w) = from_url(stated_url.get_url()) {
      stated_url.set_url(w.to_url());
    }

    (stated_url, extra_texts)
  })
}

#[impl_enum::with_methods {
  pub fn to_url(&self) -> Url {
    match self {
      Self::Wikipedia(w) => {
    	w.to_url()
      },
      Self::Wikimedia(w) => {
    	w.to_url()
      },
    }
  }
}]
pub enum WpType {
  Wikipedia(Wikipedia),
  Wikimedia(Wikimedia),
}

trait ToUrl {
  fn to_url(&self) -> Url;
}

pub struct Wikipedia {
  lang: String,
  title: String,
  fragment: Option<String>,
}
pub struct Wikimedia {
  filename: String,
}

pub fn from_url(url: Url) -> Option<WpType> {
  let re_hostname =
    Regex::new(r"(?x)(?P<lang>[a-z]{2})\.(m\.)?(wikipedia)(\.org)\.?").expect("invalid regex");
  let re_path_regular = Regex::new(r"/wiki/(?P<title>.+)").expect("invalid regex");
  let re_media_fragment =
    Regex::new(r"/media/Datei(:|%3[Aa])(?P<filename>.+)").expect("invalid regex");

  if let Some(captures) = re_hostname.captures(url.host_str()?) {
    let lang = captures.name("lang")?.as_str().to_owned();
    let fragment = url.fragment().map(|str| str.to_owned());

    if let Some(captures) = re_path_regular.captures(url.path()) {
      let title = captures.name("title")?.as_str().to_owned();

      if let Some(w) = wikimedia_parse(re_media_fragment, fragment.clone()) {
        Some(WpType::Wikimedia(w))
      } else {
        Some(WpType::Wikipedia(Wikipedia {
          lang,
          title,
          fragment,
        }))
      }
    } else if url.path() == "/w/index.php" {
      let title_matches: Vec<(String, String)> = url
        .query_pairs()
        .into_owned()
        .filter(|pair| pair.0 == "title")
        .collect();
      let title = title_matches.get(0)?.to_owned().1;
      Some(WpType::Wikipedia(Wikipedia {
        lang,
        title,
        fragment,
      }))
    } else {
      None
    }
  } else {
    None
  }
}

fn wikimedia_parse(re: Regex, fragment: Option<String>) -> Option<Wikimedia> {
  let cap = re.captures(fragment.as_ref()?)?;
  let filename = cap.name("filename")?;
  Some(Wikimedia {
    filename: filename.as_str().to_owned(),
  })
}

impl ToUrl for Wikipedia {
  fn to_url(&self) -> Url {
    let mut url = Url::parse(&format!(
      "https://{lang}.wikipedia.org/wiki/{title}",
      lang = self.lang,
      title = self.title,
    ))
    .expect("parsing error");
    url.set_fragment(self.fragment.as_ref().map(|frag| &frag[..]));
    url
  }
}

impl ToUrl for Wikimedia {
  fn to_url(&self) -> Url {
    Url::parse(&format!(
      "https://commons.wikimedia.org/wiki/File:{filename}",
      filename = self.filename,
    ))
    .expect("parsing error")
  }
}
