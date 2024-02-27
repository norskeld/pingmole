# pingmole

[![Checks](https://img.shields.io/github/actions/workflow/status/norskeld/pingmole/checks.yml?style=flat-square&colorA=22272d&colorB=22272d&label=checks)](https://github.com/norskeld/pingmole/actions/workflows/checks.yml)

CLI that helps to filter [Mullvad] servers and pick the closest one.

![Results example](/.github/assets/results.png)

## Installation

[Mullvad] must be already installed.

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
- [x] Ping matching Mullvad relays.
- [x] Print results in a table.

## Pinging

> [!NOTE]\
> Results may vary depending on the number of factors, including the current network or target relay load. It's a good idea to run the test multiple times and try to use higher pings count, like 16.

Pinging is done using TCP, not ICMP. Reasons:

- ICMP pinging turned out to be harder to implement, so I've decided to use TCP.
- ICMP requires raw sockets and, consequently, elevated priviliges on Linux/macOS.
- ICMP pinging can be less precise due to lower handling/forwarding priority.

## License

[MIT](LICENSE).

<!-- Links. -->

[mullvad]: https://mullvad.net
[rust-toolchain]: https://rust-lang.org/tools/install
