use url::Url;

#[derive(Clone)]
pub struct StatedUrl {
  url: Url,
  modified: bool,
}

impl StatedUrl {
  pub fn new(url: Url) -> Self {
    StatedUrl {
      url,
      modified: false,
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
}

impl std::fmt::Debug for StatedUrl {
  fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    formatter
      .debug_struct("StatedUrl")
      .field("Url", &self.url)
      .field("modified", &self.modified)
      .finish()
  }
}
