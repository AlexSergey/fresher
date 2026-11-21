# 027/03 — Details and navigation

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

A category's details on their own screen, opened from its card; the module pages and the sidebar's modules go; the
overview is renamed Scan; the sidebar scrolls; Review is split by danger zone. The design:
[`../00-design.md`](../00-design.md), sections 1, 3, 5 and 6 (agent's calls 10, 11, 12 and 14; decisions 5 and 7).

## Steps

| #   | Step                                                     | Status      | Depends on | Shared files                                                                                                |
| --- | -------------------------------------------------------- | ----------- | ---------- | ----------------------------------------------------------------------------------------------------------- |
| 01  | [The category's details](./01-category-details.md)       | 🔨 in progress | 02/03      | `app/ui/src/state/navigation.ts`, `app/ui/src/screens/{app-frame,frame-sidebar,screen-stack}.tsx`, `app/ui/src/locales/en.json` |
| 02  | [Scan and the sidebar](./02-scan-and-sidebar.md)          | 🔨 in progress | 03/01      | `app/ui/src/state/navigation.ts`, `app/ui/src/screens/{app-frame,frame-sidebar,screen-stack}.tsx`, `app/ui/src/screens/review/review-screen.tsx`, `app/ui/src/locales/en.json` |
| 03  | [Review in zones](./03-review-zones.md)                   | 🔨 in progress | 02/02      | `app/ui/src/screens/review/`, `app/ui/src/locales/en.json`                                                   |

01, then 02. 03 shares only `app/ui/src/locales/en.json` and `review-screen.tsx` with 02 and may run beside 01; the
second of 02 and 03 to merge rebases.

## Conclusions

Built in three steps, awaiting the user's walk.

- **03/01** a screen per category with its own path list (12 rows in a frame), the four sentences open, the left-out
  lines and the module's choices. The module pages and `modules/registry.ts` are deleted. Review found that a
  category with only left-outs had no way in; the "Found, but left out" section on Scan was added.
- **03/02** the overview is renamed Scan (the name is shared with the sidebar item, so the e2e helpers are
  `scanButton` and `navScan`), the sidebar scrolls so Settings is always reachable; the level section's heading is
  "Danger level" and the triple "safe" wording was cut.
- **03/03** Review is split in the same zones with the same heads, without "Select all" (call 10). A test of the flows
  was added in review.
- The final review added: Back returns focus to the card's Details button, the icon wording of the zones, and an e2e
  for the WebKit Space key-up (which did not reproduce).
