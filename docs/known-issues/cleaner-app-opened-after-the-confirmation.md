# An app opened after the confirmation is not asked about again

**Status:** open · **Updated:** 2026-11-22

## Symptom

The uninstall is judged again at confirm: the app must still be closed (the running apps are asked anew), and the
bundle must still be as listed. An app that is opened after that check, between the confirmation and the move, or while
a move of many parts is under way (up to 64 paths), is not asked about again. Its bundle may be moved while it runs, or
its files may move one by one while it writes them.

## Cause

The cleaner checks the open files and the running apps once, in its rehearsal and again right before each item, and
reads the running apps from a snapshot refreshed every two seconds (`docs/architecture/removal.md`, "A file in use").
An item of several paths is checked before its first part moves; the cleaner does not ask again between parts. The
cleaner is frozen in plan 036, so the uninstall adds no check of its own inside the act.

## What was tried

The engine asks the running apps anew at prepare and again at confirm, before the cleaner starts, so the window that
is left is the confirmation's own run: the time from the last ask to the last part moved, a few seconds at most.

## How it is kept safe

Everything moves to the Trash, so a bundle moved while its app runs is put back from Cleanups; the running process
keeps its open files, and macOS lets a running app's bundle be moved. The data an app writes while its folders move is
what the user would lose if the Trash were emptied before putting it back; the Trash is never emptied by Fresher.

## Options left

A check between the parts of an item of several paths (`remove/together.rs`), and a fresh running-apps read in the
cleaner's per-item check, when the cleaner is open to change (the security audit, plan 041).
