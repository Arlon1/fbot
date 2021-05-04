use std::{
  thread::sleep,
  time::{Duration, Instant},
};

pub struct InstantWaiter {
  instant: Instant,
  cooldown_period: Duration,
}
impl InstantWaiter {
  pub fn new(duration: Duration) -> Self {
    Self {
      instant: Instant::now(),
      cooldown_period: duration,
    }
  }
  pub fn wait_for_permission(&mut self) {
    let elapsed = self.instant.elapsed();
    if elapsed < self.cooldown_period {
      sleep(self.cooldown_period - elapsed);
    }
    self.instant = Instant::now();
  }
}
