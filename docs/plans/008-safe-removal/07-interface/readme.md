# 008/07 — Interface

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** app/ui, core (texts)

The review screen's "Move to Trash…" comes alive: the core's plan in `ConfirmDialog`, a calm progress view, an honest
`ResultSummary`, and a new Cleanups screen with put back. Built on plan 003's components and rules: every number and
text from the core, components small and presentational, every call through `ipc/`, the safety screens static. The
design: [`../00-design.md`](../00-design.md), sections 4, 9 and 11; plan 003's
[design](../../003-base-ui/00-design.md) for the components, the motion rules and the tests.

Depends on 06/04 (the commands and the bindings). Every test runs on the mocked core.

## Steps

| #   | Step                                                   | Status   | Shared files                                                                                                                         |
| --- | ------------------------------------------------------ | -------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| 01  | [The IPC and the state](./01-ipc-and-state.md)         | ✔️ done     | `app/ui/src/ipc/{index,mock}.ts`, `app/ui/src/state/{store,reduce,selection,findings}.ts`, `app/ui/src/test/`                        |
| 02  | [The confirmation](./02-confirmation.md)               | ✔️ done     | `app/ui/src/product/confirm-dialog/`, `app/ui/src/screens/review/`, `app/ui/src/locales/en.json`, `docs/hacks/README.md`             |
| 03  | [Progress and the result](./03-progress-and-result.md) | ✔️ done     | `app/ui/src/product/result-summary/`, `app/ui/src/screens/review/`, `app/ui/src/screens/app-frame.tsx`, `app/ui/src/locales/en.json` |
| 04  | [The Cleanups screen](./04-cleanups-screen.md)         | ✔️ done     | `app/ui/src/state/navigation.ts`, `app/ui/src/screens/{frame-sidebar,screen-stack}.tsx`, `app/ui/src/locales/en.json`                |
| 05  | [Flows and screenshots](./05-flows-and-screenshots.md) | ✔️ done     | `app/ui/e2e/`, the screenshots                                                                                                       |

01 first; then 02 and 03 in order (both touch the review screen) beside 04 (separate folders; `en.json` keys in their
own prefixes); 05 last.

## Conclusions

The five steps are built as written, with the calls each step's conclusions give; on the last commit `npm run pre-push`
passes and the UI e2e pass in WebKit and Chromium. The user flow: tick on the overview or a module page → Review →
**Move to Trash…** ("Checking each item…") → the core's plan in the confirmation (Cancel first, the permanent choice
off and red when ticked, what will not be removed and why) → the progress with Stop → the result ("Moved to the
Trash", the space measured beside it) → "See this cleanup" → put back, one item or all still in the Trash. The hack
"Review cannot remove yet" is gone.

- **The app menu's Quit asks too** (the known issue's recommended option): the default menu's Quit is replaced by an
  item of the app's own with the same title and ⌘Q (`app/tauri/src/shell.rs`); during a cleanup it publishes
  `quit-requested` and the UI asks as for the window's close, otherwise the app exits. The Dock's Quit cannot be held
  and stays the known issue ([`app-quit-during-cleanup.md`](../../../known-issues/app-quit-during-cleanup.md),
  narrowed). The menu needs the main thread, so its shape is not under test; the held quit is.
- **From the engine's fixes merged in (75c1ddd)**: a plan whose items were all left out has no confirm button (02/03's
  conclusions); covered findings in `findings-removed` and a second put back answering `put-back` need nothing more
  from the UI (the pages are dropped whole; the row shows the core's text).
- **Outside the plan's folders**: `typos` 1.51 flagged two words already on the branch, a word in a comment of
  `crates/core/src/cleanup/protection.rs` and a deliberately cut word in a journal test of `crates/store`; both were
  reworded in place (comment and test data only) so the hooks pass.
- **To check**: the "how" lines are the UI's texts and a tool's line cannot name the tool (`PlanGroup` has no tool);
  a mixed plan of Trash and tool items is titled "Move N items to the Trash?" while its tool group says it cannot be
  put back; a tool-only plan's "Remove" button is red like the permanent one.
