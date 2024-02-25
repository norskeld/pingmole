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

#[derive(Debug)]
pub struct FilterByDistance {
  user: Coord,
  distance: f64,
}

impl FilterByDistance {
  pub fn new(user: Coord, distance: f64) -> Self {
    Self { user, distance }
  }
}

impl Filter for FilterByDistance {
  fn stage(&self) -> FilterStage {
    FilterStage::Load
  }

  fn matches(&self, relay: &Relay) -> bool {
    (relay.coord.distance_to(&self.user) / 1_000.0) < self.distance
  }
}

#[derive(Debug)]
pub struct FilterByProtocol(Option<Protocol>);

impl FilterByProtocol {
  pub fn new(protocol: Option<Protocol>) -> Self {
    Self(protocol)
  }
}

impl Filter for FilterByProtocol {
  fn stage(&self) -> FilterStage {
    FilterStage::Load
  }

  fn matches(&self, relay: &Relay) -> bool {
    self
      .0
      .as_ref()
      .map_or(true, |use_protocol| relay.protocol == *use_protocol)
  }
}
