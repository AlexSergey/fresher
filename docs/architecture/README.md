# Architecture

> **Status:** the intended layout, created by plan [001](../plans/001-repository-setup/readme.md) (✅ approved). Details
> of each crate are designed in the plan that implements it.

## Layers

```
app/ui (React)  ──typed IPC──▶  app/tauri (commands)  ──▶  Rust core crates
                                                            │
        scanner ──▶ rules ──▶ findings ──▶ store (SQLite snapshots, diffs)
                                   │
                                   └──▶ cleaner (only from a confirmed plan)
```

| Crate             | Folder           | Responsibility                                                                         | May depend on            |
| ----------------- | ---------------- | -------------------------------------------------------------------------------------- | ------------------------ |
| `fresher-core`    | `crates/core`    | domain types: `Category`, `DangerLevel`, `Finding`, `ByteSize`, `CleanPlan`; no IO     | —                        |
| `fresher-scanner` | `crates/scanner` | fast parallel file system walk; logical and allocated sizes; hard links counted once   | core                     |
| `fresher-rules`   | `crates/rules`   | the rule catalogue: recognizes leftovers and turns scan results into findings          | core, scanner            |
| `fresher-store`   | `crates/store`   | the SQLite database: snapshots of every scan, growth between scans (watch mode)        | core                     |
| `fresher-cleaner` | `crates/cleaner` | the only code that deletes: executes a confirmed `CleanPlan`, Trash first              | core                     |
| `fresher-testing` | `crates/testing` | test helpers: fixture tree builder, fake clock, fake Trash, fake Docker CLI (dev only) | core                     |
| `fresher-e2e`     | `crates/e2e`     | end-to-end and golden tests of the core (tests only)                                   | all core crates, testing |
| `fresher-app`     | `app/tauri`      | the Tauri 2 shell: thin commands, events streamed to the UI                            | all core crates          |

Rules of the layout:

- The core crates know nothing about Tauri; the shell knows nothing about rules' internals.
- Only `fresher-cleaner` may call a deleting file system function (enforced by Clippy's `disallowed-methods`).
- External programs (the Docker CLI, `tmutil`) are called through traits, so tests use fakes.
- The UI gets everything it shows (names, descriptions, sizes, danger levels) from the core.

## Data

- The snapshot database lives in `~/Library/Application Support/<app id>/` — never inside a scanned root.
- Its schema and migrations are documented here when plan 002+ creates them.
