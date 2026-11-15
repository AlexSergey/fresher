# 002/04/02 — `t()` in the UI

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `app/ui/src/locales/en.json` — the UI's chrome (buttons, headings, empty states), keys `ui.<group>.<name>`.
- `app/ui/src/i18n/types.ts` — `UiKey` (`keyof` the chrome JSON), `Dictionary`, and a hand-written `Text` / `TextArg`
  that mirror the core's; plan 05/01 replaces the hand-written two with the generated ones.
- `app/ui/src/i18n/translate.ts` — `translate(dictionary, text)`: looks the key up, picks `.one` / `.other` by
  `Intl.PluralRules('en')` when a `count` arg is present, fills `{name}` placeholders; a `Size` arg is formatted with
  `Intl.NumberFormat` in decimal units (as Finder does), a `Date` arg with `Intl.DateTimeFormat`, a `Count` with
  grouping. A missing key returns the key itself, so a gap is visible, never blank.
- `app/ui/src/i18n/format-size.ts` — the size formatting with completeness (`42 GB`, `42 GB+`, `—`).
- `app/ui/src/i18n/index.ts` — `t(key: UiKey, args?)` for chrome, `tText(text)` for the core's `Text`.

## Produces

The text functions every component of plan 003 and the UI of 05/04 use.

## Checks

- Negative (`describe('negative cases')`): `returns the key when it is missing`, `leaves an unknown placeholder
  visible`, `falls back to .other when .one is missing`, `formats an unknown size as a dash`.
- Positive: `fills placeholders`, `picks the plural form`, `formats a size in decimal units`, `marks a partial size
  with a plus`.
- `app/ui/src/i18n/chrome-keys.test.ts`: every `UiKey` in `en.json` is used somewhere in `src/` (a key no file names
  fails); a missing key is already a type error (`npm run lint:ts`).
- `npx vitest run src/i18n` passes; UI coverage stays ≥ 90 %.

## Notes from the implementation

- `app/ui/src/app.tsx` uses `t()` for its three strings (`ui.app.title`, `ui.app.connecting`, `ui.app.version`): the
  chrome-keys check needs every key named by a source file, and these are the only texts the UI has yet. The file is
  plan 05's in wave 3; the change is three calls and one import, and `app.test.tsx` is unchanged (same English).
- `UiKey` strips a `.one` / `.other` suffix: a plural chrome text is asked for by its base key, as the core's are.
- The core's dictionary lives in `i18n/index.ts`, set once by `setCoreDictionary` (plan 05 calls it with the result
  of `get_dictionary`); before that `tText` shows the key.
- A size arg is formatted by `formatBytes` (bytes, `kB`, `MB`, `GB`, `TB`, `PB`, powers of 1000, one decimal at most;
  under 1 kB in words: `1 byte`, `999 bytes`). A date arg is `dateStyle: 'medium'` in the user's time zone
  (`Jan 15, 2026`). A count is grouped (`12,345`). A `count` arg that is not of kind `count` picks no plural form.
- `index.ts` exports only `t`, `tText` and `setCoreDictionary`; the types are imported from `i18n/types` and
  `formatSize` from `i18n/format-size` (knip refuses re-exports nobody uses).
- `chrome-keys.test.ts` reads the sources with `import.meta.glob(..., { query: '?raw' })`, test files excluded, and
  counts a key as used when a file names it in quotes.
