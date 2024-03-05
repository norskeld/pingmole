use std::sync::Arc;

use thiserror::Error;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration, Instant, MissedTickBehavior};

use crate::filters::Filter;
use crate::relays::Relay;

#[derive(Debug, Error)]
pub enum RelaysPingerError {
  #[error("Failed to await a task")]
  PingerAwaitFailed,
}

#[derive(Debug)]
pub struct RelayPingerConfig {
  /// How many times to ping the relay. Defaults to 4.
  count: usize,
  /// How long to wait before timing out a ping. Defaults to 750 ms.
  timeout: Duration,
  /// How long to wait between pings. Defaults to 1 second.
  interval: Duration,
}

impl RelayPingerConfig {
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the number of pings to send.
  pub fn set_count(mut self, count: usize) -> Self {
    self.count = count;
    self
  }

  /// Set the timeout for each ping.
  pub fn set_timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }

  /// Set the interval between pings.
  pub fn set_interval(mut self, interval: Duration) -> Self {
    self.interval = interval;
    self
  }
}

impl Default for RelayPingerConfig {
  fn default() -> Self {
    Self {
      count: 4,
      timeout: Duration::from_millis(750),
      interval: Duration::from_millis(1_000),
    }
  }
}

#[derive(Debug)]
pub struct RelayTimed {
  /// Relay.
  relay: Relay,
  /// Relay timings.
  timings: Vec<Duration>,
}

impl RelayTimed {
  pub fn new(relay: Relay, timings: Vec<Duration>) -> Self {
    Self { relay, timings }
  }

  /// Returns the relay.
  pub fn relay(&self) -> &Relay {
    &self.relay
  }

  /// Gets the mean RTT.
  pub fn rtt_mean(&self) -> Option<Duration> {
    match self.timings.len() {
      | 0 => None,
      | len => Some(self.timings.iter().sum::<Duration>() / len as u32),
    }
  }

  /// Gets the median RTT.
  pub fn rtt_median(&self) -> Option<Duration> {
    match self.timings.len() {
      | 0 => None,
      | len => {
        let mut timings = self.timings.clone();
        timings.sort();

        let middle = len / 2;

        if len % 2 == 0 {
          Some((timings[middle - 1] + timings[middle]) / 2)
        } else {
          Some(timings[middle])
        }
      },
    }
  }
}

#[derive(Debug)]
pub struct RelayPinger {
  /// Relay to ping.
  relay: Relay,
  /// Relay pinger config.
  config: Arc<RelayPingerConfig>,
}

impl RelayPinger {
  pub fn new(relay: Relay, config: Arc<RelayPingerConfig>) -> Self {
    Self { relay, config }
  }

  /// Execute the pinger.
  pub async fn execute(self) -> RelayTimed {
    // I'm not entirely sure about hardcoding port 80, but it seems to be open on servers I checked.
    let ping_addr = format!("{}:80", self.relay.ip);

    // Set up the interval...
    let mut interval = time::interval(self.config.interval);

    // ...and use a different behavior for missed ticks. I'm not really sure why, but this works
    // better than the default one.
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let mut timings = Vec::new();

    for _ in 1..=self.config.count {
      interval.tick().await;

      let start = Instant::now();
      let stream = TcpStream::connect(&ping_addr);

      match time::timeout(self.config.timeout, stream).await {
        | Ok(Ok(..)) => {
          let end = Instant::now();
          let elapsed = end.duration_since(start);

          timings.push(elapsed);
        },
        | Ok(Err(..)) => continue,
        | Err(..) => continue,
      }
    }

    RelayTimed::new(self.relay, timings)
  }
}

#[derive(Debug)]
pub struct RelaysPinger {
  /// Relay pinger tasks to await.
  tasks: Vec<JoinHandle<RelayTimed>>,
  /// Filters to apply to timed relays after pinging.
  filters: Vec<Box<dyn Filter<Item = RelayTimed>>>,
}

impl RelaysPinger {
  pub fn new(
    relays: Vec<Relay>,
    config: Arc<RelayPingerConfig>,
    filters: Vec<Box<dyn Filter<Item = RelayTimed>>>,
  ) -> Self {
    let tasks = relays
      .into_iter()
      .map(|relay| {
        let pinger = RelayPinger::new(relay, Arc::clone(&config));

        tokio::spawn(pinger.execute())
      })
      .collect();

    Self { tasks, filters }
  }

  /// Execute all pings.
  pub async fn ping(self) -> Result<Vec<RelayTimed>, RelaysPingerError> {
    let mut results = Vec::new();

    for task in self.tasks {
      let timings = task
        .await
        .map_err(|_| RelaysPingerError::PingerAwaitFailed)?;

      if self.filters.iter().all(|filter| filter.matches(&timings)) {
        results.push(timings);
      }
    }

    Ok(results)
  }
}
