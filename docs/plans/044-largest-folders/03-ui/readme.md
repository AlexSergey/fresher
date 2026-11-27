# 044/03 — The Folders and Browse tabs

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** app/ui, e2e, core, engine,
app/tauri, docs

Largest's Folders lists the folders left alone, with a bar, a separator and tiles above the rows and Show in Finder on
each; the old folder tree moves to a third tab, Browse, and becomes read-only (no ticks, Show in Finder). The design:
[`../00-design.md`](../00-design.md), sections 7 to 9; agent's calls 14, 17, 20 and 21.

## Steps

| #   | Step                                                          | Status   | Shared files |
| --- | ------------------------------------------------------------- | -------- | ------------ |
| 01  | [The mockup](./01-mockup.md)                                  | ✔️ done | `docs/mockups/0.2.0/largest.dc.html`, `docs/mockups/README.md` |
| 02  | [The IPC wrappers and the mock](./02-ipc-and-mock.md)         | ✔️ done | `app/ui/src/ipc/{index.ts,index.test.ts,mock.ts}`, `app/ui/e2e/support/largest-mock.ts`, `app/ui/e2e/core-mock.spec.ts`, `app/ui/src/test/largest-scan.ts` |
| 03  | [`FolderRow`](./03-folder-row.md)                             | ✔️ done     | `app/ui/src/product/folder-row/`, `app/ui/src/locales/en.json` |
| 04  | [The Folders and Browse tabs on Largest](./04-folders-tab.md) | ✔️ done     | `app/ui/src/screens/largest/`, `app/ui/src/locales/en.json`, `app/ui/e2e/{flows,visual,axe,keyboard}.spec.ts`, their baselines |
| 05  | [Browse is read-only](./05-browse-read-only.md)               | ✔️ done | `crates/core/src/largest/page.rs`, `crates/engine/src/largest/{place.rs,largest_tests.rs}`, `crates/e2e/tests/`, `bindings.generated.ts`, `app/ui/src/{product/own-item-row,screens/largest}/`, `app/ui/e2e/`, their baselines |

01 (the user approves the mockup), then 02, 03, 04 and 05.

## Conclusions

To be written when step 05 is done.
