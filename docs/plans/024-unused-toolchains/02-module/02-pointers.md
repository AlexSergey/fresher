# 024/02/02 — What points at a version

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules

The parsers: a version folder's name, a reference, and each file that holds references, resolved to the folders they
keep, `Nothing` or `Unknown`. Pure functions on text and names, so every shape is a unit test; nothing here reads a
file. The design: [`../00-design.md`](../00-design.md), section 5.

## Files

- `modules/unused-toolchains/src/pointers/mod.rs` (new) — `pub(crate) enum Resolved { Keeps(BTreeSet<usize>),
  Nothing, Unknown }` (indices into the installed versions of a row) and `pub(crate) struct Pointers { default:
  Resolved, others: Vec<Resolved> }` with `keeps(index) -> bool` (any `Keeps` naming it), `default_unknown()` and
  `other_unknown()`.
- `modules/unused-toolchains/src/pointers/rustup.rs` (new) — `ToolchainName { channel: Channel, date: Option<Date>,
  triple: Option<String> }` with `Channel { Stable, Beta, Nightly, Version(Vec<u32>) }`;
  `fn folder(name: &str) -> Option<ToolchainName>` (the triple required), `fn reference(text: &str) ->
  Option<ToolchainName>` (the triple optional), `fn is_moving(&ToolchainName) -> bool`, `fn keeps(reference,
  folder) -> bool` (equal channels or `X.Y` over `X.Y.Z`, equal dates, the triple absent or equal);
  `fn settings(text: &str) -> (Option<String>, Vec<String>)` (the default and the overrides' values; `None` default
  for a text that is not TOML or has none), `fn toolchain_file(text: &str) -> FileRef` (`Channel(String)`, `Path`,
  `Unknown`) for `rust-toolchain.toml` and the legacy `rust-toolchain`. TOML read with `toml::Table`, unknown keys
  ignored (a third party's file).
- `modules/unused-toolchains/src/pointers/nvm.rs` (new) — `fn folder(name: &str) -> Option<[u32; 3]>`
  (`v<major>.<minor>.<patch>`); `fn resolve(text: &str, aliases: &BTreeMap<String, String>, installed: &[[u32; 3]]) ->
  Resolved` with the design's order (`system`, `node`/`stable`, a version prefix, `lts/<name>` through
  `aliases["lts/<name>"]`, another alias, otherwise `Unknown`; at most eight steps, a loop `Unknown`); the first line,
  trimmed, is the reference.
- `modules/unused-toolchains/src/pointers/tool_versions.rs` (new) — `fn references<'a>(text: &'a str, keys: &[&str]) ->
  Vec<ToolVersion<'a>>` (`Version(&str)` or `Nothing` for `system`, `ref:…`, `path:…`), comments and blank lines
  skipped.
- `modules/unused-toolchains/Cargo.toml` — `proptest` as a dev-dependency.

## Produces

The references of every pointer and pin, for 02/03.

## Checks

Negatives first in each file.

- `rustup.rs`: `mod negative` `a_custom_or_beta_numbered_name_is_off_shape` (`my-toolchain`, `1.83.0-beta.3-aarch64-apple-darwin`,
  `stable`, `.DS_Store`), `a_bad_date_is_off_shape` (`nightly-2024-1-01-…`), `a_folder_needs_its_triple`,
  `x_y_z_does_not_keep_x_y`, `an_undated_reference_does_not_keep_a_dated_nightly`,
  `another_triple_does_not_keep`, `settings_that_are_not_toml_have_no_default`,
  `a_toolchain_file_without_channel_or_path_is_unknown`; `mod positive` `stable_and_nightly_without_a_date_are_moving`,
  `a_dated_nightly_and_a_version_are_not_moving`, `x_y_keeps_every_x_y_z`, `a_reference_without_a_triple_keeps_every_triple`,
  `settings_give_the_default_and_every_override`, `a_toolchain_file_gives_its_channel_or_path`,
  `a_legacy_one_line_file_is_a_channel`. Property: `keeps(r, f)` implies `r`'s channel and date equal `f`'s (or the
  `X.Y` prefix).
- `nvm.rs`: `mod negative` `a_name_off_v_x_y_z_is_off_shape` (`18.1.0`, `v18`, `v18.1.0-rc.1`, `system`),
  `an_unknown_word_is_unknown`, `an_lts_name_without_its_alias_file_is_unknown`, `a_loop_or_a_long_chain_is_unknown`,
  `system_is_nothing`; `mod positive` `node_and_stable_keep_the_newest`, `a_major_keeps_every_installed_version_of_it`,
  `an_exact_version_keeps_only_itself`, `lts_star_resolves_through_its_alias_files`, `a_user_alias_resolves_to_its_version`.
  Property: `resolve` never keeps an index outside `installed`.
- `tool_versions.rs`: `mod negative` `another_tool_s_line_gives_nothing`, `system_ref_and_path_are_nothing`,
  `comments_and_blank_lines_are_skipped`; `mod positive` `every_version_of_a_key_s_line_is_a_reference`
  (`nodejs 20.1.0 18.19.0`), `node_and_nodejs_both_count_for_nvm`.
- `cargo nextest run -p fresher-module-unused-toolchains`; clippy; fmt; `npm run check:machete`.

## Conclusions

Built as written: pure parsers for rustup's settings and toolchain files, nvm's aliases and version files, and
`.tool-versions`, each reading as little as it needs. Added by the build: a third `.tool-versions` answer, `Unknown`,
for a line naming no version; a triple's first part has no dot, so an off-shape name is not claimed; a dated name on
an `X.Y` channel is refused; a prefix matching nothing installed keeps nothing and still reads as known (the newest
Node version is protected by the design's call 7). The review found a fail-open: a file starting with a byte order mark
dropped its pin. The mark is now stripped in every parser, with tests for CRLF, tabs, blanks, channel over path and
`latest`. A non-text override is returned as a name that never parses, so it reads as unknown.
