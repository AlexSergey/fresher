# 043/01/01 — The flags' types in the core

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core

`BuildFlags` and `LargestScreen`: the typed values every later step reads. The design:
[`../00-design.md`](../00-design.md), section 2; agent's call 1.

## Read first

`crates/core/src/danger.rs` (`HighLevel`, its derives and tests), `crates/core/src/app_info.rs`,
`crates/core/src/lib.rs` (the exports).

## Files

- `crates/core/src/flags.rs` (new) — `pub struct BuildFlags { pub high: HighLevel, pub largest: LargestScreen }`
  (`Clone, Copy, Debug, PartialEq, Eq, Serialize, TS`, `#[serde(rename_all = "camelCase")]`) and `pub enum
  LargestScreen { Shown, Hidden }` (`Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS`,
  `#[serde(rename_all = "kebab-case")]`). Doc comments: the struct is what a build ships with, set before the build
  from `flags.toml` by the app's build script (plan 043), never at runtime; `LargestScreen` says whether the sidebar
  shows Largest.
- `crates/core/src/lib.rs` — `mod flags;` and `pub use flags::{BuildFlags, LargestScreen};`.

## Interfaces produced

- `fresher_core::{BuildFlags, LargestScreen}`; field names `high`, `largest` (the build script's generated literal and
  the guard rely on them).

## Tests (negatives first)

`crates/core/src/flags.rs`:
- `mod negative`: `largest_refuses_an_unknown_value` (`serde_json::from_str::<LargestScreen>("\"off\"")` is an error).
- `mod positive`: `largest_serializes_as_kebab_case` (`"shown"`, `"hidden"`, and back), `build_flags_serialize_as_the_ui_reads_them`
  (`{"high":"withheld","largest":"hidden"}`).

## Checks

- `cargo nextest run -p fresher-core`
- `cargo clippy -p fresher-core --all-targets -- -D warnings`, `cargo fmt --all --check`
- The types are not in the bindings yet (step 03 adds them), so `bindings.generated.ts` does not change.

## Docs

None: no observable effect yet (step 02 describes the flags).

Commit: `feat(core): the build flags' types`.
