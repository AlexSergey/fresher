# A disk image named after an installed app is offered as its installer by its name alone

**Status:** open · **Updated:** 2026-11-25

## Symptom

`downloads.installers` proves a `.dmg` by its name: the name without its extension names exactly one installed app,
followed only by versions, plain numbers and the words of its table, and the app changed after the file
([the category](../categories/downloads-installers.md)). The user's own disk image named like the app (an encrypted
backup `Notes 2024.dmg` while an app called Notes is installed and was updated since) is offered, at Low, to the
Trash. Found by the security audit of 2026-11-25 (SCAN-6).

## Cause

Nothing in the file is read: the category is designed to prove an installer by its name and dates only, so the scan
stays light and reads no file in Downloads.

## What was tried

Nothing yet. Kept for 0.1.0 because the danger is Low, the item goes to the Trash, and the review shows the file's
real name and size before anything moves.

## Options left

- Read the image's own header (a few bytes): an encrypted or read-write image is no installer.
- Require the quarantine attribute's downloading agent, or a size under a ceiling, besides the name.
