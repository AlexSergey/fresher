# 002/01/01 — Groundwork and domain types

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core, repo

## Files

- Root `Cargo.toml` — `[workspace.dependencies]`: `ts-rs = "=12.0.1"` (default features: `serde-compat`); every other
  crate plan 002 needs is already there (`serde`, `serde_json`, `thiserror`, `toml`, `tempfile`, `tracing`). The
  members do not change here.
- `crates/core/Cargo.toml` — normal dependencies `serde`, `serde_json`, `thiserror`, `ts-rs`; description updated.
- `.commitlintrc.cjs` — scopes `engine`, `macos`, `modules`, `testing` added; `docs/quality.md` — the scope list.
- `crates/core/src/ids.rs` — `ModuleId`, `CategoryId` (validated: kebab-case module id; category `<module>.<name>`),
  `FindingId(u32)`, `FindingKey`, `ScanId(u32)`, `ToolId`, `Timestamp(u64)`.
- `crates/core/src/danger.rs` — `DangerLevel { Safe, Low, Medium, High }`, `Ord`, `DangerLevel::ALL`.
- `crates/core/src/size.rs` — `Sizes`, `Completeness`; `ByteSize` exported as `number`; `Sizes::checked_add` keeps the
  weaker completeness.
- `crates/core/src/subject.rs`, `removal.rs` — `Subject`, `Removal` (`ToolCommand` refuses an argument with a NUL byte
  and carries no shell string).
- `crates/core/src/text.rs` — `TextKey` (dot-separated, kebab-case segments), `Text`, `TextArg`.
- `crates/core/src/category.rs`, `finding.rs` — `CategoryDescriptor` and its five derived text keys; `NewFinding::new`,
  `raise_danger`, `Finding`.
- Every exported type derives `serde` and `ts_rs::TS` (no `#[ts(export)]`: the app's bindings test collects them).

## Produces

The identity, danger, size, subject, removal, text, category and finding types used by every later step.

## Checks

- Negative: `rejects_a_module_id_that_is_not_kebab_case`, `rejects_a_category_id_outside_its_module_namespace`,
  `rejects_an_empty_text_key_segment`, `never_lowers_the_category_danger_level`,
  `rejects_a_tool_command_argument_with_a_nul_byte`, `partial_plus_complete_is_partial`.
- Positive: `danger_levels_order_from_safe_to_high`, `raises_the_danger_level_for_one_finding`,
  `derives_the_five_category_text_keys`, `serializes_text_args_in_key_order`, `exports_byte_size_as_number`.
- Property: `sizes_add_up_or_refuse_an_overflow`.
- `cargo nextest run -p fresher-core`, Clippy and `cargo deny check` pass; a commit with scope `engine` passes
  commitlint.

## Notes from the implementation

- Enum variant names serialize as kebab-case (`"medium"`, `"tool-command"`), so they can be text-key segments
  (`core.error.<code>`). Enums with newtype variants (`Subject`, `Removal`, `TextArg`) are adjacently tagged
  (`{ "kind": …, "value": … }`), a discriminated union in TypeScript.
- `ModuleId::new` also refuses `core` (`IdError::Reserved`): the core's own namespace.
- `CategoryId::new(module, id)` takes the module, so the namespace check is part of building the id; `NewFinding::new`
  takes the item's `identity` and builds the `FindingKey` from the category's module, so the two cannot disagree.
- The five category texts are an enum, `CategoryText`, read with `CategoryDescriptor::text_key(text)`.
- `ToolCommand` is its own validated type inside `Removal::ToolCommand`, so no removal can bypass the NUL-byte check.
- `Timestamp::from_system_time` turns the `Clock` port's `SystemTime` into seconds.
- `Finding` serializes flat (`#[serde(flatten)]`): Clippy refuses a field named `finding` in `Finding`.
- Types with an invariant keep their fields private (constructors and getters); plain records (`Sizes`) have public
  fields. Types holding a `&'static str` id serialize only: what comes from the UI names modules and categories by
  string, and the engine looks them up.
