# 037/01 — Numbers

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, system, testing, engine, e2e, app, app/tauri, app/ui

The core's half of the plan: the Trash view and its refusal, the two ports (`SystemUi::open_folder`,
`FinderSettings`) with their macOS readers and fakes, the engine's `trash` and `open_trash`, and the two commands
with their typed IPC calls. The design: [`../00-design.md`](../00-design.md), sections 1 to 4.

## Steps

| #   | Step                                                   | Status      | Shared files                                                                                       |
| --- | ------------------------------------------------------ | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The types and the ports](./01-types-and-ports.md)     | 🔨 in progress | `crates/core/src/{ports.rs,error.rs,lib.rs}`, `crates/core/locales/en.json`, `crates/testing/src/` |
| 02  | [The engine's Trash view](./02-engine.md)              | 🔨 in progress | `crates/engine/src/`, every `CleanupPorts` site                                                    |
| 03  | [The commands and the IPC](./03-commands.md)           | 🔨 in progress | `app/tauri/`, `app/ui/src/ipc/`                                                                    |

In order: 01, then 02, then 03; each leaves the workspace green.

## Conclusions

All three steps are built as designed, with the changes written in each step (the scanner's clock, the known-Trash
rule, `UnknownTrash`'s field). The core-change rule holds: `git diff 6c2324d3 -- crates/cleaner crates/scanner
crates/store crates/rules modules` is empty (the base is the merge base with main, since plans 033, 034 and 031 added
modules after 97d9b314). Awaiting the user's walk.
