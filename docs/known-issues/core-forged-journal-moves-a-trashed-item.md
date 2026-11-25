# A forged journal record can send a real trashed item to a place of its choosing

**Status:** open · **Updated:** 2026-11-25

## Symptom

Put back trusts a journal entry only when it landed right inside one of the user's Trash folders and its original
place is the one the history row shows. The journal is a plain file, and a same-user program can write the whole
record, the shown place included. A forged cleanup whose entry is an item the user really trashed (in `~/.Trash`,
with its volume and file id) would, on one click of Put back, move that item to a place the forger chose, for
example a cache folder the next scan offers. Found by the security audit of 2026-11-25 (the fix reader's residual of
REMOVAL-1).

## Cause

Nothing ties a journal record to the app that wrote it: the file is readable and writable by any program of the user.

## What was tried

The audit's fix for REMOVAL-1: an entry must land right inside a real Trash and come from the place shown. It stops
a file from outside the Trash, and a place other than the one shown; not a record forged whole. Kept for 0.1.0
because it needs the file id of an item in a protected folder (a same-user program learns it only for items it
trashed itself, which are harmless), and it is a move the user clicks, never a delete.

## Options left

- An integrity tag on each journal record, keyed by a secret in the user's Keychain.
- Accept `.Trashes/<uid>` only for the user's own uid (today any number is accepted).
