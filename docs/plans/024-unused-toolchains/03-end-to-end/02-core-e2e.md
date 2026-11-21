# 024/03/02 — The core e2e

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** e2e, fixtures

The engine with the real scanner, the module, the real `DiskCleaner` and `DiskFiles` on a sandbox home, as the flows
of `crates/e2e/tests/local_ai_models.rs`. The design: [`../00-design.md`](../00-design.md), section 9.

## Files

- `fixtures/homes/unused-toolchains-flow.toml` (new) — a home with rustup's settings (default `stable-…`), a moving
  channel, an old toolchain, a dated nightly, a project in `Documents/dev` with a `rust-toolchain.toml`, nvm's default
  alias, the newest Node version and an old one; the stored settings turn the Node switch on.
- `crates/e2e/tests/unused_toolchains.rs` (new) and `crates/e2e/tests/unused_toolchains_home/` (the flow's home and
  helpers, through the shared harness of the sandbox flow homes) — the flows, negatives first:
  - `mod negative`: `the_default_the_channel_and_the_pinned_toolchain_are_never_offered`;
    `a_pin_written_between_two_scans_keeps_its_version` (a `.nvmrc` naming the old Node version written after the
    first scan: the rescan does not offer it); `a_version_with_a_file_open_is_skipped_in_use` (the open files fake
    names its `bin/node`: the item ends `InUse`, nothing moved);
  - `mod positive`: `the_scan_offers_each_unused_version_in_its_category_with_its_sizes`;
    `a_confirmed_cleanup_moves_the_toolchain_folder_to_the_trash` (the folder lands whole in the sandbox's Trash, the
    journal has one entry, `settings.toml` and the pins untouched; put back restores it);
    `a_restored_scan_offers_the_same_versions` (the cached matches at launch, the files read again).
- `fixtures/golden/unused-toolchains-flow.json` (new) — the flow's report, reviewed.
- `crates/e2e/Cargo.toml` — the module as a dev-dependency.
- `docs/testing.md` — `unused_toolchains.rs` in the list of core e2e files.

## Produces

The module proven through the engine and the cleaner, for 04.

## Checks

- `cargo nextest run -p fresher-e2e --test unused_toolchains`; `npm run test:e2e` (the core part).

## Conclusions

Built with these changes. The Node switch is set by the flow through `update_setting` (the real store), not by the
fixture, because the home fixture format holds no settings; `Flow::with(changed)` was added for the rescan journal.
The brief's restored-scan test is `a_restored_scan_reads_the_pins_again`: it rewrites an already matched
`rust-toolchain.toml` between the restart and the restore and proves the offer moves, because a restore replays cached
matches and a new file has none. The in-use flow asserts two Trash entries and no moved record for the skipped
version. 6 core e2e tests.
