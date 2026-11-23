# 039/05/01 — The docs, the walk and the conclusions

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** docs

Every doc the plan affects, as built (a doc a step already updated is checked, not rewritten); the walk by hand for
the user; the plan's conclusions; the plan's section in the user's review list. The design:
[`../00-design.md`](../00-design.md).

## Files

- `docs/architecture/README.md` and `docs/architecture/removal.md` — the agreement in the cleanup's preconditions
  (order: settings file, agreement, journal), the High consent beside the permanent consent, the agreement commands in
  the command list, the UI's gate in the action layer (the sheet is not the guard).
- `docs/restrictions/deletion-safety.md` — "The agreement" and the High consent as built (01/01 and 02/01 wrote them;
  checked against the code).
- `docs/testing.md` — the harness's agreement assertion (at the first prepare of a file: a cleanup, or an uninstall
  for a file of uninstalls only), `agreement.rs` and `high_risk.rs`, the UI scenarios, the text check and its tests;
  "Demo recordings": the `cleanup` scenario accepts the agreement as a user would.
- `docs/quality.md` — the text check in the guards' table and the pre-push list (04/03 wrote it; checked).
- `docs/writing.md` — checked against the script's constants (`LIMITS`, `BANNED`, `SENTENCE_KINDS`): the doc and the
  script say the same.
- `docs/manual-testing.md` — the walk below as items of the consolidated checklist, with links to this step.
- `docs/categories/` — every quoted text matches its module's dictionary (04/02 updated them; checked with a grep of
  each category doc's quoted lines against the dictionaries).
- `docs/ideas/best-practices.md` and `docs/ideas/cleanup-preview-and-safe-removal.md` — their line on plan 039: what
  was built (the agreement, the danger zone, the High tick, the help, the guide and the check); their rows in
  `docs/ideas/README.md`.
- `docs/plans/014-onboarding/readme.md` — one line: the agreement exists (plan 039) and becomes the last of the
  first-run screens when 014 is built; the engine's guard stays.
- `docs/plans/013-release/readme.md` and `docs/plans/040-readme/readme.md` — one line each if not there: the release
  page and the README carry the agreement's words from `core.agreement.*` (040's table already names it; checked).
