# 039/01/02 — The commands and the guards

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** app/tauri, sandbox, e2e, app/ui, docs

The two IPC commands and their bindings, and the proofs outside the engine: the sandbox autotest asserts the guard for
every case file, the core e2e proves it on a real file store, and every `EXPECTED.md` says the first cleanup asks. The
design: [`../00-design.md`](../00-design.md), sections 1 and 5; agent's call 9.

## Read first

01/01's step file and commit; `app/tauri/src/commands.rs` (the shape of a command and its tests),
`app/tauri/src/lib.rs` (the invoke handler's list), `app/tauri/build.rs` (`COMMANDS`, the manifest's allowlist, in
the handler's order), `app/tauri/capabilities/default.json` (one `allow-<command>` grant each),
`app/tauri/tests/config.rs` (holds the handler, the manifest and the capability equal),
`app/tauri/src/dev_runs/commands.rs` (`ACTIONS` and `READS`: a test in `dev_runs/shell_tests.rs` holds every command
in one of them), `app/tauri/tests/bindings.rs` (`declarations()`; regenerate with
`UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings`), `app/tauri/tests/sandboxes/harness.rs` (`run`,
`clean`) and `app/tauri/tests/sandboxes/uninstall.rs` (`check_uninstalls`, `uninstall_and_put_back`),
`crates/sandbox/src/expected.rs` (`checklist`: the header of `EXPECTED.md`, and the demo world's test that asserts it),
`crates/e2e/tests/cleanup.rs`, `crates/e2e/tests/support/` and `crates/e2e/tests/uninstall_home/` (how a core e2e
builds an engine on a temporary data folder with `FileSettingsStore`).

## Files

- `app/tauri/src/commands.rs` — `get_agreement(engine) -> Result<AgreementView, Refusal>` and
  `accept_agreement(engine, version: u32) -> Result<AgreementView, Refusal>`, thin calls of the engine; registered in
  `lib.rs`'s `generate_handler!`, in `build.rs`'s `COMMANDS` at the same place, granted in
  `capabilities/default.json` (`allow-get-agreement`, `allow-accept-agreement`), and classified in
  `dev_runs/commands.rs`: `accept_agreement` in `ACTIONS` (it writes the settings file; the array becomes
  `[&str; 24]`), `get_agreement` in `READS` (`[&str; 20]`). Tests: negative — "accept_agreement of another version is refused" (the refusal's text is
  `core.error.agreement-changed`); "prepare_cleanup before the agreement is refused with the blocked text"
  (`core.cleanup.blocked.agreement`); positive — "get_agreement answers the five lines and no acceptance";
  "accept_agreement then prepare and confirm cleans a sandbox through the commands".
- `app/tauri/tests/bindings.rs` — `AgreementView` and `Acceptance` in `declarations()`; `bindings.generated.ts`
  regenerated and reviewed (also `BlockedReason`, `CommandError` and `SettingGroup` gain their new members).
- `app/tauri/tests/sandboxes/harness.rs` — a helper `check_agreement_guard(name, engine, attempt)` that asserts
  `!engine.agreement()?.is_accepted()` (a fresh sandbox's data folder holds no acceptance), runs `attempt` (a prepare)
  and asserts its refusal is exactly `CommandError::CleaningBlocked { reason: BlockedReason::Agreement }` and that a
  `confirm_cleanup` of any echo then answers `PlanGone` (no plan is pending), then calls
  `engine.accept_agreement(AGREEMENT_VERSION)`, as a user would. A failure names the case file ("the cleanup was not
  refused before the agreement"). It runs once per file, at the first prepare of that file:
  - in `run`, just before `clean`, when the file has a positive with `[case.expect]` (a Largest pick included): the
    attempt is `prepare_cleanup` of that selection;
  - otherwise in `crate::uninstall::check_uninstalls` (`sandboxes/uninstall.rs`), before its first
    `uninstall_and_put_back`, for a file whose only positives are uninstalls (a positive without `[case.expect]`): the
    attempt is `prepare_uninstall` of that app and choice;
  - a file with no positive never meets it (nothing is prepared) and stays unaccepted.
  Replace 01/01's plain acceptance in `World::engine` with this.
- `app/tauri/tests/sandboxes.rs` — a negative of the harness beside the existing ones, only if `run` can be given a
  data folder that already holds the acceptance without a new production hook: "a sandbox that starts accepted fails
  the agreement check", naming the file. Otherwise no test here; the by-hand mutation under "Checks" stands for it.
- `fixtures/sandboxes/cleanup-guards.toml` — a comment under its header: the agreement's guard is asserted by the
  harness for every case file (no case of its own; agent's call 9).
- `crates/sandbox/src/expected.rs` — the header `checklist` writes for every `EXPECTED.md` gains one plain sentence
  after its first paragraph, not a checkbox (so no line index of the existing tests moves): "Before the first cleanup,
  a fresh sandbox asks you to accept the short agreement: Not now starts nothing; I understand goes on to the
  confirmation." A new test asserts the sentence; `a_demo_world_lists_its_scenery_cases_and_checks_nothing`, which
  asserts the whole header, gains it too.
- `crates/e2e/tests/agreement.rs` (new) with its engine on a temporary data folder and `FileSettingsStore`, on the
  trial or demo fixture the other cleanup e2e tests use. Negative first: "a cleanup before the agreement is refused and
  nothing moves" (prepare refused; the fixture tree unchanged); "an acceptance of another version is refused and not
  saved" (the settings file on disk holds no agreement value); the confirm-time refusal is the engine's test of 01/01,
  not repeated here; "an uninstall before the agreement is refused and nothing moves" (on `uninstall_home/`: the
  bundle and its files stay); positive: "the acceptance survives
  a restart" (accept, drop the engine, open a new one on the same data folder: accepted, the cleanup runs to the
  Trash); "a reset of the settings keeps the acceptance"; "put back works without the acceptance" (a cleanup done
  while accepted, then a new engine whose settings file lost the acceptance puts the item back).
- `docs/testing.md` — "Sandboxes": the harness asserts the agreement's guard for every case file before it accepts;
  the core e2e list gains `agreement.rs`.

## Checks

- `CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-app -p fresher-e2e -p fresher-sandbox`;
  `npm run sandbox:test`; `cargo clippy --workspace --all-targets -- -D warnings`.
- `npm run lint:ts` and `npm run test:ui` with the regenerated bindings (a UI `switch` over `SettingGroup`,
  `BlockedReason` or `CommandError` that is exhaustive gets its new member; `CoreRowsGroup` in
  `screens/settings/settings-groups.ts` excludes `'agreement'` if 01/01 did not).
- `cargo nextest run -p fresher-app --test config` (handler, manifest and capability agree) and the `dev_runs`
  classification test.
- By hand, once, written in the conclusions: remove the harness's acceptance line and see every case file with a
  positive fail; restore it.

## Conclusions

- The commands are thin calls of the engine, registered in the handler, the manifest, the capability and the
  `dev_runs` classification. The shell's test helper accepts the agreement in every constructor but
  `Shell::cleaning_unaccepted`, which the four command tests use.
- The harness's guard (`check_agreement_guard`) runs before `clean` when the file has a positive with
  `[case.expect]`, else at the first chosen uninstall; a file with no positive stays unaccepted. The negative "a
  sandbox that starts accepted fails the agreement check" needs no production hook: the case file's `[settings]`
  write `core.agreement.version` into the sandbox's data folder before the app starts.
- By hand: with the harness's acceptance line removed, every case file with a positive failed (26 case files,
  uninstall-only `uninstall.toml` among them, and the four harness tests that clean); the files with no positive
  passed. The line was restored.
- The core e2e `agreement.rs` loads both shared homes (the cleanup home and the uninstall sandbox); they load the same
  support helpers, so its `flow` module carries `#[expect(clippy::duplicate_mod)]`. A home that starts unaccepted is a
  `Setup { accept: false }` of either home; the uninstall home's default engine accepts first, so the unaccepted home
  saves an empty settings file through the store before its own engine starts.
