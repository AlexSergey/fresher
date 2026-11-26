# A mounted volume that Spotlight does not index turns off removed-app detection

**Status:** open · **Updated:** 2026-11-18

## Symptom

While any local, browsable volume without a Spotlight index is mounted (a USB stick, a mounted installer image, a
share), the modules `jetbrains` and `app-leftovers` offer nothing for removed apps in that scan, and say so in a
left-out (`spotlight-unread`). It may happen often.

## Cause

`InstalledApps::search_complete` fails closed (plan 019/01/03): an app kept on a volume Spotlight does not index would
look removed, so one volume that is not indexed, or whose state is unknown, keeps everything. A network volume counts
as not indexed without being asked.

## What was tried

- Asking only the volume that holds the home and the apps folders: an app on an external disk would then be offered
  as removed, the case the known issue
  [Spotlight does not see an IDE on a volume it does not index](./fixed/app-spotlight-misses-an-excluded-volume.md) closed.

## Options left

- Walk the top apps folders of the unindexed volume instead of failing: slow on a large disk, and a module must not
  walk outside its places.
- Accept it: it is safe, the left-out says what to do (unmount the volume or index it), and the next scan decides
  again.
