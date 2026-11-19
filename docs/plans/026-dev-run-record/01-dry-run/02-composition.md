# 026/01/02 — The development composition and the docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app, app/tauri, docs

The app's own composition makes the dry run real: a development build outside a sandbox gets `Reach::DryRun` and a
Trash that refuses (`DryRunTrash`), a release build keeps `Reach::RealDisk` and `MacTrash`, sandbox mode is unchanged.
The shell's tests prove the development engine moves nothing through the real composition and through the commands,
and the docs of removal lose the manual way of cleaning a marked folder in `npm run dev`. The design:
[`../00-design.md`](../00-design.md), section 1.4 (agent's calls 2 and 4).

## Files

- `app/tauri/src/dry_run.rs` (new; `#[cfg(debug_assertions)] mod dry_run;` in `lib.rs`) — the module doc: "The Trash
  of a development build outside a sandbox: it refuses every call and touches nothing. The dry run never reaches it
  (the cleaner's `Reach::DryRun` stops every act); it is the second wall." `#[derive(Debug)] pub(crate) struct
  DryRunTrash;` implementing `fresher_core::Trash`: `move_to_trash(&self, path, _held)` and `put_back(&self, landed,
  original)` each `tracing::error!(path = %…display(), "a dry run reached the Trash")` and return
  `Err(TrashError::NotAllowed)`.
- `app/tauri/src/compose.rs` —
  - `reach()` in three forms: `#[cfg(not(any(debug_assertions, test)))]` `Reach::RealDisk(RealDiskToken::new())`
    (unchanged); `#[cfg(debug_assertions)]` `pub(crate) const fn reach() -> Reach { Reach::DryRun(DryRunToken::new()) }`
    documented "How far the cleaner of a development build reaches: the real disk, checked and never acted on
    (`docs/architecture/removal.md`)"; `#[cfg(all(test, not(debug_assertions)))]` `Reach::Sandbox` (a release-mode
    test never reaches the real disk). The imports follow the same cfgs (`DryRunToken` under `debug_assertions`).
  - `cleanup(home, user_temp, folder, ports, volumes, reach: Reach)`: the reach is passed in; `normal` passes
    `reach()`; the doc says "this build's cleaner" → "a cleaner with `reach`".
  - `normal`: the cleaner's `trash` is `Arc::new(crate::dry_run::DryRunTrash)` under `debug_assertions` and
    `Arc::new(MacTrash::new())` otherwise (the `MacTrash` import cfg'd `not(debug_assertions)`; sandbox mode imports
    its own). The comment "The system's Trash goes to the cleaner and to nothing else." gains "; a development build's
    cleaner gets a Trash that refuses".
- `app/tauri/src/testing.rs` — `Shell` chooses its cleaner: the private `Shell::scanning(home, held, reach, trash)`
  and `Shell::with_walk(walk, home, held, access, reach, trash)` take the reach and the Trash; `Shell::new`,
  `with_access`, `held`, `cleaning` and `cleaning_held` pass `Reach::Sandbox` and the `FakeTrash` of today (so every
  existing test keeps its meaning); new:
  - `#[cfg(debug_assertions)] pub(crate) fn dry_run(home: &Path) -> Self`: `crate::compose::reach()` and the
    `FakeTrash` (so a test can see it stays empty);
  - `#[cfg(debug_assertions)] pub(crate) fn refusing_trash(home: &Path) -> Self`: `Reach::Sandbox` and
    `crate::dry_run::DryRunTrash` (the second wall alone).
- `app/tauri/src/commands.rs` — tests only: `a_debug_build_cleans_nothing_outside_a_sandbox` is renamed
  `the_sandbox_reach_cleans_nothing_outside_a_sandbox` (its body unchanged); the new tests below.
- `app/tauri/src/lib.rs` — `#[cfg(debug_assertions)] mod dry_run;`.
- `docs/architecture/README.md` — "Rules of the layout": the bullet "A development build and every test get
  `Reach::Sandbox`…" becomes: a test gets `Reach::Sandbox` (or a sandbox folder); a development build outside a
  sandbox gets `Reach::DryRun` — every check on the real disk, no act, each item ends "a dry run" — and `DryRunTrash`,
  which refuses; only a release build's composition makes the token for the real disk. The Ports table's `Trash` row:
  "in a development build outside a sandbox `DryRunTrash` (refuses every call)".
- `docs/architecture/removal.md` —
  - "The crate and its one entry point" and "Reach and the sandbox": the fourth reach, `Reach::DryRun(DryRunToken)`,
    debug builds only: admits what `RealDisk` admits; `act_on` ends each item `SkipReason::DryRun` after its checks and
    before its act, for every action and method; put back is refused `dry-run` before any port; `DryRunToken::new` is
    made only by `app/tauri/src/compose.rs` and the cleaner's tests (`tests/reach.rs`).
  - The paragraph "**To clean in a development build**…" is rewritten: use sandbox mode; `npm run dev` on the real Mac
    makes every cleanup a dry run, a marked folder included (the manual way of a marked folder as a scan root is
    gone).
  - "Put back": "refused `dry-run` in a development build outside a sandbox".
  - The threat model: the row "A development accident" becomes: defence "`Reach::DryRun` in a development build (every
    check, no act; `DryRunTrash` behind it), `Reach::Sandbox` or a sandbox folder in tests and sandbox mode; the
    real-disk token only in the release composition"; proved by "`reach.rs` and `tests/reach.rs`, `pipeline.rs` (a dry
    run acts on nothing), `tests/never.rs` (`a_dry_run_changes_nothing_in_any_tree`), `app/tauri/src/compose.rs`
    (`the_development_engine_moves_nothing_on_the_real_disk`)".
