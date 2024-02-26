use std::thread;
use std::time::Duration;

use clap::Parser;
use pingmole::cli::{Cli, Spinner};
use pingmole::coord::Coord;
use pingmole::filters::{FilterByDistance, FilterByProtocol};
use pingmole::pinger::RelayPinger;
use pingmole::relays::RelaysLoader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let spinner = Spinner::new();

  // -----------------------------------------------------------------------------------------------
  // 1. Get the current location, either via arguments or via Mullvad API.
  spinner.set_message("Getting current location");

  let user = match cli.latitude.zip(cli.longitude) {
    | Some((latitude, longitude)) => Coord::new(latitude, longitude),
    | None => Coord::fetch().await?,
  };

  thread::sleep(std::time::Duration::from_secs(1));

  // -----------------------------------------------------------------------------------------------
  // 2. Load relays from file and filter them.
  spinner.set_message("Loading relays");

  let loader = RelaysLoader::new(
    RelaysLoader::resolve_path()?,
    vec![
      Box::new(FilterByDistance::new(user, cli.distance as f64)),
      Box::new(FilterByProtocol::new(cli.protocol)),
    ],
  );

  let relays = loader.load()?;

  thread::sleep(std::time::Duration::from_secs(1));

  // -----------------------------------------------------------------------------------------------
  // 3. Ping relays.
  spinner.set_message("Pinging relays");

  let mut tasks = Vec::new();
  let mut timings = Vec::new();

  for relay in relays {
    let mut pinger = RelayPinger::new(relay);

    pinger.set_count(cli.count);
    pinger.set_timeout(Duration::from_millis(cli.timeout));

    tasks.push(tokio::spawn(pinger.execute()));
  }

  for task in tasks {
    timings.push(task.await?);
  }

  // -----------------------------------------------------------------------------------------------
  // 4. Print results.
  spinner.stop();

  dbg!(timings);

  Ok(())
}
