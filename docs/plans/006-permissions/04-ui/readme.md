# 006/04 — UI

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** app/ui

What the user sees: the access state in the UI's store, the coverage notices next to the results they limit, sections
that are limited or unavailable, the opt-in for guarded folders, and the Permissions group of the settings screen.
Built from plan [003](../../003-base-ui/readme.md)'s components (`product/permission-card`, `product/category-card`,
`product/module-section`, `product/settings-row`, the gallery) and the screens of plans 003, 005 and 012; every
explanation from the core's dictionary (`core.access.*`, already there), counts, labels and buttons from the UI's
chrome dictionary (`app/ui/src/locales/en.json`); no business logic in the UI — states, availability and coverage come
from the core. The design: [`../00-design.md`](../00-design.md), section 8.

The new files go into the existing folder `app/ui/src/screens/permission/` (it holds `use-access.ts`), not a new
`screens/access/` as the first version said (2026-11-19).

Depends on plan 02 (done), step 03/04 (the IPC functions), plan 003 (done) and plan 005 (done).

## Steps

| #   | Step                                                 | Status      | Depends on | Shared files                                                                                                                   |
| --- | ---------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 01  | [Access in the store](./01-store.md)                 | ✔️ done     | 03/04      | `app/ui/src/state/reduce.ts`, `app/ui/src/screens/permission/use-access.ts`                                                    |
| 02  | [Coverage notices](./02-coverage-notices.md)         | ✔️ done     | 01         | `app/ui/src/screens/overview/overview-screen.tsx`, `app/ui/src/product/permission-card/`, `app/ui/src/locales/en.json`, `app/ui/e2e/core-mock.ts` |
| 03  | [Limited and unavailable sections](./03-sections.md) | ✔️ done     | 01, 02     | `app/ui/src/product/{category-card,category-grid,meaning-list}/`, `app/ui/src/screens/module/`, `app/ui/src/locales/en.json`    |
| 04  | [The Permissions group](./04-permissions-group.md)   | ✔️ done     | 01, 02     | `app/ui/src/screens/settings/{settings-screen.tsx,settings-groups.ts}`, `app/ui/src/locales/en.json`                           |

Steps run one by one, in order. Step 03 now depends on 02: an unavailable section reuses the notice's action, and both
add chrome keys to `app/ui/src/locales/en.json`.

## Conclusions

All four steps are done. The UI now shows the access state end to end: the store keeps the capabilities and the coverage
of the result on screen (01); the overview and module pages carry one notice per capability that limited a scan, with
the one action that fixes it (02); categories and sections that are limited or unavailable say what is missing (03);
the settings screen has one Permissions group listing every capability, its state, what it unlocks, the opt-in
switches and "Check again" (04). Everything is text from the core's dictionary and states from the core; the UI
computes nothing. `AppProviders` refreshes the module list on every access change, so availability follows a grant
without a restart.
