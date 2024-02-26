# pingmole

[![Checks](https://img.shields.io/github/actions/workflow/status/norskeld/pingmole/checks.yml?style=flat-square&colorA=22272d&colorB=22272d&label=checks)](https://github.com/norskeld/pingmole/actions/workflows/checks.yml)

CLI that helps to filter Mullvad servers and pick the closest one.

## Features

- [x] Ping matching Mullvad relays and print the results.
- [x] Filter servers by:
  - [ ] Ping round-trip time;
  - [x] Used protocol: OpenVPN or WireGuard;
  - [x] Distance from the current location;

Pinging is done using TCP, not ICMP. Reasons:

- ICMP pinging turned out to be harder to implement, so I've decided to use TCP.
- ICMP requires raw sockets and, consequently, elevated priviliges on Linux/macOS.
- ICMP pinging can be less precise due to lower handling/forwarding priority.

## License

[MIT](LICENSE).
