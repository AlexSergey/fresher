# 017/02/03 — Fixture tests and registration

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, fixtures, app

## Files

- `fixtures/homes/temp-recent.toml`, `temp-system.toml`, `temp-special.toml`, `temp-running.toml` (negative), then
  `temp-old.toml` (positive) — the design's section 9, "Fixture", with stated facts (01/05) where the disk cannot make
  them, the places laid out by the fixture known folders (`private/var/folders/xx/fixture/T`, `private/tmp`).
- `modules/temp/tests/fixtures.rs` — `run_module(&module, &tree, &Ports::at(FIXTURE_NOW))` with `FakeRunningApps`
  and `FakeInstalledApps`, negatives first; the findings and the left-out counts per reason; a test that hands the
  same matches in two orders gives the same answers; a test with the setting at 14 days; a run with a failing
  running-apps fake (`no-system`) and a restored run (`scan-again`), so every declared reason is used.
- `modules/temp/tests/contract.rs` — `assert_module_contract` (`Catalogue::Required`), `assert_texts_used` over every
  fixture run (every left-out reason and both finding reasons used), `check_removals_matched` empty.
- `fixtures/golden/temp.json` (new) — `UPDATE_GOLDEN=1`, reviewed.
- `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` — the module registered in `modules()` in every build, release
  included, before the dev-only ones.

## Produces

The registered module, for 03.

## Checks

- Negatives: nothing of `temp-recent`, `temp-system`, `temp-special`, `temp-running` is offered; their left-outs are
  exactly the design's; `C/`, `0/`, `X/`, `TemporaryItems` and a home file named like a temporary item are never
  matched.
- Positives: `temp-old`'s five items offered, Safe, each with its place's reason and the right number of days; the
  item named after an installed app carries its bundle id.
- `cargo nextest run -p fresher-module-temp -p fresher-app`; `modules/temp` coverage at or above 90 %
  (`cargo llvm-cov -p fresher-module-temp`); the sandbox autotest's guard now asks for `temp`'s case file (failing
  until 03/01, so 02/03 and 03/01 land in one push).

## Conclusions

Built (0cb1229): the five fixture homes, `tests/fixtures.rs`, `tests/contract.rs` and `fixtures/golden/temp.json`; the
module registered after `jetbrains`, before the dev-only ones, in every build. Everything meant to look old states
`changed` on every entry inside it, since the real status change comes after `FIXTURE_NOW`.

Beyond the design's cases, so that every declared reason has a fixture and the rulings of 02/02 are covered: an old file
read a day ago, another user's item, an item you locked, a folder holding `LCK..ttyS0` and a hidden Editor file. The
counts are therefore: `temp-recent` recent ×4; `temp-system` system ×5 and locked ×1; `temp-special` special ×3 and lock
×4; `temp-running` app-open ×2; a failing list of open apps no-system ×5; a restored run scan-again ×5; 14 days keeps
the 10-day item. `temp-old` offers its five items at Safe, the Viewer's with its bundle id, the link as itself.

Needed beyond the brief's files, accepted by ruling: `Ports::restored` and `MatchOrder` in `run_module`, a link's own
`age` and stated `changed` in the tree vocabulary (set through `O_SYMLINK`, the target untouched), and the parked
`OTHER_USER` change to `nobody`. Coverage of `modules/temp`: 100 % of lines and functions, 97 % of regions.
