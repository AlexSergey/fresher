# 028/01/01 — The list follows an ignore

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** app/ui

A change of an ignore list drops every cached finding page and asks for the rows on screen at once, as a cleanup's
removal does, so an ignored row leaves without the throttled refresh and no page keeps shifted rows. The design:
[`../00-design.md`](../00-design.md), section 1 (agent's call 4).

## Files

- `app/ui/src/state/reduce.ts` — `CoreState.ignoreChanges: number` (comment: "How many times an ignore list changed
  in this session: the finding pages start again (state/findings.ts)."), `0` in the initial state; the
  `setting-changed` case increments it when `event.key.startsWith('core.ignore.')`, in the same branch that clears
  the selection.
- `app/ui/src/state/findings.ts` — `follow` keeps `ignoreChanges` beside `overview`, `removed` and `scan`; when it
  changes (and the scan did not), `reset()` then `refresh()`, as for `removed`.
- `app/ui/src/state/reduce.test.ts`, `app/ui/src/state/findings.test.ts` — the cases below; any state fixture that
  lists every `CoreState` field gains `ignoreChanges: 0`.

## Produces

`CoreState.ignoreChanges`; finding pages that start again on every ignore or restore — for 01/03.

## Checks

- `reduce.test.ts`, negatives first:
  - `describe('negative cases')`: `it('a setting outside the ignore lists keeps the count of ignore changes')`.
  - `describe('positive cases')`: `it('a change of an ignore list counts one ignore change and clears the
    selection')`.
- `findings.test.ts`, negatives first:
  - `describe('negative cases')`: `it('an overview change alone keeps the cached pages until the throttled
    refresh')`.
  - `describe('positive cases')`: `it('an ignore change drops every cached page and asks for the rows on screen at
    once')`.
- `npm run test --workspace app/ui -- src/state`; `npm run lint:ts`; `npm run knip`; `npx eslint` and
  `npx prettier --check` on the changed files.

## Conclusions

Built in 7bf3c77b: `CoreState.ignoreChanges` counts the ignore-list changes, and the finding pages reset and the last
ensured range is asked again at once. Review: approved. Parked minor: the unit test of an ignore change covers one
range only.
