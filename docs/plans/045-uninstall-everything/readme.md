# 045 — Uninstall everything, and two copies of one app

**Status:** 📝 draft · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, system, engine, testing, sandbox,
e2e, app/ui, docs

A one-file draft with the recommended decisions taken. Asked by the user on 2026-11-26 after finding two copies of one
app on the Apps screen: "can we add an Uninstall button to our Apps that really removes everything? As a separate plan,
with the fix of the same size shown twice." Builds on plan [036](../036-uninstall/readme.md) and needs
[043](../043-build-flags/readme.md) (High offered again).

## What the user saw

Two rows of one app (same bundle id), 26.3 in `/Applications` and an older 26.2 in `~/Applications` left there
months ago, both "takes 1.4 GB". The bundles are about 1 GB each; the rest is the app's data in `~/Library`, shared
by both copies and counted on each row. The newer copy said "Removing this app needs an administrator", although its
bundle is the user's and `/Applications` is writable by the `admin` group the user is in.

## How it is today

- The app's page offers **Uninstall…** with two choices (plan 036): "Remove the app, keep its settings and data" and
  "Remove the app and everything it left". The second is High, so while High is withheld (plan 042) it is refused and
  not shown. Plan 043 offers High again; this plan makes sure the whole path works and is easy to find.
- Two copies of one id are already safe: the shared data is marked `shared` and kept when one copy goes. But each
  row's size counts the shared data again, and the screen's total does too.
- Apps that truly need an administrator stay not offered (the no-administrator rule,
  [`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)): Show in Finder only. Not changed.

## Goals

1. **Uninstall in sight**: an offered app's row has an **Uninstall…** button, not only its page; the choice of what
   goes (keep settings, or everything) is made in the confirmation, "Remove everything" shown with its High mark and
   its list of files.
2. **"Everything" really is everything Fresher can prove**: the bundle, its caches, support folders, preferences,
   containers, group containers of its team that no other installed app uses, saved state, logs, launch agents of the
   user; one item, moved to the Trash together, put back together (plan 036's `MoveTogether`). Each kind's list is
   checked against a real app on the developer's Mac.
3. **Two copies, counted once**: the copy macOS opens (the newest version; at equal versions the one in
   `/Applications`) carries the shared data; the other row shows only what is its own (its bundle) and says "An older
   copy of <app> <version>. Its settings are shared and stay." The screen's total counts the shared data once.
4. **The right reason**: find why a user-owned bundle in `/Applications` read as needing an administrator (the user's
   groups not read, a part of the footprint owned by root, or another fact), fix the cause, and when a part does need
   an administrator, the text names that part.

## Decisions taken (recommended)

- The older copy is offered as "Remove this copy" (its bundle only), never with the shared data.
- The no-administrator rule stays; the plan does not add an administrator's removal.
- No new module: the engine's apps view, the core's footprint and the UI change.

## Plans

1. Findings first: reproduce both cases in the sandbox (twins with shared data; a user's bundle in a group-writable
   apps folder) and find the cause of the wrong reason, written down before any fix.
2. The core and the engine: twins' sizes and the "older copy" row; the reason fix; tests and goldens.
3. The UI: Uninstall… on the row, the choice in the confirmation, the twin's text; mockup first; e2e, axe, visual.
4. Finish: test cases, the categories doc, the user's walk with this very app.

## Agent's calls to check

- Which copy carries the shared data (newest version, then `/Applications`).
- The choice of what goes moves from the page into the confirmation.
