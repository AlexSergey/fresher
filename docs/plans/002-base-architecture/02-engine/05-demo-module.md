# 002/02/05 — The demo module

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** modules

## Files

- Root `Cargo.toml` — `members = ["app/tauri", "crates/*", "modules/*"]` (the only change to the root manifest after
  01/01).
- `modules/demo/Cargo.toml` — `fresher-module-demo`, dependency `fresher-core` only.
- `modules/demo/src/lib.rs` — `DemoModule`: no interests; `finish` publishes made-up findings: one per danger level, one
  with a `Partial` size, one `External` subject with a `ToolCommand` removal, one `EmptyFolder` removal; paths are
  under a made-up `/demo/` root that does not exist. It reads nothing and calls no command.
- `modules/demo/locales/en.json` — the texts of its categories (`demo.caches`, `demo.downloads`, `demo.images`,
  `demo.volumes`) and of its findings, all marked in their wording as example data.
- `modules/demo/src/settings.rs` — one setting `demo.count` (number, 1–50, default 8), so the settings path has a
  module setting to show.

## Produces

The module plan 003 builds its screens on and the e2e of 05/05 runs. Its contract check through the kit comes in 06/04.

## Checks

- Negative: `reads_nothing_from_the_disk` (no interests; `FakeCommandRunner` sees no call),
  `paths_are_under_the_made_up_root_only`, `never_lowers_a_category_danger_level`.
- Positive: `produces_one_finding_per_danger_level`, `honours_the_count_setting`, `every_text_key_it_emits_is_in_its_dictionary`.
- `cargo metadata` lists `fresher-module-demo`; `cargo nextest run -p fresher-module-demo` passes.

## Notes from the implementation

- **Dependencies**: `fresher-core` and `thiserror`. Ids, text keys and descriptors are checked constructors in the
  core, so `DemoModule::new() -> Result<Self, DemoError>` (no panicking path); `DemoError` needs `thiserror`, as every
  library crate. Dev-dependencies: `fresher-testing` (`FakeClock`, `FakeCommandRunner`) and `serde_json`.
- The settings code is a few lines in `src/settings.rs`: `demo.count` and reading it. A value out of range is
  resolved to the default by the schema; the module clamps it to 1–50 as well.
- **Eight examples**, repeated with `-2`, `-3`, … added to every name past the eighth, so `count` up to 50 gives 50
  distinct findings: caches (`Safe`, `EmptyFolder`, one `Partial`, one raised to `Medium` and moved to the Trash),
  downloads (`Low`, moved to the Trash), images (`Medium`) and volumes (`High`) of a made-up tool `demo-tool`,
  removed by `ToolCommand(demo-tool, ["remove", <name>])`. Paths are under `DEMO_ROOT` (`/demo`).
- **Texts**: each category's five texts, `demo.count` and `demo.enabled` (label, description: the schema adds
  `demo.enabled`, and its texts are in the module's namespace), `demo.finding.title` (`{name}`),
  `demo.finding.reason` and `demo.error.failed`. The dictionary test also fails on an unused key and checks that each
  emitted text's arguments are its placeholders.
- `finish` stops with `ModuleError::Cancelled` when the scan is cancelled.
- The demo is not registered anywhere yet: the app registers it in debug builds (plan 05).
