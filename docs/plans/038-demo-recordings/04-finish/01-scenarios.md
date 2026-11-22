# 038/04/01 — The first scenarios

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** fixtures

Two scenarios on the world `developer-mac`: `overview` (the README's hero: scan, the danger zones, a category's
details) and `cleanup` (select, review, confirm to the Trash, the result, Cleanups, put back). Each stays within the
budget: its pauses and the app's own time under 20 s. They name only screens on the branch's base; screens of plans
merged later get scenarios when wanted. The design: [`../00-design.md`](../00-design.md), section 4.

Read first: 02/01 (the format), 02/03 (how targets are found: role and accessible name), the scan screen
(`app/ui/src/screens/scan/`), the category screen, the review and confirm screens (`app/ui/src/screens/review/`,
`app/ui/src/product/confirm-dialog/`), the Cleanups screen (`app/ui/src/screens/cleanups/`), the sidebar
(`app/ui/src/screens/frame-sidebar.tsx`), `app/ui/src/locales/en.json` and the module texts that name the world's
categories (`modules/*/locales/en.json`), and `app/ui/e2e/flows.spec.ts` (the same flows addressed by role and name).
**Every name in a scenario is copied from those texts, never guessed.** Do not run the app.

## Files

- `fixtures/demos/scenarios/overview.toml` (rewritten from 02/01's first version) — `world = "developer-mac"`,
  window 1100 × 720. Steps, about 14–16 s in all: pause 800; click the scan button; `wait = "scan-finished"`; pause
  1500 (the result and its danger zones); hover the first danger zone's largest category card (a `name-start` on its
  title, since the name carries the size); pause 1200; scroll the results down (`by` about 400) and pause 1000; click
  that category's card to open its details; pause 1800; click back to the result; pause 1000.
- `fixtures/demos/scenarios/cleanup.toml` (new) — the same world and window. Steps, about 16–18 s: pause 600; click
  the scan button; `wait = "scan-finished"`; pause 800; tick a Safe zone's category (its checkbox or the zone's
  select-all, whichever the screen offers by role); pause 600; click Review in the sidebar or the selection bar; pause
  1200; click the move-to-Trash button (`ui.confirm.trash`, "Move to Trash") in the confirm dialog; `wait =
  "cleanup-finished"`; pause 1500 (the result); open Cleanups; pause 800; click "Put back" on the cleanup; wait for its
  row's put-back state (a target that appears); pause 1200.
- A scenario's head comment: what it shows, the screens it touches, its rough length.

## Produces

The two scenarios the README (plan 040) and the release notes use.

## Checks

- The crate's test that every scenario parses and names an existing world passes (`cargo nextest run -p
  fresher-sandbox`); the pauses of each add up to well under 20 s (the parser refuses over 20 s; aim for ≤ 12 s of
  pauses so the app's own time fits).
- Each target's role and name is traced to its text key in the step's conclusions (one line per target:
  `role "name" ← key`), so a reviewer can check them without running the app.
- `typos` passes. The user plays both in 04/02's walk (`npm run demo:check -- all`).

## Conclusions

Built as briefed, with the deviations below. `overview.toml` (pauses 5.5 s) and `cleanup.toml` (pauses 9.0 s) parse,
name the world `developer-mac`, and every target is traced by `crates/sandbox/src/scenario/traced.rs` (a test that
resolves each step's target, click, hover, scroll or wait, to a key of the UI, core or module texts). The same test
refuses any step whose target is the permanent delete (`ui.confirm.permanent`, `ui.confirm.delete`, or any text with
"permanent" or "delete"): a demo moves to the sandbox's Trash only. Negative tests: an unknown name, the permanent
choice by its whole text, the permanent button by text and by start, a hover and a wait on it.

Targets, `role "name" <- key`:

- button "Scan" <- `ui.scan-hero.scan` (the sidebar's "Scan" is skipped by the finder, see below)
- heading "Xcode build data" <- `dev-caches.xcode-derived-data.name` (the largest Safe category, about 38 GB)
- region "Safe to remove" <- `core.danger.safe.zone` (scrolled by 400 and back)
- checkbox "Select Xcode build data" <- `ui.category-card.select`
- button "Review" <- `ui.selection-bar.review` (the sidebar's Review reads "Review" and the count, so it differs)
- button "Move to Trash…" <- `ui.screen.review.remove` (the Review screen's button: the brief had left it out)
- button "Move to Trash" <- `ui.confirm.trash` (the confirm dialog)
- button "Cleanups" <- `ui.screen.frame.nav.cleanups`
- button, name starts "Cleanup of" <- `ui.cleanup-row.title`
- button "Put back all still in the Trash" <- `ui.screen.cleanups.put-back-all`

Deviations:

- No step opens a category's details: every card's Details button is named "Details" (the card is the button's
  description, not its name), so the finder finds many. The overview hovers the largest card's heading and scrolls
  instead. A name such as "Details of {name}" on the button would allow it (an app/ui change, outside this plan).
- The finder takes the page's own match when the sidebar (`nav`) and the page both have one: the hero's "Scan" and
  the sidebar's "Scan" are the same name. Tests added to `finder.test.ts`.
- A scroll step scrolls the nearest ancestor that overflows (the screen scrolls in a plain container no target can
  name); a test added to `steps.test.ts`.
- The cleanup puts back with the cleanup's "Put back all still in the Trash" (one button; each item has a "Put back"
  of the same name). No target appears when it is done (the rows are list items, not a role of the list), so a
  1500 ms pause follows it instead of a wait.
- The overview heading is matched whole: a card's title carries no size.

