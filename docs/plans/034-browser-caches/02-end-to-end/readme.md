# 034/02 — End to end

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, fixtures, e2e, app, app/ui, docs

The table's rows through the whole app: the two case files and their goldens (rule 8), the core e2e flows with the
real scanner and cleaner beside the other modules of `Library`, and the interface's path. The module is already
registered in the app (plan 030): no registration change. The design: [`../00-design.md`](../00-design.md), section 7.

## Steps

| #   | Step                                 | Status      | Shared files                                                                                    |
| --- | ------------------------------------ | ----------- | ----------------------------------------------------------------------------------------------- |
| 01  | [The case files](./01-case-files.md) | 🔨 in progress | `app/tauri/tests/sandboxes.rs`, `docs/testing.md`                                               |
| 02  | [The core e2e](./02-core-e2e.md)     | ✔️ done | `docs/testing.md`                                                                               |
| 03  | [The UI](./03-ui.md)                 | ✔️ done | `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |

All three after 01/03, in any order or in parallel; each adds its paragraph to `docs/testing.md`, and the later ones
rebase.

## Conclusions

The two case files and their goldens (the open browser as its own file, since an open app is shared by every case of a
file), the core flows with the real scanner and cleaner, and the interface's path pass. `cargo nextest run -p
fresher-app --test sandboxes`: 63 tests; `npm run sandbox -- all` writes 214 cases and leaves out the same 5 files it
left out before the plan; `npm run test:e2e:core` 144 passed; UI e2e 552 passed. The case files' walks by hand are
the user's (item 1.16 of [`../../../manual-testing.md`](../../../manual-testing.md)). Changes from the design are in the
three steps' conclusions.
