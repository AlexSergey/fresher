# 002/06/02 — The matcher

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** rules

## Files

- `crates/rules/src/compile.rs` — `RuleSet::interests() -> Vec<Interest>`, remembering which rule each interest serves.
- `crates/rules/src/evaluate.rs` — `RuleSet::evaluate(&Match, &ModuleContext) -> Option<NewFinding>`: pure, no IO.
  `must-not-match` is checked first and wins; then name, version, age (from the `Clock` port and the match's modified
  time), sibling (from `Match::siblings`, reported by the walk). A symlink is never claimed. The finding's `reason` text names
  the rule.
- `crates/rules/src/glob.rs` — a small matcher of our own for `*` and `?` on one name segment (no regex, no globbing
  crate).
- `crates/rules/src/version.rs` — versions parsed from names (`2026.3`, `2025.2.1`) into numeric parts and ordered;
  `newest_per_product(&[Match])` so a module can tell the current version from stale ones in `finish`.
- `crates/rules/src/cachedir.rs` — `CACHEDIR.TAG` honoured: a folder with a valid tag (the standard signature line) is
  recognized as a cache by a `marker = "CACHEDIR.TAG"` rule; an invalid tag is not.

## Produces

The engine-free evaluation every rule-based module (plans 009+, the first real modules) calls from `on_match`.

## Checks

- Negative: `a_look_alike_name_is_not_claimed`, `must_not_match_wins_over_a_matching_name`,
  `the_current_version_is_not_stale`, `a_symlink_is_never_claimed`, `an_item_younger_than_min_age_is_not_claimed`,
  `a_cachedir_tag_without_the_signature_is_ignored`.
- Positive: `claims_a_matching_folder_with_the_rule_danger`, `older_versions_are_stale_once_a_newer_one_exists`,
  `a_valid_cachedir_tag_is_recognized`.
- Property: `version_order_is_total_and_transitive`, `glob_star_matches_any_suffix`.
- `cargo llvm-cov nextest -p fresher-rules` ≥ 90 % of lines.

## Notes from the implementation

- **`CACHEDIR.TAG` needs one read from disk** — a contract the core does not have: a `Match` says only that the folder
  holds the marker, not what the tag says, and the core is frozen. `evaluate` stays pure for every rule except a
  `marker = "CACHEDIR.TAG"` rule, for which it reads the signature's 43 bytes of the tag and no more (a regular file
  only, never a link; any error means "not a cache"). The clean fix is for the walk (plan 004) to report whether the
  tag is valid, for example as a field of `Match`; then `cachedir.rs` keeps only `is_valid_tag` and the read moves
  to the scanner.
- `newest_per_product` is a method of `RuleSet`, not a free function: a product is one rule and the name with its
  version cut out, and only the rule knows its version pattern. `RuleSet::stale(&[Match])` gives the other side: the
  matches older than the newest version of their product, in their order. A product seen in one version only is
  never stale.
- A version is the last place in the name where the pattern matches with whole runs of digits (`110.25` never yields
  `10.2`); versions compare part by part (`2025.10` is newer than `2025.9`). A run of digits too large for `u64` is no
  version.
- `must-not-match` is checked with case ignored (a case-insensitive disk treats `Com.Apple.x` as `com.apple.x`), the
  recognizing `name` with case kept: in doubt, a rule does not claim.
- An unknown or future modification time is never old enough for `min-age-days`. An `empty-folder` rule and a marker
  rule claim folders only.
- The finding: identity and subject are the matched path; the title is `<module>.rule.<id>.title` with the item's name
  as `{name}`, the reason `<module>.rule.<id>.reason` (it names the rule); the danger is raised to the rule's.
- `fresher-testing` (for `FakeClock` and `FixtureTree`) and `proptest` are dev-dependencies; `Cargo.lock` changes with
  them.
- **Fixed after review: the tag is opened once.** The tag used to be checked with `symlink_metadata` and then opened
  by path, so a link or a FIFO swapped in between was followed or blocked the scan. It is now opened with
  `O_NOFOLLOW | O_NONBLOCK` (`libc`, already in the tree through Tauri, pinned `=0.2.191`) and the opened handle's
  metadata must say "regular file". Tests: `a_fifo_named_like_the_tag_is_not_valid_and_does_not_block`,
  `a_tag_linked_to_a_valid_tag_is_not_valid`.
