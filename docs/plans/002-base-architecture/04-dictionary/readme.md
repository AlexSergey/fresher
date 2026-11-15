# 002/04 — Dictionary

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core, app/ui

The core's own texts in `crates/core/locales/en.json` with the checks that no key is missing or unused; the UI's `t()`
and its chrome dictionary. The module side (a module's `en.json` and its checks) is the kit's (06/03). The design:
[`../00-design.md`](../00-design.md), "The dictionary".

Depends on plan 01. Runs in parallel with 02, 03, 06/01–03 and 07/01. The two steps are independent of each other and
can run in parallel too: 01 touches only `crates/core/` (the one plan of wave 2 allowed to: a new file and two lines in
`lib.rs`), 02 only `app/ui/src/`.

## Steps

| #   | Step                                         | Status      | Shared files                                       |
| --- | -------------------------------------------- | ----------- | -------------------------------------------------- |
| 01  | [The core's texts](./01-core-texts.md)       | ✔️ done     | `crates/core/src/lib.rs` (one `mod`, one `pub use`) |
| 02  | [`t()` in the UI](./02-ui-t.md)              | ✔️ done     | `app/ui/src/app.tsx` (three `t()` calls)            |

## Conclusions

- The core's texts: `crates/core/locales/en.json` (26 keys: danger levels, completeness, the ten error codes, scan
  outcomes, `core.scan.roots`), `fresher_core::core_dictionary()`. The tests build every key the core can emit from its
  enums, `CommandError::CODES` and the core schema, and fail on a missing key, an unused key, or an error text whose
  placeholders differ from the args its variant passes.
- The UI: `t(key: UiKey, args?)` for the chrome (`app/ui/src/locales/en.json`, a missing key is a type error),
  `tText(text)` for the core's `Text`, `setCoreDictionary` for plan 05, `formatSize(bytes, completeness)`. Plural forms
  by `Intl.PluralRules`, sizes in decimal units, dates and counts by `Intl`; a gap shows its key.
- Deviations: `Completeness` has no `ALL`, so the tests list it by an exhaustive `match`; `CORE_KEYS` is a test-only
  `core_keys()`; `app.tsx` uses `t()` so the chrome keys are used. Details in the steps' notes.
