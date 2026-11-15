# 002/04/01 — The core's texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core

## Files

- `crates/core/locales/en.json` — the `core.` namespace: the four danger levels (name and a one-sentence meaning, the
  wording of [`../../../categories/README.md`](../../../categories/README.md)), the three completeness states,
  every `CommandError` code (`core.error.<code>`, plain English with a next step where there is one), the core's
  settings (`core.scan.roots.label`, `.description`), the scan outcomes. Every text follows rule 7 of
  [`rules.md`](../../../rules.md): short, plain, no jargon.
- `crates/core/src/texts.rs` — `core_dictionary() -> Result<Dictionary, DictionaryError>` (`include_str!`), and
  `CORE_KEYS`: every key the core can emit, built from `DangerLevel::ALL`, `Completeness::ALL`, `CommandError::CODES`,
  the core schema and the outcomes.
- `crates/core/src/lib.rs` — `mod texts;` and `pub use texts::core_dictionary;`.

## Produces

The core's dictionary the engine merges in front of the modules' (05/02).

## Checks

- Negative: `every_key_the_core_can_emit_exists`, `no_key_in_the_file_is_unused`,
  `every_placeholder_matches_the_args_the_core_passes` (for `CommandError::text()` of every code).
- Positive: `the_core_dictionary_parses`, `names_every_danger_level`.

## Notes from the implementation

- `Completeness` has no `ALL` (the core is frozen): the tests list its variants through a `match` with no wildcard, as
  `ScanOutcome`'s, so a new variant fails to compile until it has a key.
- `CORE_KEYS` became `core_keys()` inside the test module: the list is built at run time (`format!`, the core schema),
  and nothing outside the tests reads it; a `pub(crate)` item would be dead code in the library.
- The keys: `core.danger.<level>.name` / `.meaning`, `core.completeness.<state>`, `core.error.<code>`,
  `core.scan.outcome.<completed|partial|failed>`, `core.scan.roots.label` / `.description`. The core's settings are
  read from `SettingsSchema::new([])` (the core's part of the schema).
- `every_placeholder_matches_the_args_the_core_passes` checks equality, not a subset: an error text names every arg
  its variant passes. One more negative check, `the_other_texts_have_no_placeholders`: a text with no args has no
  `{name}`.
- `core.error.busy` reads "The app is already running a {running}": the arg is the raw operation name (`scan`,
  `clean`), so the wording works with both.
