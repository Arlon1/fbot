#[derive(Debug, thiserror::Error)]
#[error("{display_error}")]
pub struct DualError {
  display_error: String,
  underlying: String,
}
impl DualError {
  pub fn new(display_error: String, underlying: String) -> Self {
    Self {
      display_error,
      underlying,
    }
  }
  pub fn underlying(&self) -> &str {
    &self.underlying
  }
}