- `docs/plans/README.md` — the row of 039: 🔨 in progress (awaiting the user's walk).
- `docs/plans/039-agreement-and-wording/` — every step's conclusions (written as it was built; checked); every plan
  readme's conclusions; the global readme's **Conclusions**: what was built, the numbers (Rust and UI tests and
  coverage, the check's violation counts before and after the pass, the texts changed per dictionary, the hints
  added), what changed from the design and why, the calls taken while building; statuses 🔨 "awaiting the user's
  walk" on the global readme and on each plan and step.
- `docs/deferred/user-review-2026-11-22.md` — append a section `## 039 — The agreement, the danger zone and the
  wording` (create the file and its row in `docs/deferred/README.md` if it is absent), short bullets:
  - **Walks by hand**: the sandbox walk below; the dry run on the real Mac (the sheet once, About, a High plan's tick).
  - **The lawyer's question** (the draft's open question 1): whether to have a lawyer look at the agreement and the
    licences before the first public release; which country's law applies; the text is the agent's best effort.
  - **The agreement's five lines**, quoted, for the user to read and approve or change (a change raises
    `AGREEMENT_VERSION` before the first release only if a build with version 1 was given to anyone; otherwise version
    1 may be edited in place — the user's call).
  - **The issue form** on the repository after the merge (it cannot be previewed offline), and the report address in
    the help dialog.
  - **Rulings worth a second look**: the design's agent's calls 3, 4, 7, 8, 9, 10, 12 and 13 (one line each), and the
    refresh's 15 (the cleaner's two test echoes changed), 16 (the demo shows the sheet), 17 (Docker's part labels need
    no description) and 19 (the Trash line's two words); the banned words and the length limits of `docs/writing.md`.
  - **The demo**: `npm run demo:check -- cleanup` if it was not run while building (it needs the development app on
    screen); a GIF recorded later shows the agreement sheet for a moment.
  - **Texts whose meaning looked wrong** (from 04/02 and 04/03), listed, not changed.
  - **Known gaps**: "a changed text asks again" has no browser-level test until a second version exists; the licence
    notices of dependencies are still [deferred](../../../deferred/licence-notices.md).

## The walk by hand (for the user)

In sandboxes that together offer every level, each from a fresh folder: `rm -rf sandbox/downloads && npm run
dev:sandbox -- downloads` (Low, Medium and High on 740cf3c3) and `rm -rf sandbox/app-leftovers && npm run dev:sandbox
-- app-leftovers` (Safe, Medium and High); then `uninstall` (an uninstall of everything is High, of the app keeping
its data Medium) and `large-files` (a picked file is High). The `docker` file states all four levels but offers
nothing to remove (plan 032), so it is not the walk's; if a case file added later offers all four levels to remove,
the conclusions name it and the walk uses it alone. Each fresh sandbox meets the agreement anew:

- [ ] Scan: nothing selected; "Select all" of the Safe zone selects at once; of the Medium and High zones asks first,
  and Cancel selects nothing.
- [ ] Select one item of each level the sandbox offers; Review shows "Safe to remove" with Safe and Low, then "Danger zone" with Medium and
  High, set apart by a border; the summary says the selection reaches the danger zone.
- [ ] Move to Trash…: the agreement sheet opens; "Not now" starts nothing; again, "Read the full licence" shows both
  licences; "I understand" goes on to the confirmation.
- [ ] The confirmation: the High tick is off and the button inactive; tick it, confirm to the Trash; the result offers
  "Removed something you need?" and its four parts read well.
- [ ] Cleanups → put back: every item returns. A second cleanup does not ask for the agreement again.
- [ ] `uninstall`: Apps → an app → Uninstall asks for the agreement first (fresh folder); "Remove the app and
  everything it left" shows the High tick and the button stays inactive until it is ticked; keeping the data shows no
  High tick; there is never a permanent choice.
- [ ] `large-files`: Largest → tick a file → Review shows it under "Your own files" in the danger zone; the
  confirmation asks for the High tick.
- [ ] Settings → About: the five lines, "You accepted it on <today>", the licence button. Reset settings: the
  acceptance is still there.
- [ ] Read every screen once for the voice of `docs/writing.md`; hover and Tab over icons and numbers: each has a hint.
- [ ] VoiceOver (⌘F5) on the agreement sheet and the confirmation: every line, tick and button is read with its
  description.

## Checks

- `npm run pre-push` on the branch's last commit; `npm run test:e2e`; `npm run lint` (Markdown and links);
  `npx prettier --check` on the changed docs; every relative link of the changed files resolves.
- `git diff main -- crates/scanner crates/macos crates/system crates/store crates/rules` is empty;
  `git diff main -- crates/cleaner` touches only the echo of `src/test_support.rs` and `tests/never.rs`;
  `git diff main -- modules` touches only `locales/en.json` files.

## Conclusions

**As built.** The docs named above were updated or checked (the agreement and the High consent in
`architecture/README.md`, `architecture/removal.md` and `restrictions/deletion-safety.md`; `testing.md`: `agreement.rs`,
`high_risk.rs`, the text check, the interface's scenarios and the demo's acceptance; `writing.md` against the script's
constants; `quality.md` was right; the ideas, plans 013, 014 and 040, the plans table, `manual-testing.md` items 1.23
and 2.21, and a deferred item for the sandbox's `[settings]`). Every quoted text of `docs/categories/` matches its
dictionary. One long doc-comment line of `crates/engine/src/engine.rs` was wrapped to 120.

**The checks (on the last commit).** `npm run pre-push` passed: 4956 Rust tests (6 skipped; 97.57 % of lines),
1480 UI tests (98.13 % of lines), `check:texts` 0 violations. `git diff main` on `crates/scanner`, `crates/macos`,
`crates/system`, `crates/store` and `crates/rules` is empty; on `crates/cleaner` it touches only `src/test_support.rs`
and `tests/never.rs`; on `modules` only `locales/en.json` files. The Playwright specs were not touched by this step.
