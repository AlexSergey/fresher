# 002/07/02 — Docs and version

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** docs, repo

## Files

- [`docs/goal.md`](../../../goal.md) §5 — "The app keeps its own state in small versioned files in its own folder"
  instead of a SQLite database; snapshots become "the last two scans" (plan 004) and history points (plan 010).
- [`docs/rules.md`](../../../rules.md) §6 — "the snapshot database" → "the scan cache and settings".
- [`docs/architecture/README.md`](../../../architecture/README.md) — status note removed; the layers diagram (UI →
  shell → engine → modules over one walk → events; store; cleaner); the crate table with `engine`, `macos`, `modules/*`
  and the "may depend on" column of the design; the ports table; "Data": the data folder, `settings.json`, the version
  policy, the footprint budget; links to `modules.md` (06/04).
- [`docs/restrictions/privacy.md`](../../../restrictions/privacy.md) "Local data only" — enforced by `fresher-store` and
  the incognito webview; [`docs/restrictions/repository.md`](../../../restrictions/repository.md) — `bindings.generated.ts`
  named among the generated files.
- [`docs/testing.md`](../../../testing.md) — the module harness (`NaiveWalk`, `run_module`, the contract), the bindings
  check, `fixtures/golden/demo.json`.
- [`CLAUDE.md`](../../../../CLAUDE.md) — the repository map gets `modules/`; the architecture line names the engine.
- [`docs/ideas/architecture/crates.md`](../../../ideas/architecture/crates.md) — `rusqlite` struck through with the
  reason (files instead, plan 002); `ts-rs` and `tauri-plugin-single-instance` added as taken.
- Version 0.1.0: `[workspace.package] version` in `Cargo.toml`, `app/tauri/tauri.conf.json`, the root and
  `app/ui` `package.json` (`npm install` updates `package-lock.json`; `cargo metadata` updates `Cargo.lock`).
- [`../readme.md`](../readme.md) — the plan's conclusions; [`../../README.md`](../../README.md) — the status of 002.

## Checks

- `grep -ri sqlite docs CLAUDE.md crates` finds only history: the plans, the ideas, the staged-files guard.
- The crate table in `docs/architecture/README.md` matches the guard's constant (compared by hand, noted in the
  conclusions).
- `npm run pre-push` passes on the branch's last commit; `cargo clean` at the end.

## Notes from the implementation

- `grep -ril sqlite docs CLAUDE.md crates` finds only history: the plans, the ideas, and `docs/quality.md`'s row of the
  staged-files guard (which refuses `*.sqlite` files). The root `README.md` named "SQLite snapshots" too and was fixed.
- The crate table in `docs/architecture/README.md` was compared by hand with `TABLE` and `MODULE_ALLOWED` of
  `scripts/check-crate-deps.ts`: the same eleven rows (`fresher-module-<id>` stands for `modules/*`).
- `CLAUDE.md`: the `crates/` row names the engine and the store instead of the "snapshot store", and `modules/` has
  its row; nothing else there changed.
- `docs/restrictions/privacy.md` names `fresher-store` and the incognito webview as what enforces "Local data only";
  `docs/testing.md` gained the bindings check, the `demo` e2e and the `UPDATE_BINDINGS` command.
- `docs/categories/README.md` needed no change (06/04 already described the contract and the `demo` module).
- The e2e mocks keep answering `appVersion` with `0.0.1`: it is the mocked core's answer, not the app's version, and
  the screenshots show it.
- Before this step, two calls of the 05 conclusions were settled (agent's calls 28 and 29 of the design): the core sends
  a refusal's text, and the IPC types serialize their multi-word fields in camelCase.
