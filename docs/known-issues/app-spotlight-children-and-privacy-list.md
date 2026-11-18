# Spotlight's answer for a removed app misses a nested app and a privacy-list folder

**Status:** open · **Updated:** 2026-11-18

## Symptom

`app-leftovers` asks Spotlight for a proven id and for each shorter dot-prefix of it (`com.example.App.Helper` also asks
`com.example.App`). Two cases stay unseen:

- The id's own children: a leftover of `com.example.App` is offered when the only app on the Mac is
  `com.example.App.Pro`, kept outside the apps folders, which Spotlight does not index as a nested bundle.
- A folder in Spotlight's privacy list: `mdutil -s` says nothing of it, so an app kept there is not found.

## Cause

The query is by exact id (a wildcard query is not available), and the index state is a per-volume answer.

## What was tried

- Dot-prefixes (plan 019 pre-flight): cover a helper whose app lies elsewhere, not a sibling or child.

## Options left

- Accept it. What is offered is guarded by the proof (a saved state or container of the same id), the days setting,
  the user's own confirmation with the level shown (High for support folders), and the Trash.
- A later query by name beside the id was rejected: the design matches by bundle id only, never by name.
