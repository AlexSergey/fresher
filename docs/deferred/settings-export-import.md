# Settings export and import

**Agreed:** 2026-11-16, delegated to the agent by the user (plan 005, decision 3) · **Due at:** when a user asks to
move settings between Macs, or with the release

## What is postponed

**Export…** and **Import…** in the settings screen. Today the export is the file itself: `settings.json` in the data
folder is plain, versioned JSON a user can copy.

## What doing it needs

- Export: the system's save dialog (the `tauri-plugin-dialog` of plan 005), the file copied as it is.
- Import: the file read through the same versioned reader as `settings.json`, every value checked against the
  schema, and the changes shown (setting, from, to) before they are applied in one save; unknown keys listed, not
  applied.
