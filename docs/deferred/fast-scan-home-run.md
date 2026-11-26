# The fast scan measured on the developer's home

**Agreed:** 2026-11-16, by the user (plan 004 merged without it) · **Due at:** 0.2.0 (moved from before the first release by the user, 2026-11-26), or earlier
when a scan feels slow

## What is postponed

The by-hand part of plan 004's [budget run][budget-run]: the release build scanning the developer's real home folder,
read-only, recording counts, times and sizes, never a path. The automated part (generated trees, benches, the footprint
test) is done and meets the targets.

## What doing it needs

The checklist in plan 004's conclusions ([`../plans/004-fast-scan/readme.md`](../plans/004-fast-scan/readme.md)):

1. `npm run build`; the first scan of the home: time, peak memory, the two cache file sizes, the footprint, tree heap
   per node if logged.
2. Optionally the same scan at two threads per core: time and peak memory.
3. A rescan after a minute and after a day of ordinary use: time, changed folders, fallback if any.
4. Quit and restart: the last result shows at once, with its date.
5. Clones: the scan's total allocated bytes against `du -sk` of the same roots.
6. Once after a reboot: a cold first scan, recorded with no target.
7. Then confirm or overrule the decisions taken as recommended on 2026-11-16 (the 10 s target on a warm tree, one
   thread per core, the tree-heap bar kept on its test) and say whether a cold-scan target is wanted.

[budget-run]: ../plans/004-fast-scan/08-queries-and-finish/03-budget-run.md
