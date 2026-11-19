# 026/02 — The recorder

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app, app/tauri, scripts, docs

A development build started by the launcher writes each run to a record pair under `NO_COMMIT/dev-runs/`, never
overwritten: the session, the settings, the access, every action the UI sent, every scan, every cleanup by module
group. Nothing of it is compiled in a release build, and nothing is written without `FRESHER_DEV_RUNS`. The design:
[`../00-design.md`](../00-design.md), sections 2 and 3 (agent's calls 6 to 11, 14, 15 and 17).

## Steps

| #   | Step                                                | Status      | Shared files                                                                                                                                  |
| --- | --------------------------------------------------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| 01  | [The record's files](./01-record-files.md)          | ✔️ done | `app/tauri/src/lib.rs` (the module line), `app/tauri/Cargo.toml`, `Cargo.lock`, `docs/dependencies.md`                                       |
| 02  | [The recorder in the shell](./02-recorder.md)       | ✔️ done | `app/tauri/src/{compose,lib}.rs`                                                                                                              |
| 03  | [The launcher and the docs](./03-launcher.md)       | ✔️ done | `scripts/sandbox-dev.ts` (its comment), `package.json`, `docs/testing.md`, `docs/restrictions/privacy.md`, `docs/architecture/README.md`, `docs/deferred/` |

01, then 02. 03 shares no file with 01 or 02 and runs beside them; its docs describe what 01 and 02 build, so it is
reviewed after 02.

## Conclusions

Not written yet.
