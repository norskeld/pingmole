use tabled::builder::Builder;
use tabled::settings::object::{Columns, Rows};
use tabled::settings::{Alignment, Style};

use crate::pinger::RelayTimed;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SortBy {
  Country,
  City,
  MeanRTT,
  #[default]
  MedianRTT,
  Distance,
}

#[derive(Debug)]
pub struct Reporter {
  sort_by: SortBy,
  timings: Vec<RelayTimed>,
}

impl Reporter {
  pub fn new(timings: Vec<RelayTimed>, sort_by: SortBy) -> Self {
    Self { sort_by, timings }
  }

  /// Sorts the relay timings.
  pub fn sort(&mut self) {
    self.timings.sort_by(|a_timed, b_timed| {
      let a_relay = a_timed.relay();
      let b_relay = b_timed.relay();

      match self.sort_by {
        | SortBy::Country => a_relay.country.cmp(&b_relay.country),
        | SortBy::City => a_relay.city.cmp(&b_relay.city),
        | SortBy::MeanRTT => a_timed.rtt_mean().cmp(&b_timed.rtt_mean()),
        | SortBy::MedianRTT => a_timed.rtt_median().cmp(&b_timed.rtt_median()),
        | SortBy::Distance => a_relay.distance.total_cmp(&b_relay.distance),
      }
    });
  }

  /// Builds the report table and prints it to stdout.
  pub fn report(&self) {
    let mut builder = Builder::default();

    builder.push_record(self.columns(vec![
      ("#", None),
      ("IP", None),
      ("Protocol", None),
      ("Country", Some(SortBy::Country)),
      ("City", Some(SortBy::City)),
      ("Distance", Some(SortBy::Distance)),
      ("RTT median", Some(SortBy::MedianRTT)),
      ("RTT mean", Some(SortBy::MeanRTT)),
    ]));

    for (idx, timed) in self.timings.iter().enumerate() {
      let relay = timed.relay();
      let distance = relay.distance.round();
      let rtt_mean = timed.rtt_mean().unwrap_or_default().as_secs_f64() * 1_000.0;
      let rtt_median = timed.rtt_median().unwrap_or_default().as_secs_f64() * 1_000.0;

      builder.push_record([
        (idx + 1).to_string(),
        relay.ip.to_string(),
        relay.protocol.to_string(),
        relay.country.clone(),
        relay.city.clone(),
        format!("~{distance} km"),
        format!("{rtt_median:.2} ms"),
        format!("{rtt_mean:.2} ms"),
      ]);
    }

    let mut table = builder.build();

    table
      .modify(Columns::new(5..), Alignment::right())
      .modify(Rows::new(..1), Alignment::left())
      .with(Style::rounded());

    println!("{table}");
  }

  /// Processes column names and marks the one being sorted.
  fn columns(&self, fields: Vec<(&str, Option<SortBy>)>) -> Vec<String> {
    fields
      .into_iter()
      .map(|field| {
        if let Some(sort_by) = field.1 {
          if sort_by == self.sort_by {
            format!("{} *", field.0)
          } else {
            field.0.to_string()
          }
        } else {
          field.0.to_string()
        }
      })
      .collect()
  }
}
