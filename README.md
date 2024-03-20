# pingmole

[![Checks](https://img.shields.io/github/actions/workflow/status/norskeld/pingmole/checks.yml?style=flat-square&colorA=22272d&colorB=22272d&label=checks)](https://github.com/norskeld/pingmole/actions/workflows/checks.yml)

CLI that helps to filter [Mullvad] servers and pick the closest one.

![Results example](/.github/assets/results.png)

<sup><sub>* The actual output is not colored, this is just for demo purposes. :)</sub></sup>

## Installation

> [!NOTE]\
> By default pingmole will try to locate the `relays.json` file and use it to build the list of available servers. Otherwise, a request to Mullvad API will be performed.

### From source (Cargo)

Make sure to [install Rust toolchain][rust-toolchain] first. After that you can install pingmole via **Cargo**:

```shell
cargo install --locked --git https://github.com/norskeld/pingmole
```

## Features

- [x] Filter servers by:
  - [x] Ping round-trip time;
  - [x] Used protocol: OpenVPN or WireGuard;
  - [x] Distance from the current location.
- [x] Sort results by:
  - [x] Mean RTT;
  - [x] Median RTT (default);
  - [x] Distance;
  - [x] Country;
  - [x] City.
- [x] Ping matching Mullvad servers.
- [x] Print results in a table.

## Distance calculation

> [!NOTE]\
> While pingmole automatically detects your geolocation using the [am.i.mullvad.net](https://am.i.mullvad.net/json) endpoint, I highly recommend specifying `latitude` and `longitude` via the corresponding CLI options to pinpoint your location. This is because often, detecting the geolocation using the IP address is simply wrong.

Distance is calculated using the [haversine formula][haversine]. This affects the accuracy of the results, but generally it's good enough.

## Pinging

> [!NOTE]\
> Results may vary depending on the number of factors, including the current network or target server load. It's a good idea to run the test multiple times and try to increase the number of pings, like 16.

Pinging is done using TCP, not ICMP. Reasons:

- ICMP pinging turned out to be harder to implement, so I've decided to roll with TCP.
- ICMP requires raw sockets and, consequently, elevated priviliges on Linux/macOS.
- ICMP pinging can be less precise due to lower handling/forwarding priority.

## License

[MIT](LICENSE).

<!-- Links. -->

[mullvad]: https://mullvad.net
[rust-toolchain]: https://rust-lang.org/tools/install
[haversine]: https://en.wikipedia.org/wiki/Haversine_formula
