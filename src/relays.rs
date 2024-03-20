use std::env::consts;
use std::fmt::{self, Debug, Display};
use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use thiserror::Error;

use crate::coord::Coord;
use crate::filters::Filter;

/// Macro helper to simplify accessing JSON fields and casting them.
macro_rules! get {
  ($data:expr, $field:expr, $method:ident) => {
    $data[$field]
      .$method()
      .ok_or_else(|| RelaysError::ParseFieldFailed(stringify!($field).into()))?
  };
}

/// Macro to simplify accessing values in a JSON object (map).
macro_rules! value {
  ($data:expr, $field:expr) => {
    $data
      .get($field)
      .ok_or_else(|| RelaysError::ParseFieldFailed($field.into()))?
  };
}

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

  #[error("Could not load relays from the Mullvad API")]
  LoadRelaysFailed(reqwest::Error),

  #[error("Failed to parse the response")]
  ParseResponseFailed(reqwest::Error),
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
  path: Option<PathBuf>,
  /// Configuration/additional data needed for loading.
  config: RelaysLoaderConfig,
  /// Filters to apply to the loaded relays.
  filters: Vec<Box<dyn Filter<Item = Relay>>>,
}

impl RelaysLoader {
  pub fn new(config: RelaysLoaderConfig, filters: Vec<Box<dyn Filter<Item = Relay>>>) -> Self {
    let path = Self::resolve_path();

    Self {
      path,
      config,
      filters,
    }
  }

  /// Returns the path to the relay file.
  pub fn resolve_path() -> Option<PathBuf> {
    let path = match consts::OS {
      // NOTE: On Ubuntu and likely some other distros this is wrong.
      | "linux" => Some("/var/cache/mullvad-vpn/relays.json"),
      | "macos" => Some("/Library/Caches/mullvad-vpn/relays.json"),
      | "windows" => Some("C:/ProgramData/Mullvad VPN/cache/relays.json"),
      | _ => None,
    };

    path.map(PathBuf::from)
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

  /// Loads the relays, either from local file or from the API.
  pub async fn load(&self) -> anyhow::Result<Vec<Relay>> {
    if matches!(&self.path, Some(path) if path.try_exists().unwrap_or(false)) {
      self.load_local()
    } else {
      self.load_remote().await
    }
  }

  /// Loads the relays from the local file.
  fn load_local(&self) -> anyhow::Result<Vec<Relay>> {
    let mut results = Vec::new();

    let path = match &self.path {
      | Some(path) => path,
      | None => return Ok(results),
    };

    // Read into a string.
    let data = fs::read_to_string(path).map_err(|source| {
      RelaysError::ReadFileFailed {
        path: path.to_owned(),
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
            if relay.is_active && self.filters.iter().all(|filter| filter.matches(&relay)) {
              results.push(relay);
            }
          }
        }
      }
    }

    Ok(results)
  }

  /// Gets the relays using the [Mullvad API][api].
  ///
  /// [api]: https://api.mullvad.net/app/documentation/#/paths/~1v1~1relays/get
  async fn load_remote(&self) -> anyhow::Result<Vec<Relay>> {
    let mut results = Vec::new();

    let response = reqwest::get("https://api.mullvad.net/app/v1/relays")
      .await
      .map_err(RelaysError::LoadRelaysFailed)?;

    let data = response
      .json::<Value>()
      .await
      .map_err(RelaysError::ParseResponseFailed)?;

    let locations = get!(data, "locations", as_object);

    let openvpn = get!(data, "openvpn", as_object);
    let wireguard = get!(data, "wireguard", as_object);

    for (protocol, relays) in [
      (Protocol::OpenVPN, get!(openvpn, "relays", as_array)),
      (Protocol::WireGuard, get!(wireguard, "relays", as_array)),
    ] {
      for relay in relays {
        let location_code = get!(relay, "location", as_str).to_string();
        let location = value!(locations, &location_code);

        let coord = Coord::new(
          get!(location, "latitude", as_f64),
          get!(location, "longitude", as_f64),
        );

        let distance = self.config.location.distance_to(&coord);

        let relay = Relay {
          coord,
          protocol,
          distance,
          ip: get!(relay, "ipv4_addr_in", as_str).to_string(),
          city: get!(location, "city", as_str).to_string(),
          country: get!(location, "country", as_str).to_string(),
          is_active: get!(relay, "active", as_bool),
          is_mullvad_owned: get!(relay, "owned", as_bool),
        };

        // There's no reason to filter inactive relays.
        if relay.is_active && self.filters.iter().all(|filter| filter.matches(&relay)) {
          results.push(relay);
        }
      }
    }

    Ok(results)
  }
}
