# 017/02/01 — The crate, the group System and the texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, core, app/ui, docs

## Files

- `crates/core/src/group.rs` — `ModuleGroup::System` (`system`): "what macOS and the user's apps keep for themselves";
  `ALL` is `[System, Development, Debug]` (agent's call 11). `crates/core/locales/en.json`:
  `core.group.system.name` "System", `.description` "What macOS and your apps keep for themselves, such as temporary
  files." `crates/core/tests/typescript.rs` and `app/ui/src/ipc/bindings.generated.ts` regenerated; the UI's group
  stories and tests (`scan-hero/story-data.ts`, `settings/ignored-group.test.tsx`, the sidebar's) gain the group.
- `modules/temp/Cargo.toml` (new) — `fresher-module-temp`; the workspace's `version`, `edition`, `rust-version`,
  `license`, `publish`; `[lints] workspace = true`; depends on `fresher-core`, `thiserror`; dev-dependencies
  `fresher-testing`, `proptest`, `serde_json`. Root `Cargo.toml`: the member.
- `modules/temp/src/lib.rs` — `#![forbid(unsafe_code)]`; `TempError`; `TempModule::new()`: the descriptor (`temp`,
  `ModuleGroup::System`, the category `temp.old-files` at Safe, the setting `temp.older-than-days`, the dictionary)
  `.with_problems(…)` (the eight left-out keys); `interests()`: `Under(UserTemp, "*")` and `Under(SharedTemp, "*")`,
  `Claim::Whole`; `on_match` and `finish` return `Ok(())` until 02/02.
- `modules/temp/locales/en.json` — every text of the design, section 8: the category's five texts, the switch's label
  and description, the setting's label and description, the finding's title and two reasons, the eight left-out
  reasons. Plain English, no jargon (rule 7).
- `docs/categories/temp-old-files.md` (new) — in the catalogue's format: Safe; what it is; where it lives (the user's
  temporary folder, `$TMPDIR`, and the user's own items in `/private/tmp`); what macOS clears on its own (daily, older
  than three days; `/private/tmp` at restart); what happens if removed; how the app decides (the decision table, the
  places it never touches, and the place's contract of decision 6). `docs/categories/README.md`: its row.

## Produces

`TempModule::new()` and its texts for 02/02 and 03/03; the group System for every screen.

## Checks

- `tests/contract.rs`: `assert_module_contract(&module, &Catalogue::Required(docs/categories))` passes.
- `crates/core/src/group.rs`'s tests: every group has its two texts; `ALL` lists every variant once.
- `npx vitest run src/screens/settings/ src/screens/frame-sidebar` (the group shows with its name).
- `cargo nextest run -p fresher-module-temp -p fresher-core`; clippy; the crate guard and the dependency guard (no
  other module).

## Conclusions

Built (1e68a75): `ModuleGroup::System` with its texts, `ALL` in the order System, Development, Debug (agent's call 11),
the engine's registry and the UI's group order; `modules/temp` with its descriptor (the category `temp.old-files` at
Safe, the setting `temp.older-than-days` from 7 to 30 by decision 9, the eight left-out keys), its two interests and
every text of the design's section 8; `docs/categories/temp-old-files.md` and its row; the crate's rows in
`docs/dependencies.md`. The contract test passed with `Catalogue::Required`. Small text and doc points were parked for
the final review.
