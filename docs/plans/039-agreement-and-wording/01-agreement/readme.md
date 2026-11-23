# 039/01 — The agreement

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 ·
**Modules:** core, engine, testing, sandbox, e2e, app/tauri, app/ui, docs

Part A. The five-line agreement: its text and version in the core, its acceptance in the settings file, the engine's
refusal of a cleanup without it, the commands, the sandbox and e2e guards, the sheet before the first cleanup, the
licence dialog and Settings → About. The design: [`../00-design.md`](../00-design.md), section 1.

Built after plans 035, 036 and 037 are on `main` (they are, on 740cf3c3); the branch is rebased onto `main` before
01/01 starts if `main` moved. Refreshed on 2026-11-23: the uninstall reaches the guard through `rehearse_and_keep`, the
engines that clean are listed by place, the two commands join the manifest, the capability and the dev-runs lists, and
the `cleanup` demo scenario accepts as a user would.

## Steps

| #   | Step                                                          | Status      | Shared files                                                                                               |
| --- | ------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------- |
| 01  | [The core and the engine](./01-core-and-engine.md)            | 🔨 awaiting the user's walk | `crates/core/src/{lib.rs,settings/,command.rs,error.rs,cleanup/outcome.rs,texts.rs}`, `crates/core/locales/en.json`, `crates/engine/src/{engine.rs,settings.rs,cleanup/,apps/uninstall_tests.rs,testing.rs}`, the engines that clean in `crates/e2e/tests/` and `app/tauri/`, `docs/restrictions/deletion-safety.md`, `docs/architecture/removal.md` |
| 02  | [The commands and the guards](./02-commands-and-guards.md)    | 🔨 awaiting the user's walk | `app/tauri/{build.rs,capabilities/default.json}`, `app/tauri/src/{commands.rs,lib.rs,testing.rs,dev_runs/commands.rs}`, `app/tauri/tests/`, `crates/sandbox/src/expected.rs`, `crates/e2e/tests/`, `app/ui/src/ipc/bindings.generated.ts`, `docs/testing.md` |
| 03  | [The sheet and About](./03-sheet-and-about.md)                | 🔨 awaiting the user's walk | `app/ui/src/{ipc,state,screens/app-frame.tsx,screens/settings}`, `app/ui/e2e/support/`, the mocks, the e2e specs, `fixtures/demos/scenarios/cleanup.toml`, `app/ui/src/locales/en.json` |

01, then 02, then 03.

## Conclusions

The agreement is built as designed (steps 01 to 03): the text and version in the core, the engine's refusal at prepare
and confirm, the commands, the sandbox harness's guard for every case file, the core e2e on files, the sheet, the
licence dialog and About. The final review found one real defect, fixed: accepting over a settings file that was not
read replaced it. Open for the user: the lawyer's look, the five lines, the toast after a refused acceptance (see
[the user's review](../../../deferred/user-review-2026-11-22.md)).
