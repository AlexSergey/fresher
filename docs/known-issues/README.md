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

| Issue                                                                                | Part  | Status | What                                                                                        |
| ------------------------------------------------------------------------------------ | ----- | ------ | ------------------------------------------------------------------------------------------- |
| [A FIFO is read as a regular file](./core-fifo-read-as-file.md)                     | core  | open   | a named pipe with a marker's name counts as that marker; the core's entry kinds have no "other" |
| [Moderate audit advisories in the lint tooling](./repo-moderate-audit-advisories.md) | repo  | open   | 9 moderate `minimatch` advisories through typescript-eslint, sonarjs, jsx-a11y-x (dev only) |
| [No licence check of npm dependencies yet](./repo-no-npm-licence-check.md)          | repo  | open   | the restriction promises an npm licence check that does not exist; packages are checked by hand |
| [Stale temporary files in the data folder](./store-stale-temporary-files.md)         | store | open   | a crash during an atomic write leaves a `.<target>.<random>.tmp` file that nothing removes  |
| [The UI keeps every finding of a scan in memory](./ui-findings-kept-in-memory.md)    | ui    | fixed  | the store held every streamed finding; it keeps totals now, lists are paged (plan 003/05)   |
