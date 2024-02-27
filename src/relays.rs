use std::env::consts;
use std::fmt::{self, Debug, Display};
use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use thiserror::Error;

use crate::coord::Coord;
use crate::filters::{Filter, FilterStage};

#[derive(Debug, Error)]
pub enum RelaysError {
  #[error("Failed to read the relay file: {path}")]
  ReadFileFailed {
    path: PathBuf,
    source: std::io::Error,
  },
  #[error("Failed to parse the relay file")]
  ParseFileFailed(serde_json::Error),
  #[error("Failed to parse the field {0}: it's either missing or malformed")]
  ParseFieldFailed(String),
  #[error("Unsupported system: {0}")]
  UnsupportedSystem(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Protocol {
  OpenVPN,
  WireGuard,
}

impl Display for Protocol {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      | Protocol::OpenVPN => write!(f, "OpenVPN"),
      | Protocol::WireGuard => write!(f, "WireGuard"),
    }
  }
}

#[derive(Clone, Debug)]
pub struct Relay {
  pub ip: String,
  pub city: String,
  pub country: String,
  pub coord: Coord,
  pub protocol: Protocol,
  pub is_active: bool,
  pub is_mullvad_owned: bool,
  pub distance: f64,
}

#[derive(Debug)]
pub struct RelaysLoaderConfig {
  /// Current user location.
  pub location: Coord,
}

#[derive(Debug)]
pub struct RelaysLoader {
  /// Path to the relay file.
  path: PathBuf,
  /// Configuration/additional data needed for loading.
  config: RelaysLoaderConfig,
  /// Filters to apply to the loaded relays.
  filters: Vec<Box<dyn Filter<Item = Relay>>>,
}

impl RelaysLoader {
  pub fn new(
    path: PathBuf,
    config: RelaysLoaderConfig,
    filters: Vec<Box<dyn Filter<Item = Relay>>>,
  ) -> Self {
    Self {
      path,
      config,
      filters,
    }
  }

  /// Returns the path to the relay file.
  pub fn resolve_path() -> Result<PathBuf, RelaysError> {
    let path = match consts::OS {
      | "linux" => "/var/cache/mullvad-vpn/relays.json",
      | "macos" => "/Library/Caches/mullvad-vpn/relays.json",
      | "windows" => "C:/ProgramData/Mullvad VPN/cache/relays.json",
      | system => return Err(RelaysError::UnsupportedSystem(system.to_string())),
    };

    Ok(PathBuf::from(path))
  }

  /// Parses a protocol stored in the `endpoint_data` field of a relay, which can be either of the
  /// following:
  ///
  /// ```json
  /// "endpoint_data": "openvpn",
  /// "endpoint_data": "bridge",
  /// "endpoint_data": {
  ///   "wireguard": {
  ///     "public_key": "..."
  ///   }
  /// }
  /// ```
  ///
  /// We actually not interested in those with "bridge", so skip them with other ones.
  pub fn resolve_protocol(relay: &Value) -> Option<Protocol> {
    match &relay["endpoint_data"] {
      | Value::String(ref s) => s.eq("openvpn").then_some(Protocol::OpenVPN),
      | Value::Object(o) => o.get("wireguard").map(|_| Protocol::WireGuard),
      | _ => None,
    }
  }

  /// Loads the relays from the relay file.
  pub fn load(&self) -> Result<Vec<Relay>, RelaysError> {
    /// Simple macro helper to simplify accessing JSON fields and casting them.
    macro_rules! get {
      ($data:expr, $field:expr, $method:ident) => {
        $data[$field]
          .$method()
          .ok_or_else(|| RelaysError::ParseFieldFailed(stringify!($field).into()))?
      };
    }

    let mut locations = Vec::new();

    // Read into a string.
    let data = fs::read_to_string(&self.path).map_err(|source| {
      RelaysError::ReadFileFailed {
        path: self.path.clone(),
        source,
      }
    })?;

    // Parse the string as arbitrary JSON.
    let data = serde_json::from_str::<Value>(&data).map_err(RelaysError::ParseFileFailed)?;

    for country in get!(data, "countries", as_array) {
      for city in get!(country, "cities", as_array) {
        for relay in get!(city, "relays", as_array) {
          // We only need relays that have either "openvpn" or "wireguard" protocols.
          if let Some(protocol) = Self::resolve_protocol(relay) {
            let coord = Coord::new(
              get!(city, "latitude", as_f64),
              get!(city, "longitude", as_f64),
            );

            let distance = self.config.location.distance_to(&coord);

            let relay = Relay {
              coord,
              protocol,
              distance,
              ip: get!(relay, "ipv4_addr_in", as_str).to_string(),
              city: get!(city, "name", as_str).to_string(),
              country: get!(country, "name", as_str).to_string(),
              is_active: get!(relay, "active", as_bool),
              is_mullvad_owned: get!(relay, "owned", as_bool),
            };

            // There's no reason to filter inactive relays.
            if relay.is_active
              && self
                .filters
                .iter()
                .filter(|filter| filter.stage() == FilterStage::Load)
                .all(|filter| filter.matches(&relay))
            {
              locations.push(relay);
            }
          }
        }
      }
    }

    Ok(locations)
  }
}
