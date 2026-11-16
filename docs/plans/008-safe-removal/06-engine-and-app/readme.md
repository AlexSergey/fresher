# 008/06 — Engine and app

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** engine, app, e2e

The engine turns a selection into a plan and a confirmation into a cleanup: the plan builder and the pending plan with
its expiry, the `Clean` operation with the journal written ahead and the events batched, the history and put back.
Then the app wires the real adapters (`DiskCleaner` with its reach, `MacTrash`, `MacOpenFiles`, the protection list,
`FileCleanupJournal`, `MacVolumes`), exposes the commands, and guards the window's close; the core e2e proves the whole
path on a fixture home, with a dev-only `trial` module that gives a development build something real to clean in a
sandbox folder. The design: [`../00-design.md`](../00-design.md), sections 1, 3, 4, 8, 9 and 10.

Steps 01–03 depend on 02 only (they run on `FakeCleaner`, `MemoryCleanupJournal` and `FakeVolumes`) and run in
parallel with 03, 04 and 05. Step 04 needs 03, 04 and 05; step 05 may run beside 01–04 (a module crate of its own,
one line in the composition); step 06 needs 04 and 05.

## Steps

| #   | Step                                                          | Status   | Shared files                                                                               |
| --- | ------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------ |
| 01  | [The plan builder and the pending plan](./01-plan-builder.md) | ✔️ done | `crates/engine/src/{engine,lib}.rs`                                                        |
| 02  | [The clean operation](./02-clean-operation.md)                | ✔️ done | `crates/engine/src/{engine,state,results,session}.rs`                                      |
| 03  | [History and put back](./03-history-and-put-back.md)          | ✔️ done | `crates/engine/src/engine.rs`                                                              |
| 04  | [The app wired](./04-app-wiring.md)                           | ✔️ done    | `app/tauri/`, `app/ui/src/ipc/bindings.generated.ts`, `docs/hacks/README.md`, `Cargo.lock` |
| 05  | [The trial module](./05-trial-module.md)                      | ✔️ done    | `app/tauri/{Cargo.toml,src/compose.rs}`, `fixtures/`, `docs/hacks/README.md`               |
| 06  | [The core end to end](./06-core-e2e.md)                       | ✔️ done    | `crates/e2e/`, `fixtures/`                                                                 |

01–04 in order, 05 beside them, 06 last.

## Conclusions

The six steps are built, each with its conclusions: the plan builder and the pending plan (01), the `Clean` operation
with the journal written ahead (02), the history and put back (03), the app wired to the real adapters with the close
guard (04), the dev-only `trial` module (05) and the core e2e (06), which proves selection → plan → confirmation →
cleaner → journal → history → put back on files, with the refusals, a cancel, a replaced item and a permanent cleanup.

- The engine's cleanup ports are a struct of their own (`Engine::with_cleanup(registry, ports, cleanup)`); an engine
  built with `Engine::new` refuses every cleanup. A development build and every test reach only a sandbox folder.
- The trial module claims a folder whole and only right next to the sandbox marker, so the e2e builds its nested
  (`Covered`) case with a small module of its own.
- Found by the e2e (06) and fixed in the engine (`75c1ddd`): a `Covered` finding now leaves the results with the item
  that held it; a plan of no item is still shown but its confirmation is refused (`NothingToClean`, nothing
  journalled); a second put back of an item answers `PutBack` without a new record, so its detail still reads back.
