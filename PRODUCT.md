# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

Recorded as `web` because Impeccable only knows web/ios/android/adaptive and this app renders its own custom (non-OS-native) design language. In reality it is a cross-platform native desktop app (macOS, Windows, Linux) built with Iced: one design language shared across all three OSes, no per-OS widget adaptation, no browser or mobile target. Design for a desktop window (mouse + keyboard, resizable; default 1440x900, minimum 1180x720 per `src/main.rs`), not for responsive web breakpoints.

## Users

Two equally important audiences, both members of virtual airlines run on a YAAMS instance:

- **Pilots** - fly in a flight simulator and log their flights as PIREPs (pilot reports): file a flight after landing, check its review status, browse their flight log.
- **Airline staff** - owners and managers who run the virtual airline: review the PIREP queue, manage fleet, members, invite codes, NOTAMs and airline settings.

One person is often both, possibly in several airlines and on several YAAMS instances. What they may do is defined per airline by the API's `abilities` flags, not by a fixed user type.

## Product Purpose

A desktop client for YAAMS that lets pilots and staff do their virtual-airline work outside the browser, against any YAAMS instance through its REST API v1. Success means filing and reviewing flights feels faster and more serious than the web UI, and the app is a natural thing to keep open next to the simulator.

## Positioning

- **Ops-software feel:** it deliberately looks and behaves like professional airline enterprise / operations software - dense, data-first, UTC everywhere, ICAO codes and flight numbers as first-class data. The seriousness is the fun: a hobby tool that plays it straight.
- **Sim-side companion:** it lives on the desktop next to the flight simulator, not in a browser tab. Integration with simulator data (ACARS-style features) is a direction, not yet built.
- Multi-profile: several instances/accounts in one app, switchable from the top bar.

## Operating Context

- Used before and after sim sessions, often on the same machine as the simulator.
- Aviation vocabulary is native: PIREP, callsign, ICAO airport codes, block off/on times (UTC `Y-m-d H:i:s`), cruise altitude, online networks (loaded per instance), NOTAMs.
- Times are always UTC with a `Z` suffix.
- PIREP status lifecycle: Pending, Accepted, Rejected.
- Suspended airlines are read-only for everyone.
- The web interface remains the fallback for features the client does not implement yet.

## Capabilities and Constraints

- Implemented: first-run wizard, multi-profile configuration, flight log (list, filters, sorting, paging, detail panel), PIREP filing with client-side validation mirroring the server.
- Not yet implemented (placeholder screens linking to the web UI): review queue, fleet, members, airline settings, notifications, found an airline, administration.
- UI is gated exclusively on `abilities` flags; entries the user cannot use are hidden, not disabled. The server still enforces everything, so `403` must be handled gracefully.
- Rust + Iced (custom theme, embedded fonts, Lucide icon font). Visual verification goes through headless `iced_test` snapshots, never synthetic OS input.
- The API token is stored in the config file; it must never be logged or displayed casually.

## Brand Commitments

- Name: **YAAMS Commander**; YAAMS = Yet Another Airline Management System.
- Brand assets from the YAAMS logo package (chevron wing icon + wordmark) in `crates/yaams-commander/assets/brand/`.
- Copy never uses the em dash character; use a normal dash `-`.
- Existing visual tokens and fonts are documented in `CLAUDE.md` (Design section) and implemented in `src/theme.rs`.

## Evidence on Hand

- Brand files: `crates/yaams-commander/assets/brand/yaams-1b-icon.svg`, `yaams-1b-lockup-dark.svg`, `yaams-1b-lockup-light.svg`.
- Real data comes only from a live YAAMS instance; UI tests use fixture data.
- No user testimonials, usage numbers, screenshots for marketing, or airline partnerships exist - do not invent any.

## Product Principles

1. **Play it straight.** Treat a hobby like a professional operation: precise data, correct terminology, no cute or gamified tone.
2. **Data first.** Flight numbers, codes, times and statuses are the content; chrome stays out of their way.
3. **Show only what you can do.** Permissions shape the interface; no dead ends, clear read-only states.
4. **Trust the server, help the user.** Mirror server validation so mistakes surface before submit, and explain server errors plainly when they still happen.
5. **Built to sit next to the sim.** Fast to open, quick to file, calm to leave running.
