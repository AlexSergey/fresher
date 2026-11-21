# 024/01/02 — File text and settings in the test worlds

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** testing, sandbox, app, docs

A fixture tree or a case file states a small file's text (a pin, a manager's settings file), and a case file sets a
module's setting for every case of the file (the Node switch, off by default). The design:
[`../00-design.md`](../00-design.md), section 2.

## Files

- `crates/testing/src/tree_description.rs` — `FileSpec` gains `text: Option<String>`; `FixtureError::TextWith { path,
  field }` when `text` comes with `size`, `fill` or `kind`, `FixtureError::TextTooLong { path }` over 4096 bytes; the
  file is written with exactly those bytes (its size theirs, counted in the fill cap). The head comment's list of
  fields names it.
- `crates/sandbox/src/case.rs` — the file-level `[settings]` table: `CaseFile::settings() -> &BTreeMap<String,
  toml::Value>` (a `bool`, an integer or a string), refused for a key that is not `<module>.<name>` in kebab case;
  `text` in `[[case.file]]` comes with the tree description. Merged files (`all`): a key set to two different values is
  refused (`CaseError::SettingTwice { key }`), as a path written twice.
- `crates/sandbox/src/generate.rs`, `layout.rs` — the sandbox writes the settings into its data folder's settings file
  (the same format the app's store reads) before the app starts; none written when the table is absent.
- `app/tauri/tests/sandboxes.rs` — the autotest resolves the engine's settings from the case file's table through the
  registered schema; an unknown key or a value of the wrong kind fails the run with the key named.
- `docs/testing.md` — "Fixture trees": `text`; "Sandboxes": `[settings]`, what it is for and the `all` rule.

## Produces

`text` in fixture homes and case files, `[settings]` in case files, for 02/04 and 03/01.

## Checks

- `tree_description.rs`, negatives first: `text_with_a_size_a_fill_or_a_kind_is_refused`,
  `text_over_four_kilobytes_is_refused`; then `text_is_written_byte_for_byte` (a multi-line UTF-8 text, its size).
- `case.rs`, negatives first: `a_setting_key_off_the_shape_is_refused`, `a_setting_set_twice_in_all_is_refused`; then
  `settings_are_read_for_every_case_of_the_file`, `a_file_without_settings_has_none`.
- The autotest: a case file with `"demo.count" = 3` changes the demo module's count (a test case file under the
  sandbox crate's tests, never shipped); an unknown key fails with its name.
- Every existing case file and fixture home unchanged and their goldens equal.
- `cargo nextest run -p fresher-testing -p fresher-sandbox -p fresher-app`; `npm run sandbox:test`.

## Conclusions

Built as written. A fixture file states `text` (byte for byte, over 4 KiB refused), and a case file states
`[settings]` (keys checked against the settings registry, values boolean, integer or string, written into the
sandbox's data folder in the store's own format, a key twice refused). One deviation: `fresher-sandbox` now depends on
`fresher-store` and `serde_json` to write that file; the crate guard and the architecture table say so. The autotest's
settings cases are inline in the app's sandbox tests, with no case file of their own. The harness fails a run that
names an unknown or mistyped setting.
