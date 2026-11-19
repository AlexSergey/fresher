# "Reopen Fresher" right after a confirmed cleanup cuts it short

**Status:** open · **Updated:** 2026-11-19

## Symptom

The relaunch command is refused while the engine reports a cleanup running. A cleanup confirmed a moment before the
click may not have started yet when the status is read, so the relaunch goes ahead and the cleanup stops early.

## Cause

`shell::relaunch` reads `Engine::status()` once and then shuts the engine down; the check and the shutdown are not one
step, so a cleanup that starts between them is not seen.

## What was tried

Nothing beyond the check. There is no data loss: the engine's shutdown stops after the item in flight, which is
recorded in the cleanup's journal, as quitting from the Dock does ([`app-quit-during-cleanup.md`](./app-quit-during-cleanup.md)).
The user sees the items not yet done as still in place after the relaunch.

## Options left

- Let the engine refuse to start a cleanup once a relaunch has begun (a shutdown flag checked under the same lock).
- Ask the user, as ⌘Q does, when a cleanup is running or just confirmed.
