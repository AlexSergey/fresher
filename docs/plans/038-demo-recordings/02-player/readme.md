# 038/02 — The player

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, app, app/tauri, app/ui, fixtures

The scenario format and the generator's `demo` command; the shell's demo mode (`FRESHER_DEMO`, the window, two
debug-only commands, the two files for the recorder); the UI's player (a lazy debug-only chunk, the finder, the
steps, the drawn pointer, the private-text check). The design: [`../00-design.md`](../00-design.md), section 2.

## Steps

| #   | Step                                                        | Status      | Shared files                                                                                                       |
| --- | ----------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------ |
| 01  | [The scenario and the `demo` command](./01-scenario.md)     | 🔨 in progress | `crates/sandbox/{Cargo.toml,src/main.rs,src/lib.rs}`, `Cargo.lock`, `docs/dependencies.md`                         |
| 02  | [The shell's demo mode](./02-shell.md)                      | 🔨 in progress | `app/tauri/src/{lib,compose}.rs`, `app/tauri/src/dev_runs/commands.rs`, `app/tauri/tests/bindings.rs`, `app/ui/src/ipc/` |
| 03  | [The UI's player](./03-ui-player.md)                        | 🔨 in progress | `app/ui/src/main.tsx`, `app/ui/src/product/build-banner/build-banner.module.css`                                   |

In order: 01, then 02, then 03.

## Conclusions

The scenario format and the `demo` command, the shell's demo mode (`demo_play`, `demo_finished`, two files at the
sandbox's root) and the UI's player are built; the capabilities and the app manifest list the two commands in every
build (Tauri checks them at build time), the handler only in debug builds. UI: 1276 tests passing, coverage 98.5 % of
lines; the release bundle has no `data-demo-player`. See each step.
