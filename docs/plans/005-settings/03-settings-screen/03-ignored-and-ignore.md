# 005/03/03 — Ignored and Ignore

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** app/ui, core (texts)

## Files

- `app/ui/src/screens/settings/ignored-group.tsx` — every ignored category (its name from the dictionary) and path,
  each with **Restore** (`restore_ignored`); "Nothing is ignored." when both lists are empty.
- `app/ui/src/screens/module/` — **Ignore** on a category section (`ignore` with the category).
- `app/ui/src/product/finding-row/` — **Ignore** on a finding of a path (`ignore` with the finding).
- `app/ui/src/screens/overview/` — "N ignored" with a link to the settings when the overview's lists are not empty.
- `app/ui/src/locales/en.json` — the texts.

## Checks

- Negative: no Ignore on a tool's finding; no note when nothing is ignored; a refused ignore shows the core's text.
- Positive: Ignore sends the category or the finding; Restore sends the entry; the note counts both lists.
