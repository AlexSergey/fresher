# 018/02/03 — Fixture tests and registration

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, testing, fixtures, app

## Files

- `crates/testing/src/tree_description.rs` — `fill = "cachedir-tag"` on a `[[file]]`: the file holds the standard tag
  (`CACHEDIR_SIGNATURE`, a newline, then `# This file is a cache directory tag.` and its line on the standard, as
  `large_tree.rs` writes it); refused with `size` other than 0 or absent, and with `kind`. Fixture homes and case files
  read it (both are tree descriptions). `docs/testing.md`'s vocabulary gains the line.
- `fixtures/homes/dev-caches-similar-names.toml`, `dev-caches-unsigned.toml`, `dev-caches-kept.toml` (negative), then
  `dev-caches-caches.toml` (positive) — the design's section 6, "Fixture": every path under the fixture home
  (`Library/Caches/…` for `app-caches`, `Library/Developer/Xcode/DerivedData/…` for `library`, dot folders for
  `home`); stated facts (`kind = "socket"`, `owner = "system"`, `flags = ["user-lock"]`) where the disk cannot make
  them; valid tags with `fill = "cachedir-tag"`, an invalid one as a sparse file of 43 bytes; the derived data
  projects named `Demo-abcdefghijklmnopqrstuvwxyzab` and `My-App-bcdefghijklmnopqrstuvwxyzabc`.
- `modules/dev-caches/tests/fixtures.rs` (new) — `run_module(&module, &tree, &Ports::at(FIXTURE_NOW))` with
  `FakeRunningApps` (empty, Xcode `com.apple.dt.Xcode` open, or failing) and the facts lens, negatives first:
  - `dev-caches-similar-names`: no finding, no left-out;
  - `dev-caches-unsigned`: no finding, no left-out; the link's target untouched;
  - `dev-caches-kept` with Xcode open: `special` 1 (npm), `not-yours` 2 (Gradle, Cargo's `src`), `app-open` 1 (the
    project); the same home with running apps failing: `no-system` 1 instead of `app-open`, the rest unchanged;
  - `dev-caches-caches`: eleven findings, each in its category at its danger, its title and reason; the projects
    `for_app` Xcode with titles `Demo` and `My-App`; the same matches in two orders and a restored run give the same.
- `modules/dev-caches/tests/contract.rs` (new) — `assert_module_contract` (`Catalogue::Required`), `assert_texts_used`
  over every fixture run (every title, both reasons, every left-out reason), `check_removals_matched` empty.
- `fixtures/golden/dev-caches.json` (new) — `UPDATE_GOLDEN=1`, reviewed.
- `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` — the module registered in `modules()` in every build, release
  included, after `temp` and before the dev-only ones.

## Produces

The registered module and the tag in the tree vocabulary, for 03.

## Checks

- `crates/testing`, negatives first: `fill = "cachedir-tag"` with `size = 10` or with `kind` refused; then the written
  file is valid (`is_valid_cachedir_tag`).
- The fixtures above, negatives first; `cargo nextest run -p fresher-module-dev-caches -p fresher-testing -p
  fresher-app`; `modules/dev-caches` coverage at or above 90 % (`cargo llvm-cov -p fresher-module-dev-caches`).
- The sandbox autotest's guard now asks for `dev-caches`'s case file (failing until 03/01, so 02/03 and 03/01 land in
  one push).

## Conclusions

Built (8688168, 87ae785): `fill = "cachedir-tag"` in the tree vocabulary (the text of the large trees, one
`pub(crate)` constant shared with `large_tree.rs`, whose bytes and generator version are unchanged; refused with a size
(`FixtureError::TagWithSize`) or a kind); the fixture homes `dev-caches-similar-names`, `-unsigned`, `-not-folders`,
`-kept` and `-caches`; `tests/support/mod.rs`, `tests/fixtures.rs` and the extended `tests/contract.rs`; the golden
`fixtures/golden/dev-caches.json`; the crate registered in `app/tauri` after `temp` in every build.

Deviations and calls:

- The tag text is `# A cache directory tag.`, the large trees' own, not the brief's wording (F7): one shared constant,
  checked with `is_valid_cachedir_tag` on the written file.
- `dev-caches-not-folders.toml` is new (F2): the design's unsigned home put two entries at one path (go-build "README
  only" and "a file"; CocoaPods "Specs only" and "a link"), so those cases moved to it, with a derived data project that
  is a link and one that is a file. The design's table is corrected.
- The not-folders test asserts no finding and no left-out at all, stronger than F8's "none names the link or its
  target"; "unchanged after the cleanup" is proven by the sandbox and the core e2e.
- `serde_json` is the dev-dependency added here (the golden).

Tests: negatives first (similar names, unsigned and not-folders: nothing offered, nothing left out; kept with Xcode
open and with the running apps failing), then the eleven identities, every row's category, danger, title, reason and
`{tool}`, the projects' `for_app` and `{project}`, the golden unchanged when the matches are reversed or restored. RED
first (the homes missing), and six hand mutations of the negative homes (a valid Gradle tag, npm's `index-v5` added, a
Cargo tag added, `info.plist` added, `Plain` renamed, the link made a folder) each failed a test. `cargo nextest run`
on `fresher-app`, the module and `fresher-testing`: 431 run, 430 passed; the one failure, `every_registered_module_has_a_case_file`,
was expected until 03/01 (F12). Coverage of the module: 97.08 % regions, 99.78 % lines (bar 90 %). Parked minors:
`Fill::CachedirTag` writes without `sync_all` (the golden pins 4096 allocated; no instability seen in any run);
`fixtures.rs` names download against build with a bare `bool`.
