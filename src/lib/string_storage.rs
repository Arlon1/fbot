use std::time::Instant;

pub struct StringStorage {
  s: String,
  instant: Instant,
}
impl StringStorage {
  pub fn new(s: String) -> Self {
    Self {
      s,
      instant: Instant::now(),
    }
  }

  pub fn get_s(&self) -> String {
    self.s.clone()
  }
  pub fn set_s(&mut self, s: String) {
    self.s = s;
    self.instant = Instant::now();
  }

  pub fn instant(&self) -> Instant {
    self.instant
  }
}
