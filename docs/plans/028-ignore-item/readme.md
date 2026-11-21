# 028 — Ignore one item

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** app/ui, e2e, docs

Written on 2026-11-21 from the deferred note `ignore-an-item-button` (removed by this plan) and
from a read of the code on `feat/028-ignore-item` (stacked on plan 027's head, 951a8e79). Approved by the user's
delegation of 2026-11-20: after plan 027, build the Ignore of a single item autonomously, take the recommended
decision on every question and leave each as an agent's call; the user checks everything together later. The design:
[`00-design.md`](./00-design.md).

## Goal

A finding on a category's details can be ignored from its own row: a small settings menu at the row's end with
"Ignore this item" and its plain sentence. The row leaves the list at once, the focus goes to its neighbour, and the
item is restored from Settings → Ignored → Ignored items, as every ignored path already is. The core, the engine,
the shell, the cleaner's protection and the Restore side are built (plan 005); this plan adds only the UI action, the
list's prompt follow-up of an ignore, and the e2e mock's paths.

Branch: `feat/028-ignore-item`, stacked on `feat/027-scan-screen` (plan 027 is not in `main` yet: its walk is
pending). It is squash-merged after plan 027.

## Plans

| #   | Plan                                                | Status      | Depends on | What                                                                                  |
| --- | --------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------- |
| 01  | [The row's action](./01-row-action/readme.md)       | 🔨 in progress | —          | the list follows an ignore at once; the row's menu; the details wire it, mock and e2e |
| 02  | [Docs and finish](./02-docs-and-finish/readme.md)   | 🔨 in progress | 01         | the deferred item goes; architecture, testing, manual-testing; checks; the user's walk |

01, then 02; the steps run one after the other (they share `finding-row.tsx`, `locales/en.json` and the baselines).

## Acceptance

- On a category's details, every finding with a path has its row menu; choosing "Ignore this item" takes the row out
  of the list at once, the focus lands on the row that took its place (or on "Back to Scan" when the list empties),
  and Settings → Ignored → Ignored items lists the path with Restore, which brings the row back without a rescan.
- Review's lists have no row menu; no removal path changes (`git diff 951a8e79 -- crates modules app/tauri` is
  empty).
- `npm run pre-push` and `npm run test:e2e --workspace app/ui` pass on the branch's last commit; coverage stays at or
  above 90 % of lines; the 100,000-row scroll stays within rule 4's 5 % of plan 027's numbers.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. The action is a per-row settings menu (the card's menu primitive and icon), not a key alone or a hover button.
2. The row's menu button is a tab stop only in the list's focused row; Tab from the row reaches it.
3. Keys on a control inside a row belong to that control; the list moves only from a row itself.
4. An ignore-list change resets and refreshes the finding pages at once, as a cleanup's removal does.
5. The whole selection is still cleared on an ignore (today's rule: the core drops its pending plan); not narrowed.
6. A finding the app may not remove can be ignored too; a tool's object (no path) has no menu.
7. Only the category's details offer the menu; Review's rows do not.
8. When the last row is ignored, the focus goes to "Back to Scan".
9. The texts are UI keys beside the card's (`ui.finding-list.*`).

## Open questions

None.

## Conclusions

Built on `feat/028-ignore-item` in 8 commits over plan 027's head (951a8e79), **awaiting the user's walk**: the plan is
not marked done until the user has looked at the result (the walk list is in
[`02-docs-and-finish/02-checks-and-walk.md`](./02-docs-and-finish/02-checks-and-walk.md)). The branch is stacked on
`feat/027-scan-screen` and is squash-merged after it.

**What was built.** A finding row on a category's details has a settings menu at its end with "Ignore this item" and
its sentence. Choosing it calls the existing `ipc.ignore`; the finding list asks the core again at once (an ignore-list
change resets the cached pages, as a cleanup's removal does), the row leaves, the focus goes to the row that took its
place, and to "Back to Scan" when the list empties. Settings → Ignored → Ignored items lists the path and Restore
brings the row back without a rescan. The e2e mock learned paths and nested ignored paths. `git diff 951a8e79 HEAD --
crates modules app/tauri` is empty: no core, shell or removal path changed. The deferred note
`ignore-an-item-button` is deleted; the architecture, testing and manual-testing docs say what the UI is now.

**Numbers on the last commit of the code (40c669ac), on the reference Mac.**

| Check                                 | Result                                                                                  |
| ------------------------------------- | --------------------------------------------------------------------------------------- |
| `npm run pre-push`                    | green: lint, guards, knip, machete, deny, scripts' tests                                  |
| Rust tests and coverage               | 3,290 passed (5 skipped); the floor of 90 % held                                          |
| UI unit tests and coverage            | 1,150 passed in 130 files; 98.59 % of lines (floor 90 %)                                  |
| Playwright gallery, WebKit, Chromium  | 449 passed in one run, no flake; `git status` clean after it                              |
| `npm run test:e2e:core`               | 110 passed, 2 skipped                                                                     |
| Perf, Chromium, 100,000-row scroll    | `951a8e79`: 1300 frames, p95 8.9 ms; branch: 1300 frames, p95 8.5 ms                      |
| Perf, Chromium, details after a scan  | `951a8e79`: 107 frames, p95 10 ms; branch: 108 frames, p95 9.3 ms                         |
| Perf, Chromium, scan streaming        | `951a8e79`: 1307 frames, p95 9.4 ms, longest 26.6; branch: 1305 frames, p95 9.0, longest 25.4 |
| Perf, WebKit (reference), scroll      | `951a8e79`: 652 frames, p95 19 ms (idle 19); branch: 651 frames, p95 17 ms (idle 18), second run 19 (idle 19) |
| Perf, bundle (gzipped)                | JS 119.3 kB to 119.6 kB, CSS 16.2 kB to 16.2 kB (budgets 200 and 40)                      |
| DOM nodes, a category's details       | 764 to 878 for 19 rows: the row's menu button and its icon (rule 4 limits time, not nodes) |

Rule 4's 5 % holds: the scroll and the details row are the same or better in Chromium, the scroll is the same in
WebKit. The WebKit scan and transitions tests failed on the branch and on `951a8e79` alike, on the same rows ("details
after a scan" p95 21 to 24 ms against an idle row of 19, one frame over 50 ms; "re-sort of 12 cards" 21 ms) and in a
rerun: the machine was in a noisy spell, the base fails in the same places, so it is the known
[`ui-perf-webkit-jitter`](../../known-issues/ui-perf-webkit-jitter.md) (updated with this run), not a result of this
plan. The first full branch run failed one WebKit scan test only; WebKit's idle row moved between 18 and 19 ms
between runs.

**The calls taken while building (for the user to look at).**

- Ignoring one item clears the whole selection (the core's existing rule: it drops its pending plan); narrowing it
  needs the core to say which findings left.
- After the row menu closes, focus is on the menu button; the arrow keys move rows only from the row itself, so
  Shift+Tab goes back to the row.
- In the cleanup screen's long list, the arrows pressed on a row's put-back button no longer move the list (the same
  rule: a key on a control inside a row belongs to the control).
- Only Details offers the menu; Review's rows do not. A tool's object (no path) has no menu.
- When the last row is ignored, the focus goes to "Back to Scan".

**What changed from the design and why.**

1. The last-row focus is keyed on the overview's category count and is covered by a unit test only: the e2e mock does
   not recompute category totals for an ignored item (design call 10), so the count never reaches zero there.
2. The refresh after an ignore re-asks only the last ensured range of a list; the cache is dropped, so pages that
   scroll into view load again on their own.
3. A review found that the focus jumped to "Back to Scan" when a scan started and emptied the list: the focus rule
   now waits for the scan to end (`!scanning`, with a negative test). The same fix round added the nested ignored path
   to the mock, a focus assertion to the keyboard e2e, and a truer fixture for the refused ignore.
4. The final review found the level and size columns off by one menu column in a row with an empty menu: such a row
   keeps an `aria-hidden` placeholder of the column (story and baseline added).
5. The menu's `menuOpen` prop is gallery-only (a story needs it to show the open menu); nothing in the app sets it. The
   details' header gets a gap rule in the finding list's CSS so the menu column does not shift it.
6. Old plans 005 and 015 still link the deleted deferred note; they are history and keep the dead links (the same
   precedent as plan 027).

**Left open (parked minors, none a defect the user can hit):** the virtual list has a redundant `slot !== null` test; the
story's 110 px padding is a magic number; the architecture sentence about the virtual list's keys sits inside the menu
bullet.

**The user's walk.** Pending. The six items of the walk list are unchecked; the walk in a sandbox
(`npm run dev:sandbox -- trial`) is the user's and this plan becomes ✔️ done when it passes. The user checks plans 027,
028, 023 and 024 together (the delegation of 2026-11-20).
