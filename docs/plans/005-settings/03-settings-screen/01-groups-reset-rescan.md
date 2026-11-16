# 005/03/01 — Groups, reset and rescan

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `app/ui/src/screens/settings/settings-groups.ts` — the groups by the core's `group`, in the design's order; the
  modules' settings under each module's name.
- `app/ui/src/product/settings-row/` — **Reset** when the value differs from the default; it sends the default.
- `app/ui/src/screens/settings/rescan-notice.tsx` — "Rescan to apply" with **Rescan** when the last scan is marked;
  shown on the settings screen and the overview.
- `app/ui/src/state/reduce.ts` — `setting-changed` marks the overview stale, so totals and lists are asked again.
- `app/ui/src/screens/use-motion-setting.ts` — `core.motion` sets `data-motion` on `<html>` (`reduced`) or clears it.
- `app/ui/src/locales/en.json` — the group headings and the notice.

## Checks

- Negative: no Reset on a row at its default; no notice when the scan is not marked; the motion attribute is cleared
  for `system`.
- Positive: the groups render in order; Reset sends the default; the notice's Rescan starts a scan.
