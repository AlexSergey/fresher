# Known issues

A known issue is a defect we live with for now. Each one records the symptom, the cause, what was tried and why it
failed, and the options left, so that nobody repeats the work. A known issue is not deferred work
([`../deferred/`](../deferred/README.md)): deferred work is agreed and waits for its turn, a known issue has no agreed
fix yet.

## Format

One file per issue, named `<part>-<name>.md` (`core-`, `scanner-`, `rules-`, `store-`, `cleaner-`, `app-`, `ui-`,
`repo-`). Each opens with a status line and the date it was last updated:

```markdown
# <Issue>

**Status:** open | parked | fixed (<link to the change>) · **Updated:** YYYY-MM-DD

## Symptom

## Cause

## What was tried

## Options left
```

When an issue is fixed, its file stays with the status `fixed` and a link to the change.

| Issue                                                                                     | Part    | Status | What                                                                                              |
| ----------------------------------------------------------------------------------------- | ------- | ------ | ------------------------------------------------------------------------------------------------- |
| [Quitting from the Dock during a cleanup does not ask](./app-quit-during-cleanup.md)      | app     | open   | The Dock's Quit cannot be held: the cleanup stops after the item in flight, no question; ⌘Q asks |
| [A setting changed during a scan does not ask for a rescan](./core-settings-changed-during-a-scan.md) | core    | open   | the running scan ends unmarked, so "Rescan to apply" is not shown for a change made during it     |
| [The scan offers an item inside the Trash](./app-scan-offers-an-item-in-the-trash.md) | app     | open   | the cleanup leaves it out as protected; whether the scan should skip the Trash is open            |
| [Spotlight does not see an IDE on a volume it does not index](./app-spotlight-misses-an-excluded-volume.md) | app | open | an IDE on a volume left out of Spotlight is not found; its data is offered as a removed IDE's |
| [A sandbox scan says it was made days ago](./app-sandbox-scan-dated-at-generation.md) | app | open | sandbox mode's clock starts at the generation; the interface counts from the computer's clock |
| [The cleaner acts on a path after checking a handle](./cleaner-act-by-path.md)            | cleaner | fixed  | a permanent delete, an emptying and the growth walk act beneath held folders now (plan 008/04)    |
| [The in-use check sees only the user's own processes](./cleaner-other-users-processes.md) | cleaner | open   | a file held only by another user's process, or opened after the last snapshot, is not seen        |
| [The system's Trash call takes a path](./cleaner-trash-call-by-path.md)                   | cleaner | open   | a parent swapped for a link at the call moves the item behind it; detected after the move         |
| [A FIFO is read as a regular file](./core-fifo-read-as-file.md)                           | core    | open   | a named pipe with a marker's name counts as that marker; the core's entry kinds have no "other"   |
| [Moderate audit advisories in the lint tooling](./repo-moderate-audit-advisories.md)      | repo    | open   | 9 moderate `minimatch` advisories through typescript-eslint, sonarjs, jsx-a11y-x (dev only)       |
| [No licence check of npm dependencies yet](./repo-no-npm-licence-check.md)                | repo    | open   | the restriction promises an npm licence check that does not exist; packages are checked by hand   |
| [The `all` sandbox's checklist is wrong where worlds interact](./repo-sandbox-all-worlds-interact.md) | repo    | open   | one case file's apps change another's cases in `npm run sandbox -- all`; its `EXPECTED.md` does not say so |
| [Stale temporary files in the data folder](./store-stale-temporary-files.md)              | store   | open   | a crash during an atomic write leaves a `.<target>.<random>.tmp` file that nothing removes        |
| [The keyboard e2e test fails at times on a long dialog in Chromium](./ui-keyboard-e2e-dialog-long-body.md) | ui | open | "every story, part 8" at times does not reach the buttons of `Dialog/long-body` in Chromium only |
| [The UI keeps every finding of a scan in memory](./ui-findings-kept-in-memory.md)         | ui      | fixed  | the store held every streamed finding; it keeps totals now, lists are paged (plan 003/05)         |
