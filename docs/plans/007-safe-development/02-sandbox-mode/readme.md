# 007/02 — Sandbox mode

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** core, cleaner, app, app/ui, repo

A development build started on a sandbox folder lives inside it: its home and known folders, its data folder, its
Trash; the cleaner reaches only that folder, and both the real Mac's and the sandbox's protection lists apply. A banner
always says which world the window is in. The design: [`../00-design.md`](../00-design.md), section 4.

01 runs in parallel with plan 01 (no shared file). 02 needs 01/03 (the layout) and 01 of this plan; 03 and 04 then run
in parallel with plan 03.

## Steps

| #   | Step                                                     | Status   | Shared files                                                                                                  |
| --- | -------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------- |
| 01  | [The reach of a sandbox folder](./01-sandbox-reach.md)   | ✅ approved | `crates/core/src/cleanup/protection.rs`, `crates/cleaner/src/{reach,lib}.rs`, `crates/cleaner/tests/never.rs` |
| 02  | [The sandbox composition](./02-composition.md)           | ✅ approved | `app/tauri/src/{compose,lib}.rs`, `app/tauri/Cargo.toml`, `Cargo.lock`, `docs/hacks/README.md`                |
| 03  | [The banner](./03-banner.md)                             | ✅ approved | `app/tauri/src/commands.rs`, `app/ui/src/ipc/`, `app/ui/src/screens/app-frame*`, `app/ui/e2e/`                |
| 04  | [The command](./04-command.md)                           | ✅ approved | `package.json`                                                                                                |

## Conclusions

**Built:** the sandbox's reach in the cleaner, a joined protection list, the shared marker reader, the composition of a
debug build over a sandbox (`FRESHER_SANDBOX`), the build banner, and `npm run dev:sandbox`.

**Measured:** 358 Rust tests passed (1 skipped) over cleaner, macos and sandbox after 02/01; `fresher-app` 83 at the end;
UI 849 tests and 225 end-to-end screenshots after 02/03, only new baselines; `scripts` 46 after 02/04.

**Left:** `cargo test -p fresher-app --release` runs only the release-gating tests (12 older debug tests cannot pass there);
the real-Trash run by hand waits in 05/02.

**Agent's calls taken during the build (to check):**

- Adapt the step text to the code as it is in `main` (plans 005 and 009 merged after this plan was written). Costs, if wrong: a reviewer flags a drift between text and code.
- The one marker reader moved to `fresher-macos` in 02/01 (which owns the reach), replacing the cleaner's and the generator's copies. Costs, if wrong: none; if skipped, two safety checks drift apart.
- The sandbox avoids its own home's places that prompt (faithful to the real app before plan 006 grants), so the Documents case became a negative and the one-by-one positive moved to `Library/Caches`. Costs, if wrong: Documents positives wait for plan 006.
