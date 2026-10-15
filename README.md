<p align="center">
  <img src="crates/yaams-commander/assets/brand/yaams-1b-lockup-dark.svg" alt="YAAMS" width="420">
</p>

# YAAMS Commander

Desktop operations client for [YAAMS](https://github.com/YAAMSOrg/yaams-server) - Yet Another Airline Management System. Runs natively on macOS, Windows and Linux.

## Features

- First-run wizard: connect to a YAAMS instance with its URL and a personal API token
- Multiple connection profiles, switchable from the top bar
- Flight log: your PIREPs per airline with status counters, filters (status, departure, arrival, date range), sorting, paging and a detail panel
- PIREP filing with validation that mirrors the server rules (callsign, cruise altitude steps, block times, location continuity, ...)
- Permission-aware UI: navigation follows the `abilities` reported by the API; suspended airlines are read-only
- Areas that are not implemented yet (review queue, fleet, members, notifications, ...) show a placeholder with a link to the web interface

## Requirements

- A YAAMS instance with API v1 and a personal API token (web UI: Settings → API tokens)
- Rust stable (to build from source)
- Linux: `libxkbcommon`, `libwayland` / `libx11` development packages

## Build and run

```
cargo run --release -p yaams-commander
```

## Configuration

All profiles and preferences live in one TOML file, written with owner-only permissions:

| OS | Location |
|---|---|
| macOS | `~/Library/Application Support/YAAMS Commander/config.toml` |
| Linux | `$XDG_CONFIG_HOME/yaams-commander/config.toml` (default `~/.config/yaams-commander/`) |
| Windows | `%APPDATA%\YAAMS Commander\config.toml` |

Set `YAAMS_COMMANDER_CONFIG` to use a different file.

## Development

```
cargo test --workspace                                   # unit, API mock and headless UI tests
YAAMS_UI_SNAPSHOTS=/tmp/snaps cargo test -p yaams-commander ui_tests   # also write UI snapshots
YAAMS_URL=https://va.example.com YAAMS_TOKEN=... cargo test -p yaams-api -- --ignored   # read-only live checks
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Packages are built with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager) (see `.github/workflows/release.yml`):

```
cd crates/yaams-commander && cargo packager --release
```

## Project layout

- `crates/yaams-api` - typed client for the YAAMS REST API v1
- `crates/yaams-commander` - the Iced desktop application

## License

GPL v3.0
