# Licence notices

**Agreed:** 2026-11-15, by the user (with the MPL-2.0 exceptions) · **Due at:** before the first release

## What is postponed

Every build the app ships in carries the licence notices of everything it contains: the Rust crates and the npm
production dependencies of the UI. The five MPL-2.0 crates of Tauri (`cssparser`, `cssparser-macros`, `dtoa-short`,
`option-ext`, `selectors`) must be named there with a link to their source
([`../restrictions/dependencies.md`](../restrictions/dependencies.md)).

## What doing it needs

- Generate the Rust notices with `cargo-about` (an `about.toml` with the same allowlist as `deny.toml`) and the npm
  ones with a licence collector for production dependencies.
- Bundle the generated file in the app and show it in an "About / Licences" screen.
- A check in CI that fails on a licence outside the allowlist, for npm as for Rust.
