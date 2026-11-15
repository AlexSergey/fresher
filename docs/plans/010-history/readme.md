# 010 — History and watch mode

**Status:** 📝 draft · **Version:** 0.6.0 · **Date:** 2026-11-15 · **Modules:** core, store, engine, app/ui

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

[`goal.md`](../../goal.md), section 5: every scan is remembered, and the next one shows which apps and categories grew,
by how much and since when — "an app that keeps accumulating junk is named, not guessed". The plan itself can wait for
the first modules, but its data shape constrains the cache of plan [004](../004-fast-scan/readme.md), so it is drafted
now.

## Goals

1. After every scan: what grew and what shrank since the last one, by category and by app.
2. A timeline of the disk over weeks and months.
3. Any two remembered states can be compared, down to folders where the detail is still kept.
4. History costs the disk almost nothing.

## Non-goals

- Scans in the background: history has points only where the user scanned, and the timeline says so.
- The leak detector and the guardian: later, on top of this.

## Design

### Two kinds of memory

| Kept              | What                                                                       | Size             | How long                         |
| ----------------- | -------------------------------------------------------------------------- | ---------------- | -------------------------------- |
| A **point**       | per scan: totals by category, module, app and top-level folder; the volume's used and free space | a few KB | every scan, for years |
| A **state**       | the folder tree of a scan (the cache file of plan 004)                     | a few MB         | thinned: the last two, then one a week for a month, one a month for a year — inside the footprint budget |

Timelines and "who grew" need only points, so they work for the whole history. Folder-level comparison needs two
states, so it works for the dates where a state is still kept; elsewhere the comparison stops at categories and says so.

### Comparing

- Two states are stored in the same depth-first order, so comparing them is one merge pass: added, removed, changed,
  with the change rolled up to every folder above. No database is needed for it.
- Growth is attributed through the claims of the scan: the answer is "this app grew by 6 GB since 3 November", not
  "this folder grew".
- With the [cleanup journal](../008-safe-removal/readme.md): "cleaned on 3 November, 4 GB came back since" — the
  clearest sign of an app that keeps accumulating junk.
- Ignored sections and items are not tracked ([ignore list](../../ideas/ignore-list.md)); restoring one starts its
  history again.

### What this asks of plan 004

- The cache file records, per folder, the category that claimed it — so an old state stays attributable after rules
  change.
- How many states are kept is a policy of the store, not "two" fixed in the format.
- Each scan emits the totals a point needs.

### In the UI

Growth next to each category and app on the overview; a history screen with the timeline and its gaps; a comparison of
two dates.

## Plans (after approval)

| #   | Plan              | What                                                            |
| --- | ----------------- | --------------------------------------------------------------- |
| 01  | Points            | the record, writing one per scan, growth since the last scan    |
| 02  | States and thinning | the retention policy inside the budget                        |
| 03  | Comparing         | the merge pass, roll-up, attribution, "what came back"          |
| 04  | History screen    | growth badges, the timeline, the comparison                     |

Depends on plans 002, 003, 004 and, for "what came back", 008.

## Ideas this touches

[Storage history](../../ideas/storage-history.md) (becomes this plan),
[snapshots and watch mode](../../ideas/architecture/snapshots-and-watch-mode.md) (proposed decision 10), the
[storage leak detector](../../ideas/storage-leak-detector.md) and the
[cleanup guardian](../../ideas/cleanup-guardian.md) (both read this history later), the
[external drive analyzer](../../ideas/external-drive-analyzer.md) (history per volume).

## Decisions

None yet.

## Agent's calls (to check)

1. **Points for ever, states thinned**: the full detail of old scans is given up to keep the disk clean.
2. **Files, not SQLite** (see plan 002, call 1); revisited only if the queries outgrow a merge pass.
3. **Claims are stored in the state**, so old states stay attributable.
4. **This plan is built after the first real modules**; its three requirements go into plan 004 now.
5. **Target version 0.6.0.**

## Conclusions

Not written yet.
