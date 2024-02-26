use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoordError {
  #[error("Failed to fetch coordinates")]
  FetchFailed(reqwest::Error),
  #[error("Failed to parse response")]
  ParseResponseFailed(reqwest::Error),
  #[error("Failed to get latitude and longitude from the response")]
  GetCoordsFailed,
}

/// Represents a point on Earth.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Coord {
  latitude: f64,
  longitude: f64,
}

impl Coord {
  /// Constructs a new `Coord`.
  pub fn new(latitude: f64, longitude: f64) -> Self {
    Self {
      latitude,
      longitude,
    }
  }

  /// Fetches the current coordinates using the Mullvad API.
  pub async fn fetch() -> Result<Self, CoordError> {
    let response = reqwest::get("https://am.i.mullvad.net/json")
      .await
      .map_err(CoordError::FetchFailed)?;

    let data = response
      .json::<Value>()
      .await
      .map_err(CoordError::ParseResponseFailed)?;

    let lat = data["latitude"].as_f64();
    let lon = data["longitude"].as_f64();

    lat
      .zip(lon)
      .map(|(latitude, longitude)| Self::new(latitude, longitude))
      .ok_or_else(|| CoordError::GetCoordsFailed)
  }

  /// Finds the distance (in kilometers) between two coordinates using the haversine formula.
  pub fn distance_to(&self, other: &Self) -> f64 {
    // Earth radius in meters. This is *average*, since Earth is not a sphere, but a spheroid.
    const R: f64 = 6_371_000f64;

    // Turn latitudes and longitudes into radians.
    let phi1 = self.latitude.to_radians();
    let phi2 = other.latitude.to_radians();
    let lam1 = self.longitude.to_radians();
    let lam2 = other.longitude.to_radians();

    // The haversine function. Computes half a versine of the given angle `theta`.
    let haversine = |theta: f64| (1.0 - theta.cos()) / 2.0;

    let hav_delta_phi = haversine(phi2 - phi1);
    let hav_delta_lam = phi1.cos() * phi2.cos() * haversine(lam2 - lam1);
    let hav_delta = hav_delta_phi + hav_delta_lam;

    // The distance in meters.
    let distance = (2.0 * R * hav_delta.sqrt().asin() * 1_000.0).round() / 1_000.0;

    distance / 1_000.0
  }
}
