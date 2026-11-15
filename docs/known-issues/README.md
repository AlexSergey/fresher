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

| Issue                                                                                | Part | Status | What                                                                                        |
| ------------------------------------------------------------------------------------ | ---- | ------ | ------------------------------------------------------------------------------------------- |
| [Moderate audit advisories in the lint tooling](./repo-moderate-audit-advisories.md) | repo | open   | 9 moderate `minimatch` advisories through typescript-eslint, sonarjs, jsx-a11y-x (dev only) |
