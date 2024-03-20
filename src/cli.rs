use std::time::Duration;

use clap::builder::PossibleValue;
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};

use crate::relays::Protocol;
use crate::reporter::SortBy;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Filter servers by used protocol.
  #[arg(short, long, value_enum)]
  pub protocol: Option<Protocol>,

  /// Filter servers by maximum physical distance (in km).
  #[arg(short, long, default_value_t = 500)]
  pub distance: usize,

  /// Filter servers by maximum rtt (in ms).
  #[arg(short, long)]
  pub rtt: Option<u64>,

  /// Sort by specified field.
  #[arg(short, long, value_enum)]
  pub sort_by: Option<SortBy>,

  /// Set pings count to perform.
  #[arg(short, long, default_value_t = 8)]
  pub count: usize,

  /// Set ping timeout (in ms).
  #[arg(long, default_value_t = 750)]
  pub timeout: u64,

  /// Set ping interval (in ms).
  #[arg(long, default_value_t = 1000)]
  pub interval: u64,

  /// Set the latitude.
  #[arg(long = "lat", requires = "longitude")]
  pub latitude: Option<f64>,

  /// Set the longitude.
  #[arg(long = "lon", requires = "latitude")]
  pub longitude: Option<f64>,
}

impl ValueEnum for Protocol {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::OpenVPN, Self::WireGuard]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      | Protocol::OpenVPN => PossibleValue::new("openvpn"),
      | Protocol::WireGuard => PossibleValue::new("wireguard"),
    })
  }
}

impl ValueEnum for SortBy {
  fn value_variants<'a>() -> &'a [Self] {
    &[
      Self::Country,
      Self::City,
      Self::MedianRTT,
      Self::MeanRTT,
      Self::Distance,
    ]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      | SortBy::Country => PossibleValue::new("country"),
      | SortBy::City => PossibleValue::new("city"),
      | SortBy::MedianRTT => PossibleValue::new("rtt_median"),
      | SortBy::MeanRTT => PossibleValue::new("rtt_mean"),
      | SortBy::Distance => PossibleValue::new("distance"),
    })
  }
}

/// Small wrapper around the `indicatif` spinner.
pub struct Spinner {
  spinner: ProgressBar,
}

impl Spinner {
  pub fn new() -> Self {
    let style = ProgressStyle::default_spinner()
      .tick_strings(&["   ", "·  ", "·· ", "···", " ··", "  ·", "   "]);

    let spinner = ProgressBar::new_spinner();

    spinner.set_style(style);
    spinner.enable_steady_tick(Duration::from_millis(150));

    Self { spinner }
  }

  /// Sets the message of the spinner.
  pub fn set_message<S>(&self, message: S)
  where
    S: Into<String> + AsRef<str>,
  {
    self.spinner.set_message(message.into());
  }

  /// Stops the spinner and clears the message.
  pub fn stop(&self) {
    self.spinner.finish_and_clear();
  }
}

impl Default for Spinner {
  fn default() -> Self {
    Self::new()
  }
}
