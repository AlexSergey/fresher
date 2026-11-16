# 006/04 — UI

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** app/ui, core (texts)

What the user sees: the access state in the UI's store, the coverage notices next to the results they limit, sections
that are limited or unavailable, the opt-in for guarded folders, and the Permissions group of the settings screen.
Built from plan [003](../../003-base-ui/readme.md)'s components; every text from the core's dictionary; no business
logic in the UI — states, availability and coverage come from the core. The design:
[`../00-design.md`](../00-design.md), section 8.

Depends on plan 02, step 03/04 (the IPC functions), plan 003 (components, screens, gallery) and step 005/03 (the
settings screen).

## Steps

| #   | Step                                                     | Status   | Depends on        | Shared files                                                     |
| --- | -------------------------------------------------------- | -------- | ----------------- | ---------------------------------------------------------------- |
| 01  | [Access in the store](./01-store.md)                     | ✅ approved | 03/04             | `app/ui/src/state/`                                              |
| 02  | [Coverage notices](./02-coverage-notices.md)             | ✅ approved | 01, 003           | the overview screen and its result summary (plan 003)            |
| 03  | [Limited and unavailable sections](./03-sections.md)     | ✅ approved | 01, 003           | the overview and the generic module page (plan 003)              |
| 04  | [The Permissions group](./04-permissions-group.md)       | ✅ approved | 01, 02, 005/03    | the settings screen (plan 005), `app/ui/src/gallery/`             |

Steps 02 and 03 touch different screens and may run in parallel after 01.

## Conclusions

Not written yet.