- `docs/restrictions/deletion-safety.md` — "Never follow symlinks out…", its last sentences: after the sandbox folder,
  "A development build outside a sandbox has a fourth, the dry run (`Reach::DryRun`, debug builds only): every check
  runs on the real disk and no item is acted on; it ends 'a dry run'. Tests: `crates/cleaner/src/pipeline.rs`,
  `crates/cleaner/tests/never.rs`, `app/tauri/src/compose.rs`."

## Produces

`compose::reach()` (`pub(crate)`, the dry run in debug builds), `compose::cleanup(…, reach)`, `DryRunTrash`,
`Shell::dry_run`, for 02/02 (the recorder's tests clean through `Shell::dry_run`).

## Checks

- `app/tauri/src/dry_run.rs`, `mod negative`:
  - `the_dry_run_trash_refuses_to_move_and_touches_nothing` (a file in a `tempfile::tempdir()`, opened, handed to
    `move_to_trash`: `Err(TrashError::NotAllowed)`, the file is still there with its bytes, the folder lists only it);
  - `the_dry_run_trash_refuses_to_put_back` (a "landed" file and an absent original: `Err(NotAllowed)`, the landed
    file stays, the original is not made).
- `app/tauri/src/compose.rs`:
  - `mod negative`: `#[cfg(debug_assertions)] the_development_engine_moves_nothing_on_the_real_disk`: `engine(folder,
    home)` (the real composition: the real scanner, `MacOpenFiles`, `DryRunTrash`, `reach()`) on `trial_home(true)` —
    the trial module's folders beside a **valid** marker, where `Reach::Sandbox` would move them — with
    `core.scan.roots` set to the tree's root; a full scan, `prepare_cleanup(&caches())` (its count is above zero),
    `confirm_cleanup(&echo(&view))`, wait for `CleanupFinished`: `sandbox/fresher-trial-cache/blob.bin` and every
    other entry of the tree are still there (a recursive listing of the tree before equals the one after), and `engine.cleanup(id)` lists every item
    `RecordedOutcome::Skipped { reason: SkipCode::DryRun }`;
  - `mod positive`: `#[cfg(debug_assertions)] a_development_build_reaches_the_real_disk_without_acting`
    (`matches!(reach(), Reach::DryRun(_))`).
- `app/tauri/src/commands.rs`, through the commands:
  - `mod negative`:
    - `the_sandbox_reach_cleans_nothing_outside_a_sandbox` (renamed, unchanged);
    - `#[cfg(debug_assertions)] a_dry_run_through_the_commands_moves_nothing`: `Shell::dry_run` on
      `trial_home(true)`; scan, `prepare_cleanup`, `confirm_cleanup`, wait (`is_cleanup_finished`): the blob is there,
      `shell.trash()` does not exist, `get_cleanup` shows every item skipped `dry-run`, its summary's `moved` and
      `deleted` sizes are zero; `put_back` of the whole cleanup answers `PutBackOutcome::NotTrashed` for every item and
      moves nothing;
    - `#[cfg(debug_assertions)] the_refusing_trash_alone_moves_nothing_in_a_marked_sandbox`: `Shell::refusing_trash`
      on `trial_home(true)`; the same flow: the blob is there and every item that reached the Trash ended `Failed` (the
      Trash refused), none `Moved`;
  - `mod positive`: `#[cfg(debug_assertions)] a_dry_run_plan_holds_what_the_sandbox_reach_leaves_out`: `Shell::dry_run`
    on `trial_home(false)` (no valid marker): the plan of both trial categories has two items and no left-out line,
    where `the_sandbox_reach_cleans_nothing_outside_a_sandbox` has none and two `NotInSandbox`.
- `cargo nextest run -p fresher-app --lib`; `npm run sandbox:test` (sandbox mode and its real cleanups unchanged);
  `cargo clippy --workspace --all-targets -- -D warnings`; `cargo clippy -p fresher-app --release --all-targets -- -D
  warnings`; `cargo nextest run --release -p fresher-app --lib` (the release-mode tests, `reach()` is `Sandbox` there);
  `cargo fmt --check`; `npx prettier --check` on the changed docs. `export CARGO_TARGET_DIR=…/fresher/target`.
- `grep -rn "manual way\|fresher-sandbox\`, never the home" docs/architecture/removal.md` finds nothing of the old
  paragraph.

## Conclusions

- Built as written. `Shell::scanning` and `with_walk` take the Trash as a small enum (`ShellTrash::{Fake,
  Refusing}`, preflight), never a ready Trash: the fake's folder is made inside `with_walk`.
- The refusing-Trash test asserts what preflight C10 asks: the tree's recursive listing is unchanged, the fake Trash
  folder does not exist, no item is `Moved` or `Deleted`, the cache item is `Failed`, and an `Emptied` item has
  `moved == 0`. The commands tests pick both trial categories, so the emptied folder is covered too.
- Put back of the whole dry-run cleanup with `items: null` answers no item at all (only items still in the Trash
  are tried), so the test names both items and gets `NotTrashed` for each.
- A shared helper `testing::listing` (debug builds only) is the recursive listing the tests compare.
- Not named by the step: `docs/hacks/README.md`'s entry "Development builds clean only in a sandbox folder" lost the
  manual way and says what a development build gets now; the docs say what a dry run cannot rehearse (an emptied
  folder's checks of its place and entries, the Trash's own `NoTrash`); `adapters.rs` is compiled only in debug
  builds and tests (`UnknownAccess` was unused in a release build, so the release Clippy failed); a comment on the
  `OutsideRoots` item of the cleaner's `a_dry_run_still_skips_what_a_check_refuses`.
- `cargo clippy --release --workspace --all-targets -- -D warnings` passes. `cargo nextest run --release -p
  fresher-app --lib`: the compose tests pass (`reach()` is `Sandbox` there); 13 command and shell tests fail in
  release as before this step, since they use the `demo` and `trial` modules, which a release build does not have.
