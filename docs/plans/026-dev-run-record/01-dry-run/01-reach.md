# 026/01/01 — The dry-run reach and its outcome

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, store, cleaner, sandbox, app/ui

The cleaner learns to check the real disk without acting on it. Today a debug build's `Reach::Sandbox` refuses every
real-Mac item at check 1, so a plan made there is empty; `Reach::DryRun` admits what `Reach::RealDisk` admits, every
later check runs, and the one place that makes every act (`pipeline::act_on`) ends the item `SkipReason::DryRun`
instead. Debug builds only. The design: [`../00-design.md`](../00-design.md), sections 1.1 to 1.3.

## Files

- `crates/core/src/cleanup/outcome.rs` —
  - `SkipReason::DryRun` (doc: "A development build on the real Mac: every check passed and nothing was done."),
    after `ForcedTool`, before `CheckFailed`;
  - `SkipCode::DryRun`, its arm in `SkipReason::code`, in `SkipCode::ALL` (now `[Self; 20]`), `id()` `"dry-run"`;
    not in `how_key`'s list (no instruction: nothing is wrong);
  - the test sample `sample::skip_reasons()` gains `SkipReason::DryRun` at the same place.
- `crates/core/locales/en.json` — `"core.cleanup.skip.dry-run": "Development build: a dry run. It passed every check
  and was left where it is."`.
