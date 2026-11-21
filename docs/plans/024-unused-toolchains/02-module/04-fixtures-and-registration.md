# 024/02/04 — Fixtures, golden and registration

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, fixtures, app

The module on fixture trees through `run_module` (the reference walk, the projects found as the engine finds them,
the files read by `DiskFiles`), its golden report, the contract test over the fixture runs, and its registration in
every build of the app. The design: [`../00-design.md`](../00-design.md), section 9. Pins and settings are stated with
01/02's `text`.

## Files

- `fixtures/homes/unused-toolchains-not-claimed.toml` (new) — `.rustup/settings.toml` with `default_toolchain =
  "1.80.0-aarch64-apple-darwin"` and an override naming `nightly-2024-04-01-aarch64-apple-darwin`; toolchains
  `1.80.0-…`, `nightly-2024-04-01-…`, `stable-aarch64-apple-darwin`, `nightly-aarch64-apple-darwin`, `my-custom`, a
  link `dev` to a folder outside `.rustup`, `1.79.0-…` kept by `Documents/dev/pinned/rust-toolchain.toml` (`channel =
  "1.79"`, beside a `Cargo.toml`), `nightly-2024-03-01-…` kept by `Code/legacy/rust-toolchain` (one line),
  `1.77.2-…` kept by `Code/asdf/.tool-versions` (`rust 1.77.2`); `.nvm/alias/default` `20`, `.nvm/alias/myproj`
  `lts/hydrogen`, `.nvm/alias/lts/hydrogen` `v18.20.0`; versions `v20.11.0`, `v22.1.0` (the newest), `v18.20.0`,
  `v16.20.2` kept by `Documents/dev/web/.nvmrc` `16`, `v19.9.0` kept by `Code/app2/.node-version`, `v18.19.0` kept by
  `Code/asdf/.tool-versions` (`nodejs 18.19.0`); a folder `18.1.0` off shape. Run with the switch on: nothing claimed.
- `fixtures/homes/unused-toolchains-kept.toml` (new) — a toolchain with a socket inside (`special`), one with a file
  carrying `flags = ["user-lock"]` (`not-yours`); a second home without `settings.toml` (`default-unknown` on every
  offerable toolchain) and with a `.nvmrc` `lts/unknown` (`pin-unknown` on every offerable Node version); as a
  home of its own, `unused-toolchains-kept-unknown.toml`. `partial-scan` is proven only in `src/tests.rs` (a walk that
  is not complete), and the contract test's text check is fed by that run.
- `fixtures/homes/unused-toolchains-offered.toml` (new) — `1.70.0-aarch64-apple-darwin` and
  `nightly-2023-06-01-aarch64-apple-darwin` nothing names (Low); `v12.22.12` nothing names and `v14.21.3` named only by
  `Code/web3/node_modules/dep/.nvmrc` (`14`) of a project `Code/web3` (Medium); beside the defaults and the newest of
  the not-claimed home. Small files with real sizes.
- No `-off` home: the offered home is run with the Node switch off (no Node version claimed, the Rust ones offered).
- `fixtures/golden/unused-toolchains.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings (titles, reasons,
  commands, sizes, dangers), the left-outs.
- `modules/unused-toolchains/tests/fixtures.rs` (new) and `tests/support/mod.rs` — the homes through `run_module`
  (`Ports.settings` with `unused-toolchains.node-versions = true`, but off for the offered home's switch-off run), as
  `modules/local-ai-models/tests/fixtures.rs`; the golden comparison.
- `modules/unused-toolchains/tests/contract.rs` — the fixture runs added, so every text and every declared left-out is
  used.
- `app/tauri/Cargo.toml` — the crate as a dependency; `app/tauri/src/compose.rs` — `UnusedToolchains(#[from] …)` in
  the compose error and the module in `modules()`, beside `local-ai-models`; `app/tauri/src/commands.rs` — the module
  list, if it names modules.

## Produces

The registered module, for 03.

## Checks

- `tests/fixtures.rs`, negatives first: `mod negative` `nothing_in_the_not_claimed_home_is_offered_or_left_out`,
  `the_kept_homes_offer_nothing_and_leave_out_each_with_its_reason`, `the_switch_off_claims_no_node_version`;
  `mod positive` `the_offered_home_offers_each_unused_version_in_its_category`,
  `a_pin_inside_node_modules_keeps_nothing`, `the_golden_report_matches`.
- `tests/contract.rs` passes with the fixture runs.
- The app's composition tests (the module registered, its id taken once); `cargo nextest run -p
  fresher-module-unused-toolchains -p fresher-app`; coverage of the crate at or above 90 %.

## Conclusions

Built with these changes. There is no `-off` home: the switch-off run reuses the offered home with the switch
false, a duplicate would add nothing. A fifth home, kept-unknown, exists because one TOML file is one home and the
unknown-pointer reasons need their own (its Node sizes are distinct, 500, 600, 700 bytes, after the final fix wave).
`partial-scan` has no fixture run: the test ports' `avoid` never makes a walk incomplete and a fixture cannot state an
unreadable folder, so it is proven in `src/tests.rs` only, and the contract test names the four reasons a fixture run
gives. The app's test `every_registered_module_has_a_case_file` was red from this step until step 03/01 added the case
file (no push in between). The `Cargo.lock` change is a second commit, because the hook ran before the lock moved.
