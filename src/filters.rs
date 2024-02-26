use std::fmt::Debug;

use crate::coord::Coord;
use crate::relays::{Protocol, Relay};

#[derive(PartialEq)]
pub enum FilterStage {
  /// Such filters apply when loading them from the relays file.
  Load,
  /// Such filters apply after pinging relays.
  Ping,
}

/// Filter trait to dynamically dispatch filters.
pub trait Filter: Debug {
  /// Returns the stage of the filter.
  fn stage(&self) -> FilterStage;

  /// Filter predicate.
  fn matches(&self, relay: &Relay) -> bool;
}

/// Filter by distance. The distance is in kilometers.
#[derive(Debug)]
pub struct FilterByDistance {
  coord: Coord,
  distance: f64,
}

impl FilterByDistance {
  pub fn new(coord: Coord, distance: f64) -> Self {
    Self { coord, distance }
  }
}

impl Filter for FilterByDistance {
  fn stage(&self) -> FilterStage {
    FilterStage::Load
  }

  fn matches(&self, relay: &Relay) -> bool {
    relay.coord.distance_to(&self.coord) < self.distance
  }
}

/// Filter by protocol. `None` means any protocol.
#[derive(Debug)]
pub struct FilterByProtocol {
  protocol: Option<Protocol>,
}

impl FilterByProtocol {
  pub fn new(protocol: Option<Protocol>) -> Self {
    Self { protocol }
  }
}

impl Filter for FilterByProtocol {
  fn stage(&self) -> FilterStage {
    FilterStage::Load
  }

  fn matches(&self, relay: &Relay) -> bool {
    self
      .protocol
      .as_ref()
      .map_or(true, |protocol| relay.protocol == *protocol)
  }
}
