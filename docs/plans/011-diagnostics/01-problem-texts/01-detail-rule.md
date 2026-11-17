# 011/01/01 — The detail rule in the core

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
cleaner

## Files

- `crates/core/src/text.rs` — the constant `DETAIL_ARG = "detail"` and its doc: the arg of a technical detail (a
  tool's own message), shown only on demand; no template names it.
- `crates/core/src/cleanup/outcome.rs` — `Failure::ToolFailed { error }` becomes `ToolFailed { detail }`; `text()`
  puts it under `DETAIL_ARG`. The field still holds the tool's first non-empty line of standard error, at most 200
  characters (the cleaner's cut, unchanged).
- `crates/core/locales/en.json` — `core.cleanup.failed.tool-failed` reworded without a placeholder: "The tool that
  made this item could not remove it, so it is still there. Close the app that uses it, then try again."
- `crates/core/src/texts.rs` — two tests over the core's dictionary.
- `crates/core/src/error.rs` — one test over `one_of_each()` (which covers plan 012's `UnknownVolume`).
- `crates/cleaner/src/remove/tool.rs` — builds `ToolFailed { detail }`; its doc says the line is shown on demand,
  never journalled (the journal keeps the status, as today).

## Produces

`DETAIL_ARG` for 01/03 (the UI's disclosure) and 02/02 (the module contract's check of `{detail}`).

## Checks

- Negative: `no_core_template_names_the_detail_arg` (every template of `core_dictionary()`),
  `a_refusal_never_serializes_the_errors_display` (for every error of `one_of_each()`, the JSON of its `Refusal` does
  not contain `error.to_string()`), `a_tools_failure_sentence_holds_no_tool_output` (the rendered template of
  `ToolFailed` is the same whatever the detail).
- Positive: `a_tools_failure_carries_its_line_as_the_detail`, the cleaner's two tests of `run_tool` updated (the first
  line, the 200-character cut).
- `cargo nextest run -p fresher-core -p fresher-cleaner`; the core e2e goldens (`crates/e2e`) do not change, or change
  only in this text's args (reviewed in the diff).

## Conclusions

Built as planned (4eac8fd). `DETAIL_ARG` in `crates/core/src/text.rs`; `Failure::ToolFailed` carries `detail` and its
sentence (`core.cleanup.failed.tool-failed`) names no placeholder; the tool's first error line is cut to 200 characters
and never journalled. Three negative tests keep it: no core template names `{detail}`, the tool sentence holds no tool
output, and no serialized `Refusal` holds its error's `Display` (over `one_of_each()`, so 012's `UnknownVolume` too).
No binding changed: `Failure` reaches the UI only as a `Text`.

