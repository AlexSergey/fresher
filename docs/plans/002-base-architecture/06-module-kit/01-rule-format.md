# 002/06/01 — The rule format

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** rules

## Files

- `crates/rules/Cargo.toml` — description "the shared matcher for declarative rule data; holds no rules itself";
  dependencies `fresher-core`, `serde`, `thiserror`, `toml`.
- `crates/rules/src/format.rs` — the TOML schema, `#[serde(deny_unknown_fields)]`:

  ```toml
  [[rule]]
  id = "caches"                         # unique in the file; findings name it ("why this?")
  category = "example.caches"           # a category of the owning module
  danger = "safe"                       # never below the category's
  removal = "trash"                     # trash | empty-folder
  [rule.look]                           # where: compiles to Interests
  under-home = "Library/Caches/*"       # or folder-name = "node_modules", or marker = "CACHEDIR.TAG"
  [rule.recognize]                      # how: all must hold
  name = "com.example.*"                # a simple glob on the last segment: `*` and `?` only
  version = "{major}.{minor}"           # optional; captures a version from the name
  min-age-days = 30                     # optional
  sibling = "package.json"              # optional; a file next to the item
  [[rule.must-not-match]]               # required, at least one
  name = "com.apple.*"
  ```

- `crates/rules/src/rules.rs` — `RuleSet::parse(module, toml)` → validated rules; `RuleError` (`Toml`,
  `MissingMustNotMatch { rule }`, `DangerBelowCategory { rule }`, `UnknownCategory { rule }`, `DuplicateId`,
  `PathOutOfHome { rule }`, `BadPattern { rule }`).

## Produces

The parsed rule data the matcher (06/02) compiles and modules embed with `include_str!("../rules.toml")`.

## Checks

- Negative: `rejects_a_rule_without_must_not_match`, `rejects_a_danger_level_below_its_category`,
  `rejects_a_category_of_another_module`, `rejects_a_path_that_climbs_out_of_home`, `rejects_an_unknown_field`,
  `rejects_a_duplicate_rule_id`, `rejects_a_glob_with_brackets` (only `*` and `?`).
- Positive: `parses_a_complete_rule`, `the_optional_parts_may_be_left_out`.

## Notes from the implementation

- `RuleSet::parse` takes the owning `&ModuleDescriptor`: the categories and their danger levels come from it.
- **Rule ids are kebab-case** (`RuleError::InvalidId`, a variant the step did not list): a rule's finding texts are
  keyed by its id, `<module>.rule.<id>.title` and `<module>.rule.<id>.reason`, so the module's `en.json` holds them.
- `[rule.look]` takes exactly one of `under-home`, `folder-name`, `marker` and `extension` (the four `Pattern`s of the
  core); two keys or an unknown key are a `Toml` error. Every rule compiles to one interest that claims the item whole
  (`Claim::Whole`).
- Each segment of `under-home` and every name pattern is a glob with `*` and `?` only; `sibling` must be a single name
  (`BadPattern` otherwise). A version pattern needs at least one `{placeholder}`, never two adjacent ones, and no
  digit or brace in the text between them.
- `glob.rs` and `version.rs` start here with their validation; step 02 adds the matching.
- Public API: `RuleSet::{parse, rules}`, `Rule::{id, category, danger}`, `RuleError`.
