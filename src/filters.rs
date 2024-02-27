use std::fmt::Debug;
use std::time::Duration;

use crate::pinger::RelayTimed;
use crate::relays::{Protocol, Relay};

/// Filter trait to dynamically dispatch filters.
pub trait Filter: Debug {
  type Item;

  /// Filter predicate.
  fn matches(&self, item: &Self::Item) -> bool;
}

/// Filter by distance. The distance is in kilometers.
#[derive(Debug)]
pub struct FilterByDistance {
  /// Distance in kilometers.
  distance: f64,
}

impl FilterByDistance {
  pub fn new(distance: f64) -> Self {
    Self { distance }
  }
}

impl Filter for FilterByDistance {
  type Item = Relay;

  fn matches(&self, relay: &Self::Item) -> bool {
    relay.distance < self.distance
  }
}

/// Filter by protocol.
#[derive(Debug)]
pub struct FilterByProtocol {
  /// Protocol to compare with. `None` means any protocol.
  protocol: Option<Protocol>,
}

impl FilterByProtocol {
  pub fn new(protocol: Option<Protocol>) -> Self {
    Self { protocol }
  }
}

impl Filter for FilterByProtocol {
  type Item = Relay;

  fn matches(&self, relay: &Self::Item) -> bool {
    self
      .protocol
      .as_ref()
      .map_or(true, |protocol| relay.protocol == *protocol)
  }
}

/// Filter by Round-Trip Time.
#[derive(Debug)]
pub struct FilterByRTT {
  /// RTT value to compare with. `None` means any RTT.
  rtt: Option<Duration>,
}

impl FilterByRTT {
  pub fn new(rtt: Option<Duration>) -> Self {
    Self { rtt }
  }
}

impl Filter for FilterByRTT {
  type Item = RelayTimed;

  fn matches(&self, timings: &Self::Item) -> bool {
    // If `rtt` is `None`, then it means any RTT, so we then default to `true`.
    self.rtt.map_or(true, |filter_rtt| {
      // Otherwise, we compare the measured RTT with the filter RTT, but here we default to `false`.
      timings
        .rtt_mean()
        .map_or(false, |relay_rtt| relay_rtt <= filter_rtt)
    })
  }
}
