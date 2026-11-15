# 002/01/05 — The dictionary type

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core

## Files

- `crates/core/src/dictionary.rs` — `Dictionary` (a flat, sorted map `TextKey → String`): `Dictionary::parse(namespace,
  json)` (a flat JSON object; every key inside the namespace, every value a non-empty string), `merge` (refuses a
  duplicate key), `get`, `keys`; `DictionaryError` (`NotFlat`, `OutsideNamespace { key }`, `Duplicate { key }`,
  `EmptyText { key }`, `Json`). Placeholders `{name}` in a text are listed by `placeholders(key)`, so a test can check
  the args a finding passes.

## Produces

What the engine merges (02/01), what the core's own `locales/en.json` is parsed with (04/01), what the module harness
checks texts with (06/03).

## Checks

- Negative: `rejects_a_key_outside_the_namespace`, `rejects_a_nested_object`, `rejects_an_empty_text`,
  `rejects_a_duplicate_key_on_merge`.
- Positive: `parses_a_flat_dictionary`, `merges_two_namespaces`, `lists_the_placeholders_of_a_text`.
- After this step: `cargo llvm-cov nextest -p fresher-core` shows at least 90 % of lines; plan 01's conclusions are
  written in its readme.

## Notes from the implementation

- `DictionaryError` has two more variants than the step named: `InvalidKey { key }` (a key that is not a text key,
  `demo.A`) and `NotFlat { key: Option<String> }` carries the key whose value is not a string (`None` when the
  document is not an object). `Json { reason }` keeps the parser's message as a string, so the error stays
  `Clone + PartialEq`.
- A key written twice inside one file is refused too (`Duplicate`): the file is read entry by entry, not through a map
  that would keep only the last value.
- `merge` checks every key before adding any: a refused merge leaves the dictionary as it was.
- `placeholders(key)` returns `Option<BTreeSet<&str>>`: `None` for a missing text, the sorted unique names otherwise
  (`{}` is not a placeholder).
- `Dictionary` serializes as a flat object; in TypeScript `{ [key in TextKey]: string }`.