- `crates/store/src/journal/records.rs` — `SkipCodeForm`'s `mirror!` gains `DryRun` at the same place.
- `crates/cleaner/src/reach.rs` —
  - `Reach::DryRun(DryRunToken)` under `#[cfg(debug_assertions)]`, documented "Anywhere the other checks allow, acting
    on nothing: a development build on the real Mac.";
  - `#[cfg(debug_assertions)] #[derive(Debug)] pub struct DryRunToken(());` with
    `#[cfg(debug_assertions)] pub const fn new() -> Self` (`#[expect(clippy::new_without_default, reason = "a
    `Default` would be one more way to make the token")]`, as `RealDiskToken::new`), documented as in the design;
  - `admits`, `admits_tools`, `admits_unplaced`: `DryRun` answers as `RealDisk` (`true`), the arm cfg'd;
  - `pub(crate) const fn acts(&self) -> bool`: `false` for `DryRun` (cfg'd arm), `true` otherwise; the module's doc
    names the fourth reach.
- `crates/cleaner/src/lib.rs` — `#[cfg(debug_assertions)] pub use reach::DryRunToken;`; the crate doc: "In a
  development build on the real Mac it checks and acts on nothing ([`Reach`])".
- `crates/cleaner/src/pipeline.rs` — `act_on`: after `verify_tool` returns `Ok(())`, and after `verify(item, guard,
  Depth::Full)` returns `Ok(verified)`, `if !guard.reach.acts() { return
  Removed::ended(ItemOutcome::Skipped { reason: SkipReason::DryRun }); }` (the `verified` dropped), before `way` is
  chosen. `act_on`'s and `execute`'s docs say so. `rehearse` is unchanged (it never acts).
- `crates/cleaner/src/put_back.rs` — `put_back`: after the `state(item)` match and before `roots`, `if
  !cleaning.reach.acts() { return refused(SkipReason::DryRun); }`; its doc says so.
- `crates/cleaner/src/test_support.rs` — `Rig::dry_run(toml: &str) -> Self`: the tree of `toml` **without** a sandbox
  marker (`Setup::new_without_sandbox`), the same fakes as `Rig::new`, the cleaner made with
  `Reach::DryRun(DryRunToken::new())`; `Rig::new`'s body shared through a private `Rig::with(setup, reach)` so the two
  differ only in those two lines.
- `crates/cleaner/tests/reach.rs` — the guard checks two tokens:
  - `sources::offenders(root: &Path, rule: &Rule) -> Vec<String>` with `pub(crate) struct Rule { token: &'static str,
    makers: &'static [&'static str], namers: &'static [&'static str] }`;
  - `REAL_DISK`: as today (`RealDisk` + `Token`; makers `crates/cleaner/src/reach.rs`, `app/tauri/src/compose.rs`;
    namers `crates/cleaner/src/lib.rs`);
  - `DRY_RUN`: `concat!("DryRun", "Token")`; makers `crates/cleaner/src/reach.rs`, `app/tauri/src/compose.rs`,
    `crates/cleaner/src/test_support.rs`, `crates/cleaner/tests/never.rs`; namers `crates/cleaner/src/lib.rs`;
  - the module doc: "The real disk is reached only from the app's composition, and the dry run only from it and the
    cleaner's own tests".
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (`UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test
  bindings`): `SkipCode` gains `"dry-run"`, `SkipReason` its `{ "kind": "dry-run" }`; review the diff. Nothing else of
  the UI changes: it shows a skip by its core text.
- `crates/cleaner/tests/never.rs` — `Bench::dry_run(tree: Tree) -> Self` (`Self::within(tree,
  Reach::DryRun(DryRunToken::new()))`, no marker written); one property test (below).

## Produces

`Reach::DryRun`, `DryRunToken::new()` (debug builds), `SkipReason::DryRun` / `SkipCode::DryRun` with its text, for
01/02 (the composition) and 02 (the record's `would-*` outcomes).

## Checks

- `crates/core/src/cleanup/outcome.rs`: the existing `every_skip_reason_has_its_code` passes with the sample's new
  reason; `mod positive` `the_dry_run_skip_has_its_code_and_text_and_no_instruction` (`code()` is `SkipCode::DryRun`,
  `text()` key `core.cleanup.skip.dry-run`, `how()` `None`, serialised `{ "kind": "dry-run" }`). `crates/core/src/texts.rs`'s
  dictionary test passes (every code's key is in `en.json`).
- `crates/store/src/journal/records.rs` (or the journal's tests beside it): `mod positive`
  `a_dry_run_skip_round_trips_through_the_journal` (an item recorded `Skipped { reason: SkipCode::DryRun }` reads back
  the same).
- `crates/cleaner/src/reach.rs`, negatives first:
  - `mod negative`: `the_dry_run_reach_never_acts` (`Reach::DryRun(..).acts()` is false; `Sandbox` and a
    `SandboxFolder` act);
  - `mod positive`: `the_dry_run_reach_admits_paths_tools_and_unplaced_paths_without_a_marker` (a tree with no marker:
    `admits` true for an item below the root, `admits_tools` and `admits_unplaced` true).
- `crates/cleaner/src/pipeline.rs`, on the module's `rig()` tree built with `Rig::dry_run`:
  - `mod negative`:
    - `a_dry_run_moves_empties_deletes_and_runs_nothing`: a file, a folder, `Empty` of a folder and the tool, each with
      `Method::Trash` and again with `Method::Permanent`: the tree's `listing` is the same before and after, the fake
      Trash moved nothing (`rig.trash.moved()` empty), the runner was never called (`rig.runner.calls()` empty), and
      every outcome is `Skipped { reason: SkipReason::DryRun }`;
    - `a_dry_run_still_skips_what_a_check_refuses`: the link `home/cache/to-outside` (its own reason, as the sandbox
      rig's test of links gives), an ignored path (the cleanup context's protection with the path ignored: `Ignored`),
      a file held open (`with_open_files`: `InUse`), a path outside the roots (`OutsideRoots`) — none ends `DryRun`;
    - `a_cancelled_dry_run_leaves_the_rest_not_reached`: the cancel token set before the second item: `DryRun`,
      `NotReached`, `NotReached`; `Ended::Cancelled`;
  - `mod positive`: `a_dry_run_rehearses_and_ends_every_item_dry_run_without_a_marker`: rehearse `Depth::Quick` and
    `Depth::Full` pass every path item and the tool (`Rehearsal::Passed`), where `Reach::Sandbox` on the same
    unmarked tree gives `NotInSandbox`; executing gives `DryRun` for each, in plan order, and `Ended::Completed`.
- `crates/cleaner/src/put_back.rs`: `mod negative` `a_dry_run_puts_nothing_back`: an item moved into the dry-run rig's
  fake Trash through the port itself (`rig.trash.move_to_trash(path, &held)`), then `rig.cleaner.put_back(&trashed,
  &rig.cx())` answers `PutBackOutcome::Refused { reason: SkipCode::DryRun }`, the item stays in the Trash and its
  original place stays empty.
- `crates/cleaner/tests/reach.rs`:
  - `mod negative`: `the_token_appears_only_in_the_composition` (as today, with the `REAL_DISK` rule) and
    `the_dry_run_token_appears_only_in_the_composition_and_the_cleaner_s_tests` (planted in `crates/engine/src`,
    `modules/demo/src`, `crates/cleaner/src/planted.rs` and `app/tauri/src/dev_runs/planted.rs`: those four are
    offenders; planted in `app/tauri/src/compose.rs` and `crates/cleaner/src/test_support.rs`: not);
  - `mod positive`: `the_workspace_makes_the_token_only_in_the_composition` and
    `the_workspace_makes_the_dry_run_token_only_in_the_composition_and_the_cleaner_s_tests`.
- `crates/cleaner/tests/never.rs`: `a_dry_run_changes_nothing_in_any_tree` (the property test of the file's
  strategies: random trees with links in, out and up, random picks of items, removals, emptyings and tools, with
  `Method::Trash` and `Method::Permanent`): the listing before equals the listing after, the Trash moved nothing, the
  runner ran nothing, and no outcome is `Moved`, `Emptied`, `Deleted` or `ToolDone`.
- `cargo nextest run -p fresher-core -p fresher-store -p fresher-cleaner`; `cargo clippy --workspace --all-targets --
  -D warnings`; `cargo clippy -p fresher-cleaner --release --all-targets -- -D warnings` (the cfg'd arms build without
  `debug_assertions`); `cargo fmt --check`. `export CARGO_TARGET_DIR=…/fresher/target` for every cargo command.
- `cargo nextest run -p fresher-app --test bindings` (the regenerated file is current); `npm run lint:ts`.

## Conclusions

- Built as written. `act_on` stops a tool item in the arm of `verify_tool`'s `Ok(())` and a path item right after
  `verify(.., Depth::Full)`; put back refuses before the roots are read. A release build has no variant, token, arm
  or branch: `cargo clippy -p fresher-cleaner --release --all-targets` builds and the release tests pass.
- `crates/sandbox/src/expected.rs` (not named by the step) matches every `SkipCode` with no wildcard: it gains
  `dry-run` in `code_name` and in its read-back test.
- Every dry-run test and helper is `#[cfg(debug_assertions)]`, so the crate's tests also build in release.
- The check-refusal test: the link item is a path through `home/cache/to-outside` (`ThroughLink`); the link itself
  passes every check and ends `DryRun`, as `a_dry_run_moves_empties_deletes_and_runs_nothing` checks with it as a
  fifth item. The file in use is held after the plan is made (with `with_open_files` before it, the rehearsal skips
  it and the item ends `CheckFailed` for its missing identity); a forced tool (`ForcedTool`) is checked too; the
  path outside the roots is executed with the current roots narrowed to `home`.
- `never.rs`'s `Bench::dry_run` blanks the marker `Tree::build` writes (`std::fs::remove_file` is disallowed outside
  the deletion point; a wrong signature does not count) and allows the tool, so tools reach the barrier;
  `the_dry_run_bench_ends_what_may_go_dry_run` proves the bench reaches it.
- Pre-flight amendments: `Rig::dry_run` and `Bench::dry_run` allow the test tool, so tools reach the stop (C2);
  `acts()` and both `if !acts()` branches (and the tool's guarded arm) are `#[cfg(debug_assertions)]`, so a release
  build has no such branch (C3); the full rehearsal is no longer called "the dry run" (`removal.md`; the test
  `a_full_rehearsal_reports_what_would_happen_with_the_folder_walk`) (C9).
- `cargo clippy --release --workspace --all-targets` fails on `fresher-app`'s `UnknownAccess` (never constructed in
  a release build) already on the plan's base commit; every other crate passes it.
- Proven against the barrier: with `acts()` answering `true` for `DryRun`, the reach, pipeline, put-back and both
  `never.rs` dry-run tests fail.
