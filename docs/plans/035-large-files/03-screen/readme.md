# 035/03 — The screen

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** app/ui

The Largest screen: a sidebar entry, the Files view with its filters and pages, the Folders view with its breadcrumb,
the rows' checkboxes wired to the selection through the core's pick, the coverage notices, and every state the core can
be in. The UI renders the core's answers and texts and decides nothing (CLAUDE.md, "TypeScript / React Rules"). The
design: [`../00-design.md`](../00-design.md), sections 7 and 8.

## Steps

| #   | Step                                                          | Status      | Shared files                                                                                                   |
| --- | ------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------------- |
| 01  | [The entry, the Files list and the row](./01-list.md)         | 🔨 in progress | `app/ui/src/ipc/index.ts`, `app/ui/src/state/navigation.ts`, `app/ui/src/screens/{frame-sidebar.tsx,screen-stack.tsx,app-frame.tsx}`, `app/ui/src/locales/en.json`, `app/ui/src/gallery/registry.ts`, `app/ui/src/design/icons/icons.ts` |
| 02  | [Filters, Folders and the selection](./02-folders-and-selection.md) | 🔨 in progress | `app/ui/src/locales/en.json`                                                                            |

In order: 01, then 02.

## Conclusions

The Largest screen is built as designed: two views, three filters, the selection feeding the same bar and Review as the
scan's findings, every sentence from a dictionary. The UI decides nothing. Details are in the steps.
