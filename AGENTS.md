# YAAMS Commander

Cross-platform (macOS, Windows, Linux) desktop client for [YAAMS](https://github.com/YAAMSOrg/yaams-server), a virtual airline management system. It talks to a YAAMS instance through its REST API v1 and should look and feel like real-world airline operations software.

- Implemented: first-run wizard, multi-profile configuration, flight display (list + detail), PIREP filing.

## Stack

- Rust stable, Cargo workspace
- UI: Iced (pure Rust, latest stable release), tokio executor
- HTTP: reqwest with rustls + platform verifier (OS trust store - the dev instance uses an internal CA)
- serde / serde_json / toml, `dirs` for config paths, `chrono` for UTC times

## Layout

```
crates/yaams-api/         API client: models, errors, pagination. No UI dependencies.
crates/yaams-commander/   Iced desktop app
  src/app.rs              Application state, Message, update/view, subscriptions
  src/config.rs           Profiles, load/save, per-OS paths, file permissions
  src/theme.rs            Palette, custom theme, widget styles
  src/icons.rs            Lucide icon font helpers
  src/widgets/            Table, badges, sidebar, top bar, status bar, form fields, toasts
  src/screens/            wizard, flights, flight detail, pirep form, not implemented, settings
  assets/                 Embedded fonts, brand logos, app icons
```

## Commands

```
cargo build
cargo run -p yaams-commander
cargo test --workspace
cargo test --workspace -- --ignored      # live tests, need YAAMS_URL + YAAMS_TOKEN env
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

Testing layers:

- `yaams-api/tests/client.rs` - wiremock-based client tests (auth header, paging, error mapping, abilities defaults)
- `yaams-commander/src/ui_tests.rs` - headless UI tests with `iced_test`: feed fixture data through the real `App::update`, find/click widgets by text, assert gating. `YAAMS_UI_SNAPSHOTS=<dir>` writes PNG snapshots for visual review. CI uses `ICED_TEST_BACKEND=tiny-skia`.
- Never drive the real app window with synthetic OS input - use the headless UI tests instead.

## API rules

- All calls go to `<instance>/api/v1/...` with `Authorization: Bearer <token>` and `Accept: application/json`. Only `GET /api/v1/info` is public.
- Errors are JSON: `401` invalid/missing token, `403` no permission (`message`), `404`, `422` validation (`message` + `errors` keyed by parameter name).
- Lists share one contract: `page` (from 1), `per_page` (1-100, default 25), `sort` (documented key, `-` prefix = descending). Response has `data`, `links` (`first`, `last`, `prev`, `next`, null if none) and `meta` (`current_page`, `from`, `to`, `per_page`, `total`, `last_page`, `path`). To collect every page, increment `page` until `meta.current_page == meta.last_page` (the `links` URLs carry the server's configured APP_URL, which may differ from the URL the user connected with).
- Unknown sort keys or invalid filter values return `422` - never send unvalidated values.
- Dates in filters are UTC days `Y-m-d`; PIREP block times are UTC `Y-m-d H:i:s`.
- Online network IDs differ per instance - always load them from `GET /api/v1/online-networks`.
- Flight status ids: 1 Pending, 2 Accepted, 3 Rejected.

## Permissions

- Gate UI exclusively on `abilities` flags, never on role names (roles are display-only).
- Instance abilities (`GET /api/v1/user` -> `abilities`): `foundAirline`, `administerInstance`.
- Per-airline (`airlines[].membership.abilities`): `fileFlights`, `reviewFlights`, `addAircraft`, `editAircraft`, `manageMembers`, `manageInviteCodes`, `manageNotams`, `editSettings`. `membership` is null for non-members.
- A missing or unknown ability key means `false` (`#[serde(default)]`).
- A suspended airline (`isSuspended`) is read-only: all its abilities are false; show a read-only banner.
- The server still enforces everything - always handle `403` gracefully.
- Hide nav entries the user cannot use instead of showing dead ends.

## Design

Brand assets come from the YAAMS logo package (chevron wing icon + wordmark).

| Token | Value | Use |
|---|---|---|
| Primary Dark | `#20242e` | App background, sidebar |
| Surface | `#282e3a` | Panels, cards, table header |
| Accent Amber | `#e8a53f` | Logo, primary buttons, active nav, highlights |
| Amber dark | `#865f31`, `#574736` | Hover/pressed and subtle accent fills |
| Success | `#3a8f42` / `#d9f4d9` | Accepted PIREP, active aircraft |
| Danger | `#cc272e` / `#ffdfda` | Reject, delete, form errors |
| Warning | `#ccac16` / `#f9efc9` | Pending, inactive, read-only |
| Info | `#2c94b0` / `#d6f0f9` | Informational notices |
| Secondary | `#69737d` / `#e2e5e8` | Neutral, cancel buttons |

- Fonts (embedded): Space Grotesk Bold (headings, wordmark), Inter (body), JetBrains Mono (ICAO codes, flight numbers, times, figures). Use static font files only - cosmic-text does not pick instances from variable fonts.
- Icons: Lucide icon font. Every nav item and primary action has an icon.
- Dense, data-first layout: sidebar nav, top bar with airline/profile switchers and a UTC clock, status bar with instance info. Times are shown in UTC with a `Z` suffix.

## Configuration

Single TOML file holding all profiles (URL, token, preferences). Token lives in the file, so it is written with mode 600 (directory 700) on Unix and saved atomically.

- macOS: `~/Library/Application Support/YAAMS Commander/config.toml`
- Linux: `$XDG_CONFIG_HOME/yaams-commander/config.toml` (default `~/.config/yaams-commander/`)
- Windows: `%APPDATA%\YAAMS Commander\config.toml`
- Override: `YAAMS_COMMANDER_CONFIG` env var

## Conventions

- Never use the em dash character; use a normal dash `-`.
- Commit messages and PR texts: short unordered lists, no long paragraphs, no AI co-author line.
- Prefer quality, simplicity, robustness and maintainability over development cost.
- Keep `yaams-api` free of UI code; keep validation rules that mirror the server in one place and unit-test them.
- `cargo fmt` and clippy with `-D warnings` must pass.

