# 002/01/04 — Events, commands and errors

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core

## Files

- `crates/core/src/event.rs` — `Event` (`#[serde(tag = "type")]`), `ModuleOutcome`, `ScanOutcome`.
- `crates/core/src/command.rs` — the commands' input and output types: `Operation { Scan, Clean }`,
  `EngineStatus { Idle, Scanning { scan }, Cleaning }`, `Overview`, `CategoryTotal` (category, count, sizes, highest
  danger), `ModuleInfo` (id, categories, setting keys), `FindingsQuery` (limit capped at 500), `FindingsPage`,
  `SettingsView`, `SettingUpdate`, `Locale` (`"en"` only).
- `crates/core/src/error.rs` — `CommandError` (`thiserror` + `#[serde(tag = "code")]`), `CommandError::text()` → key
  `core.error.<code>` with the variant's fields as args, `CommandError::CODES` (exhaustive, checked by a `match` in a
  test so a new variant cannot be forgotten in the dictionary).

## Produces

The IPC surface of 05/01–05/04, the event stream of 02/03, the user-facing error shape plan 011 extends.

## Checks

- Negative: `a_findings_query_over_the_limit_is_capped`, `an_unknown_locale_is_refused`,
  `every_error_code_is_in_the_code_list`.
- Positive: `events_serialize_with_a_type_tag`, `command_errors_serialize_with_a_code_tag`,
  `busy_names_the_running_operation_in_its_text_args`, `exports_the_event_union_to_typescript` (the `ts-rs` declaration
  is a union of objects with a `type` field).

## Notes from the implementation

- Tags: `Event` on `type`, `CommandError` on `code`, `EngineStatus` on `state`, `ModuleOutcome` / `ScanOutcome` on
  `kind`; every tag value is kebab-case (`scan-started`, `not-running`), so an error code is a text-key segment.
- `ModuleOutcome::Failed` and `ScanOutcome::Failed` are struct variants, `Failed { reason: Text }`, so they can be
  internally tagged.
- `Overview` is `{ status, last_scan: Option<LastScan>, totals }`, with `LastScan { scan, finished, outcome }` (a type
  the step did not name). `ModuleInfo` is built `From<&ModuleDescriptor>` and lists `<module>.enabled` first.
  `SettingsView::new(schema, resolved)` builds the answer of `get_settings`.
- `FindingsQuery` and `SettingUpdate` name modules, categories and settings by string (`&'static str` ids cannot be
  deserialized); the engine looks them up and answers `UnknownModule` / `UnknownSetting`. `FindingsQuery::limit()` is
  capped at `FindingsQuery::MAX_LIMIT` (500).
- `Locale::parse(&str)` refuses anything but `en` with `CommandError::UnknownLocale`; `Locale` serializes only.
- `CommandError::CODES` is an array in declaration order; `code()` gives a variant's code; the test builds one error of
  every variant through a `match` without a wildcard, so a new variant fails to compile until it is listed.
- An integration test, `crates/core/tests/typescript.rs`, builds the `ts-rs` declaration of every exported type and
  checks that no `u64` / `i64` became `bigint`.
