use tokio::net::TcpStream;
use tokio::time::{self, Duration, Instant, MissedTickBehavior};

use crate::relays::Relay;

#[derive(Debug)]
pub struct RelayTimings {
  /// Relay.
  relay: Relay,
  /// Relay timings.
  timings: Vec<Duration>,
}

impl RelayTimings {
  pub fn new(relay: Relay, timings: Vec<Duration>) -> Self {
    Self { relay, timings }
  }

  pub fn relay(&self) -> &Relay {
    &self.relay
  }

  pub fn rtt(&self) -> Option<Duration> {
    match self.timings.len() {
      | 0 => None,
      | len => Some(self.timings.iter().sum::<Duration>() / len as u32),
    }
  }
}

#[derive(Debug)]
pub struct RelayPinger {
  /// Relay to ping.
  relay: Relay,
  /// How many times to ping the relay. Defaults to 4.
  count: usize,
  /// How long to wait before timing out a ping. Defaults to 750 ms.
  timeout: Duration,
  /// How long to wait between pings. Defaults to 1 second.
  interval: Duration,
}

impl RelayPinger {
  pub fn new(relay: Relay) -> Self {
    Self {
      relay,
      count: 4,
      timeout: Duration::from_millis(750),
      interval: Duration::from_millis(1_000),
    }
  }

  pub fn set_count(&mut self, count: usize) {
    self.count = count;
  }

  pub fn set_timeout(&mut self, timeout: Duration) {
    self.timeout = timeout;
  }

  pub fn set_interval(&mut self, interval: Duration) {
    self.interval = interval;
  }

  pub async fn execute(self) -> RelayTimings {
    // I'm not entirely sure about hardcoding port 80, but it seems to be open on servers I checked.
    let ping_addr = format!("{}:80", self.relay.ip);

    // Set up the interval...
    let mut interval = time::interval(self.interval);

    // ...and use a different behavior for missed ticks. I'm not really sure why, but this works
    // better than the default one.
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut timings = Vec::new();

    for _ in 1..=self.count {
      interval.tick().await;

      let start = Instant::now();
      let stream = TcpStream::connect(&ping_addr);

      match time::timeout(self.timeout, stream).await {
        | Ok(Ok(..)) => {
          let end = Instant::now();
          let elapsed = end.duration_since(start);

          timings.push(elapsed);
        },
        | Ok(Err(..)) => continue,
        | Err(..) => continue,
      }
    }

    RelayTimings::new(self.relay, timings)
  }
}
