# 003/07 — Base screens

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

The app as the user sees it, on the `demo` module: the frame with its sidebar and screen changes, the overview, the
generic module page, the review screen, the settings and the missing-permission state; then the flows and the
screenshots of every screen. Screens are the only place that reads the store and calls the IPC module; they pass data
and callbacks to the product components. The design: [`../00-design.md`](../00-design.md), "Screens".

Depends on plans 04, 05 and 06. 01 first; 02–05 write separate folders of `app/ui/src/screens/` and run in parallel;
06 last. Screen texts go to `app/ui/src/locales/en.json` under `ui.screen.<screen>.*`.

## Steps

| #   | Step                                                          | Status   | Depends on | Shared files                                       |
| --- | ------------------------------------------------------------- | -------- | ---------- | -------------------------------------------------- |
| 01  | [The app frame](./01-app-frame.md)                            | ✔️ done    | —          | `app/ui/src/app.tsx`, `app/ui/src/main.tsx`, `locales/en.json` |
| 02  | [Overview](./02-overview.md)                                  | ✔️ done    | 01         | `locales/en.json`                                  |
| 03  | [The module page](./03-module-page.md)                        | ✔️ done    | 01         | `locales/en.json`                                  |
| 04  | [Review](./04-review.md)                                      | ✔️ done    | 01         | `locales/en.json`, `docs/hacks/README.md`          |
| 05  | [Settings and permission](./05-settings-and-permission.md)    | ✔️ done    | 01         | `locales/en.json`, `app/ui/src/state/` (settings and access events) |
| 06  | [Flows and screenshots](./06-flows-and-screenshots.md)        | ✔️ done    | 02–05      | `app/ui/e2e/core-mock.ts`                          |

## Conclusions

Done on 2026-11-16; each step's conclusions are in its file. The app is the frame and four screens on the `demo`
module, proven together by the e2e flows and photographed in every state in WebKit and Chromium. No new dependency or
token; one new icon (`Package`).

- **The frame** (01): the sidebar with the modules of `get_modules` and their found sizes, screen changes that fade
  and rise (only fade under reduced motion), the focus on each new screen's `h1`, refusals as error toasts and an
  unreachable core as the frame's error state. Each screen replaced its placeholder without touching a frame file.
- **The overview** (02): the scan hero, Scan again only after a scan, the partial notice, the permission card when
  Full Disk Access is denied, the cards largest first opening their module's page at their category.
- **The module page** (03): generic, from the descriptor alone: one section per category with findings, the largest
  expanded, the four sentences behind a disclosure, the selection bar.
- **Review** (04): the selection grouped by category with what is lost, the summary's count, size and highest level;
  **Move to Trash…** disabled until plan 008 (the hack "Review cannot remove yet").
- **Settings and permission** (05): every setting by kind, the core's first then each module's, a refusal in its row,
  read-only files explained; the store keeps the settings and the access status, kept current by their events.
- **Flows and screenshots** (06): the scan-to-review, Stop, settings, long-list and keyboard-only paths, the busy
  refusal, the disabled removal and reduced motion; 22 screenshots. The mock holds a scan mid-stream, stops it, refuses
  a second one and echoes setting changes. One bug fixed on the way: the focus of a list row was lost when its page
  arrived (`VirtualList`).
- **To fold or watch later**: the duplicate checkbox rule of 07/02 and 07/03 is one copy in `state/selection.ts`; a
  module switched off still shows a "scanning" chip during a scan (the store does not keep `scan-started`'s modules); a
  failed scan shows both Scan again and the hero's Try again.
