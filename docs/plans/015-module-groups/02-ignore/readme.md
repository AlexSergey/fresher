# 015/02 — Ignoring a module and a group

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, app, app/ui

The two new ignore lists in the core and the engine, Settings' three tabs, the ignored module's look, and the
end-to-end scenarios. The design: [`../00-design.md`](../00-design.md), sections 3 to 5 and 7.

## Steps

| #   | Step                                                  | Status                                  | Shared files                                                                                       |
| --- | ----------------------------------------------------- | --------------------------------------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The ignore lists of modules and groups](./01-ignore-lists.md) | ✔️ done | `crates/core/src/{settings,command,error}.rs`, `crates/engine/src/{ignore,settings,results}.rs`   |
| 02  | [Settings: the three tabs](./02-settings-tabs.md)     | ✔️ done | `app/ui/src/screens/settings/`, `app/ui/src/design/layout/tabs/`                                    |
| 03  | [An ignored module in the sidebar and on its page](./03-ignored-module.md) | ✔️ done | `app/ui/src/screens/{frame-sidebar.tsx,module/,overview/}`                     |
| 04  | [End-to-end scenarios](./04-e2e.md)                   | ✔️ done | `crates/e2e/tests/ignore.rs`, `app/ui/e2e/`                                                         |

01 first; 02 and 03 in parallel after it (03 after 01/02); 04 last.

## Conclusions

Two lists beside plan 005's two, three tabs in Settings, the dimmed module and its note, and the e2e over them. **The
user's decision of 2026-11-17 on the two Settings tabs**: "Ignoring a module or a group must also be editable in
Settings, which gets two tabs for it: one listing the ignored groups, and one for ignoring specific modules. Each tab
lists every group or module with its toggle and restore, consistent with the Ignored group of plan 005; UI tests and
e2e scenarios for the tabs." Built so: the two tabs plus the third, "Ignored items", for plan 005's lists. The
Debug heading sits tight under the description in the Ignored modules tab (spacing, left for a later polish).
