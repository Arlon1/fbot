use chrono::Duration;
use std::thread::sleep;
use std::time::Instant;

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
  pub fn wait_for_permission(&mut self) -> () {
    let elapsed = self.instant.elapsed();
    if elapsed.as_secs() < self.cooldown_period.num_seconds() as u64 {
      sleep(std::time::Duration::new(1, 0));
    }
    elapsed.checked_add(elapsed);
  }
}
