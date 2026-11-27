# 044/02 — The query, the setting and the commands

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, scanner, engine, modules,
testing, sandbox, app/tauri, app/ui (bindings only), docs

The folders left alone, from the tree to the IPC: the state's query, the size setting and the stale days, the
engine's list with its rule, `reveal_node`, and the sandbox's cases. The old Folders tree stays (plan 03 shows it
as the read-only tab Browse). The design: [`../00-design.md`](../00-design.md), sections 3 to 7 and 10;
agent's calls 5 to 19.

## Steps

| #   | Step                                                          | Status   | Shared files |
| --- | ------------------------------------------------------------- | -------- | ------------ |
| 01  | [`DiskState::left_alone`](./01-query.md)                      | ✔️ done | `crates/core/src/{disk_state.rs,lib.rs}`, `crates/scanner/src/state/`, `crates/testing/src/fixed_disk_state.rs`, `crates/engine/src/{disk_tests.rs,largest/largest_tests.rs}` |
| 02  | [The size setting and the stale days](./02-setting.md)        | ✔️ done    | `crates/core/src/settings/{core_schema.rs,days.rs}`, `crates/core/src/settings.rs`, `crates/core/locales/en.json`, `crates/engine/src/settings.rs`, `modules/stale-projects/src/lib.rs` |
| 03  | [The engine's Folders list](./03-engine-list.md)              | ✔️ done    | `crates/core/src/largest/`, `crates/engine/src/largest/`, `crates/engine/src/own_tests.rs` |
| 04  | [`get_largest_folders` and `reveal_node`](./04-commands.md)    | ✔️ done     | `crates/engine/src/reveal.rs`, `app/tauri/src/{largest_commands.rs,lib.rs,dev_runs/commands.rs}`, `app/tauri/build.rs`, `app/tauri/tests/bindings.rs`, `bindings.generated.ts` |
| 05  | [The sandbox's cases](./05-sandbox.md)                        | ✔️ done | `crates/sandbox/src/{largest.rs,case.rs,error.rs,expected.rs,stated_times.rs,lib.rs}`, `app/tauri/src/sandbox.rs`, `app/tauri/tests/sandboxes/{harness,largest}.rs`, `fixtures/sandboxes/largest-folders.toml` |

01, then 02, 03, 04 and 05, one at a time (03 needs 01 and 02; 04 and 05 need 03).

## Conclusions

To be written when step 05 is done.
