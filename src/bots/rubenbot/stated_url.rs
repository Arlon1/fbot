use url::Url;

#[derive(Clone)]
pub struct StatedUrl {
  url: Url,
  modified: bool,
  extra_url: Vec<Url>,
}

impl StatedUrl {
  pub fn new(url: Url) -> Self {
    StatedUrl {
      url,
      modified: false,
      extra_url: vec![],
    }
  }
  pub fn set_url(&mut self, url: Url) {
    if self.url != url {
      self.url = url;
      self.modified = true;
    }
  }
  pub fn get_url(&self) -> Url {
    self.url.clone()
  }
  pub fn is_modified(&self) -> bool {
    self.modified
  }

  pub fn add_extra_url(&mut self, url: Option<Url>) {
    if let Some(url) = url {
      self.extra_url.push(url);
    }
  }
  pub fn get_extra_urls(&self) -> Vec<Url> {
    let mut list = self.extra_url.clone();
    list.sort();
    list.dedup();
    list
  }
}
