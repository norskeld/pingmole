use std::sync::Arc;
use std::thread;
use std::time::Duration;

use clap::Parser;
use pingmole::cli::{Cli, Spinner};
use pingmole::coord::Coord;
use pingmole::filters::{FilterByDistance, FilterByProtocol, FilterByRTT};
use pingmole::pinger::{RelayPingerConfig, RelaysPinger};
use pingmole::relays::{RelaysLoader, RelaysLoaderConfig};
use pingmole::reporter::Reporter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let spinner = Spinner::new();

  // -----------------------------------------------------------------------------------------------
  // 1. Get the current location, either via arguments or via Mullvad API.

  spinner.set_message("Getting current location");

  let location = match cli.latitude.zip(cli.longitude) {
    | Some((latitude, longitude)) => Coord::new(latitude, longitude),
    | None => Coord::fetch().await?,
  };

  thread::sleep(std::time::Duration::from_secs(1));

  // -----------------------------------------------------------------------------------------------
  // 2. Load relays from file and filter them.

  spinner.set_message("Loading relays");

  let loader = RelaysLoader::new(
    RelaysLoader::resolve_path()?,
    RelaysLoaderConfig { location },
    vec![
      Box::new(FilterByDistance::new(cli.distance as f64)),
      Box::new(FilterByProtocol::new(cli.protocol)),
    ],
  );

  let relays = loader.load()?;

  thread::sleep(std::time::Duration::from_secs(1));

  // -----------------------------------------------------------------------------------------------
  // 3. Ping relays.

  spinner.set_message("Pinging relays");

  let config = Arc::new(
    RelayPingerConfig::new()
      .set_count(cli.count)
      .set_timeout(Duration::from_millis(cli.timeout))
      .set_interval(Duration::from_millis(cli.interval)),
  );

  let pinger = RelaysPinger::new(
    relays,
    config,
    vec![Box::new(FilterByRTT::new(
      cli.rtt.map(Duration::from_millis),
    ))],
  );

  let timings = pinger.ping().await?;

  // -----------------------------------------------------------------------------------------------
  // 4. Print results.

  spinner.stop();

  let mut reporter = Reporter::new(timings, cli.sort_by.unwrap_or_default());

  reporter.sort();
  reporter.report();

  Ok(())
}
