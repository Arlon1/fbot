use regex::Regex;
use url::Url;

use super::{simple_enhancer, LinkEnhancer};

pub fn wikipedia_enhancer() -> impl LinkEnhancer {
  simple_enhancer(|(stated_url, extra_texts)| {
    let mut stated_url = stated_url.clone();
    let /* mut */ extra_texts = extra_texts.clone();

    if let Some(w) = Wikipedia::from_url(stated_url.get_url()) {
      stated_url.set_url(w.to_url());
      //if let Some(url_translated) = w.query_translation_en() {
      //  extra_texts.push(url_translated.to_string());
      //}
    }

    (stated_url, extra_texts)
  })
}

struct Wikipedia {
  lang: String,
  title: String,
  fragment: Option<String>,
}
impl Wikipedia {
  pub fn from_url(url: Url) -> Option<Self> {
    let re_hostname =
      Regex::new(r"(?x)(?P<lang>[a-z]{2})\.(m\.)?(wikipedia)(\.org)\.?").expect("invalid regex");
    let re_path_regular = Regex::new(r"/wiki/(?P<title>.+)").expect("invalid regex");

    if let Some(captures) = re_hostname.captures(url.host_str()?) {
      let lang = captures.name("lang")?.as_str().to_owned();
      let fragment = url.fragment().map(|frag| frag.to_owned());

      if let Some(captures) = re_path_regular.captures(url.path()) {
        let title = captures.name("title")?.as_str().to_owned();
        Some(Self {
          lang,
          title,
          fragment,
        })
      } else if url.path() == "/w/index.php" {
        let title_matches: Vec<(String, String)> = url
          .query_pairs()
          .into_owned()
          .filter(|pair| pair.0 == "title")
          .collect();
        let title = title_matches.get(0)?.to_owned().1;
        Some(Self {
          lang,
          title,
          fragment,
        })
      } else {
        None
      }
    } else {
      None
    }
  }
  pub fn to_url(&self) -> Url {
    let mut url = Url::parse(&format!(
      "https://{lang}.wikipedia.org/wiki/{title}",
      lang = self.lang,
      title = self.title,
    ))
    .expect("parsing error");
    url.set_fragment(self.fragment.as_ref().map(|frag| &frag[..]));
    url
  }

  pub fn query_translation_en(&self) -> Option<Url> {
    let query_url = format!("https://www.wikidata.org/w/api.php?action=wbgetentities&sites={lang}wiki&titles={titles}&normalize=1&props=sitelinks/urls&sitefilter=enwiki&format=json", titles=self.title, lang=self.lang);
    let text = reqwest::blocking::get(query_url).ok()?.text().ok()?;
    let obj: serde_json::value::Value = serde_json::from_str(&text).ok()?;
    let name_en = obj
      .get("entities")?
      .as_object()?
      .iter()
      .next()?
      .1
      .get("sitelinks")?
      .get("enwiki")?
      .get("url")?
      .as_str()?;
    let url = Url::parse(name_en);
    url.ok()
  }
}