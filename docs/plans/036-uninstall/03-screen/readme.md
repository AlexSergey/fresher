# 036/03 — Screen

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

The sidebar's **Apps** entry, the Apps list with its filter, an app's page with its files item by item, the two
choices and **Uninstall…**, the normal confirmation, and the UI's end-to-end scenarios. The UI renders what the core
sends: every size, danger, sentence and reason comes from the core (rule 7, "No business logic in the UI"). The
design: [`../00-design.md`](../00-design.md), sections 7, 8, 11 and 12.

## Steps

| #   | Step                                                          | Status      | Shared files                                                                                                     |
| --- | ------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------- |
| 01  | [The Apps list](./01-apps-list.md)                            | 🔨 awaiting the walk | `app/ui/src/{ipc/index.ts,ipc/mock.ts,state/navigation.ts,screens/frame-sidebar.tsx,screens/screen-stack.tsx,design/icons/icons.ts,locales/en.json}`, `app/ui/src/gallery/` |
| 02  | [The app's page and the uninstall](./02-app-page.md)          | 🔨 awaiting the walk | `app/ui/src/{ipc/index.ts,ipc/mock.ts,state/navigation.ts,state/cleanup.ts,state/reduce.ts,state/cleanup-actions.ts,locales/en.json}`, `app/ui/e2e/`, `app/ui/src/gallery/` |

01, then 02.

## Conclusions

The Apps list and the app's page with the choices, in WebKit and Chromium, with axe and keyboard specs. A refused uninstall reloads the page.
