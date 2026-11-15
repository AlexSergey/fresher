# User settings

**Since:** 2026-11-15

## Problem

Every Mac and every person is different: what counts as "old", which folders hold projects, which sections a person
never wants to see. Hardcoded defaults either miss things or nag about things the user has already decided on.

## Direction

- A settings screen for the user's preferences, stored in the app's local SQLite database next to the snapshots (one
  place for all of the app's state; nothing leaves the Mac).
- First candidates:
  - the age threshold of a stale project ([`stale-project-cleanup.md`](./stale-project-cleanup.md));
  - the folders scanned for projects;
  - ignored sections and items ([`ignore-list.md`](./ignore-list.md));
  - Trash or permanent delete as the default (permanent delete still confirmed every time — rule 2 of
    [`../rules.md`](../rules.md)).
- Every setting has a plain-English label, a sensible default and a "reset to default".
- A setting can never turn off the explicit confirmation before deletion.

Open questions:

- Export and import of settings (a JSON file) — needed, or overkill for one Mac?
- Do settings change rules' danger levels? Proposed: no — a setting may hide or filter, never lower a level.
