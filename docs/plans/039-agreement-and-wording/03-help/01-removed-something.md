# 039/03/01 — "Removed something you need?"

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** app/ui, repo

A help entry on the result of a cleanup and on Cleanups, and the issue form for a wrong finding. The design:
[`../00-design.md`](../00-design.md), section 3; agent's call 11.

## Read first

`app/ui/src/product/result-summary/` (its footer buttons: on 740cf3c3 "Show in Trash" with the Trash's invitation,
plan 037, "See cleanup" and "Done"), `app/ui/src/screens/cleanups/cleanups-screen.tsx` (its header and toolbar) and
`cleanup-screen.tsx`, `app/ui/src/product/trash-line/` and the `ui.trash.*` texts (plan 037: Finder may empty the Trash
by itself; the app never empties it — the help says the same), `app/ui/src/design/layout/dialog/`, `app/ui/src/design/primitives/button/`,
`docs/restrictions/privacy.md` (nothing is sent; the app opens no address), `.github/` (no issue templates yet).

## Files

- `app/ui/src/product/recovery-help/` (new: `recovery-help.tsx`, `.module.css`, `.stories.tsx`, `.test.tsx`) — a
  ghost `Button` "Removed something you need?" (`ui.recovery.button`) that opens a `Dialog` titled `ui.recovery.title`
  with four parts, each a third-level heading and one or two sentences:
  - `ui.recovery.put-back.title` "Put it back" / `ui.recovery.put-back.text` "Open Cleanups, choose the cleanup, and
    put back the items you need. They return to where they were."
  - `ui.recovery.trash.title` "Find it in the Trash" / `ui.recovery.trash.text` "Items moved to the Trash stay there
    until the Trash is emptied, by you or by Finder. You can also drag one back from the Trash in Finder."
  - `ui.recovery.permanent.title` "If you deleted it permanently" / `ui.recovery.permanent.text` "A permanent delete,
    and what an app's own tool removed, cannot be undone by Fresher. A Time Machine backup may still hold a copy."
  - `ui.recovery.report.title` "Tell us what went wrong" / `ui.recovery.report.text` "If Fresher offered something it
    should not have, report it so the next version leaves it alone. Fresher sends nothing itself; open this address in
    your browser:" followed by the address `https://github.com/AlexSergey/fresher/issues/new?template=wrong-finding.yml`
    as selectable text (a `<code>` the user can select and copy; not a link the app opens; agent's call 11).
  - The address is one constant in the component (`REPORT_ADDRESS`), with a comment pointing to the issue form.
  - The close button `ui.recovery.close` "Close", first focus.
- `app/ui/src/product/result-summary/result-summary.tsx` — the help button beside the result's actions.
- `app/ui/src/screens/cleanups/cleanups-screen.tsx` — the help button in the screen's toolbar (and on a cleanup's own
  screen, `cleanup-screen.tsx`, if its toolbar has room; otherwise only the list).
- `app/ui/src/locales/en.json` — the texts above, exactly.
- `.github/ISSUE_TEMPLATE/wrong-finding.yml` (new) — an issue form: `name: Fresher offered something it should not
  have`, `description: A finding that was wrong: something the app offered to remove that it should have left alone.`,
  `labels: [wrong finding]`; fields (all `textarea` or `input`, the first four required): "What was offered" (the
  category's name and the item's title as shown), "Its danger level" (dropdown: Safe, Low, Medium, High), "Why it
  should not have been offered", "What happened" (dropdown: I noticed before removing it / I put it back / It was lost),
  "macOS version", "Fresher version" (Settings → About); a markdown note at the top: "Replace your user name in any
  path with `~`. Do not attach files with private data." No product other than Fresher named.
- `.github/ISSUE_TEMPLATE/config.yml` (new) — `blank_issues_enabled: true`.

## Tests (negatives first)

- `recovery-help.test.tsx`: negative — "the dialog is closed until the button is pressed"; "the address is not a
  link"; "the help offers no way to empty the Trash" (as the result and the Trash line, plan 037); positive — "the four parts are shown in order"; "Close returns the focus to the button".
- `result-summary.test.tsx`, `cleanups-screen.test.tsx`: "shows the help button".
- `app/ui/e2e/flows.spec.ts`: "the result offers help to get an item back" (open, read the four headings, close);
  `visual.spec.ts` and `axe.spec.ts`: the help dialog; baselines for the result and Cleanups regenerated and looked at.

## Checks

- `npx vitest run` on the changed test files; `npm run test:cov:ui`; `npm run lint:ts`; `npm run lint:code`;
  `npx prettier --check .github/ISSUE_TEMPLATE`; `npm run check:typos`; `npm run test:e2e:ui`.
- The issue form is valid YAML of GitHub's issue-form schema (keys `name`, `description`, `labels`, `body` with
  `type`, `id`, `attributes`, `validations`); it cannot be previewed offline: the user looks at it on the repository
  after the merge (the user's review list).

## Conclusions

- `product/recovery-help/`: "Removed something you need?" on the result, Cleanups and a cleanup, four parts (put it back, find
  it in the Trash, if you deleted it permanently, tell us what went wrong); the report address is selectable text and the app opens
  nothing. `.github/ISSUE_TEMPLATE/wrong-finding.yml` is the form for "Fresher offered something it should not have".
- The form can only be seen on GitHub after a push; it is on the user's list. The e2e opens the help from the keyboard
  (a click does not focus a button in WebKit).
