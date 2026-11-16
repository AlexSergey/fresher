# 003/02/03 — Fields

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/primitives/text-field/` — `TextField`: `<label>` + `<input>`, optional `description`, `error` (a message
  under the field, `aria-invalid`, `aria-describedby`, a red outline and the `CircleAlert` icon — never colour alone),
  `disabled`. Focus: the focus ring on the field.
- `design/primitives/search-field/` — `SearchField`: `type="search"` with the `Search` icon, a clear `IconButton` when
  it has text (Escape clears too), `label` required (shown or visually hidden).
- `design/primitives/select/` — `Select<T extends string>`: a native `<select>` restyled (the system menu opens, as on
  every Mac), a `ChevronDown` icon, `label`, `options: readonly { value: T; label: string }[]`.

## Produces

The fields the settings rows (06/05) and later the search of lists use.

## Checks

- Negative: `an invalid field announces its error`, `the clear button is absent when the field is empty`, `Escape on an
  empty search does nothing`, `a disabled select does not change`.
- Positive: `the label names the input`, `typing calls onChange`, `the clear button and Escape empty the search and
  keep the focus in it`, `the select calls onChange with the typed value`.
- Stories: text field empty, filled, focus, invalid, disabled; search empty, with text, focus; select closed, focus,
  disabled; screenshots looked at.
