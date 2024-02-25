use std::time::Duration;

use clap::builder::PossibleValue;
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};

use crate::relays::Protocol;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Filter servers by used protocol.
  #[arg(short, long, value_parser = clap::value_parser!(Protocol))]
  pub protocol: Option<Protocol>,

  /// Filter servers by maximum physical distance (in km).
  #[arg(short, long, default_value_t = 500)]
  pub distance: usize,

  /// Filter servers by maximum rtt (in ms).
  #[arg(short, long)]
  pub rtt: Option<usize>,

  /// How many pings to send for each relay.
  #[arg(short, long, default_value_t = 4)]
  pub count: usize,

  /// Specify ping timeout (in ms).
  #[arg(long, default_value_t = 750)]
  pub timeout: u64,

  /// Specify the latitude.
  #[arg(long, requires = "longitude")]
  pub latitude: Option<f64>,

  /// Specify the longitude.
  #[arg(long, requires = "latitude")]
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
  pub fn set_message(&self, message: &'static str) {
    self.spinner.set_message(message);
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
