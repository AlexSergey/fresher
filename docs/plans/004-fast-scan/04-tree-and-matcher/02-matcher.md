# 004/04/02 — The compiled matcher

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/matcher/compile.rs` — `Matcher::compile(interests: &InterestSet, anchor: &dyn Fn(&Pattern) ->
  Option<(PathBuf, Vec<Segment>)>) -> Matcher`:
  - `folder_names: HashMap<Box<[u8]>, SmallList>` (interest indices), `markers`, `extensions`, `siblings` (every
    sibling name of every interest, so a folder's listing is checked once for all of them);
  - a trie of anchored paths: each trie node has `literal: HashMap<Box<[u8]>, TrieIndex>`, `any: Option<TrieIndex>`
    (`*`), `ends: SmallList`. The resolver turns `Pattern::UnderHome("Library/Caches/*")` into the home folder and
    its segments; after plan 009, a known folder into its path. An anchor that does not resolve is skipped and
    logged: its interest matches nothing.
  - `SmallList` is an inline list of up to 4 `u32` (no allocation in the common case); written here, no crate.
- `crates/scanner/src/matcher/state.rs` — `TrieState` (a small `Copy` value: the trie nodes live at this folder,
  usually 0 or 1). `Matcher::root_state(root: &Path)` places a root in the trie (a root above an anchor walks down
  the anchor's literal segments; a root below it starts inside); `Matcher::child_state(state, name)` is one lookup per
  live trie node.
- `crates/scanner/src/matcher/entry.rs` — `Matcher::entry(state, name, kind) -> Hits`: the folder-name, extension and
  anchored hits of one entry, by the semantics of `NaiveWalk` (design section 6); `Matcher::folder(listing) ->
  FolderFacts`: which markers and sibling names the listing holds, found in one pass over its names. A marker is
  checked for validity by the walker (05) only when its name is `CACHEDIR.TAG`.
- `crates/scanner/src/matcher/fingerprint.rs` — `Matcher::fingerprint() -> u64`: `stable_hash` over a canonical
  encoding of every interest (owner, local index, pattern, claim, siblings), so the same modules give the same
  fingerprint on every run and any change of an interest changes it.
- `crates/scanner/src/lib.rs` — `mod matcher;`.

## Produces

The matcher the workers use (05/03) and the fingerprint the cache stores (06/02) and checks (07/01).

## Checks

- Negative (each also asserted against `NaiveWalk` on the same fixture): `a_folder_name_does_not_match_a_file`,
  `an_extension_does_not_match_a_folder`, `a_look_alike_name_does_not_match` (`node_modules2`, `Node_modules`),
  `star_matches_exactly_one_name` (`Library/Caches/*` does not match `Library/Caches/a/b`),
  `an_anchor_that_does_not_resolve_matches_nothing`, `a_marker_that_is_a_folder_is_not_a_marker`.
- Positive: `a_folder_name_matches_at_any_depth`, `an_extension_matches_a_file_and_a_link`,
  `an_anchored_path_matches_under_its_anchor_only`, `a_root_below_an_anchor_starts_inside_the_trie`,
  `siblings_are_found_in_one_pass`, `two_interests_on_one_name_both_hit`.
- Property: `the_fingerprint_is_stable_across_runs` (a fixed vector) and `the_fingerprint_changes_with_any_interest`;
  `the_matcher_agrees_with_the_reference_walk` (random interest sets on random fixture trees: the hits equal
  `NaiveWalk`'s matches).
- A micro-bench (criterion, added with 05/01; until then a test that only counts) of `entry` on a 64-interest set:
  under 50 ns per entry on the developer's machine, recorded in the notes.

## Conclusions

- `crates/scanner/src/matcher/`: `compile.rs` (`Matcher::compile`, the `Anchor` resolver type, `under_home`),
  `trie.rs` (`Segment`, the trie builder and its deterministic form), `state.rs` (`TrieState`, `root_state`,
  `child_state`), `entry.rs` (`entry`, `folder`, `FolderFacts`), `fingerprint.rs`, `small_list.rs` (`SmallList`;
  `Hits` is its alias), and test helpers `testing.rs` (a small walk driven by the matcher over `std::fs`, compared with
  `NaiveWalk`). Crate-private, `#[expect(dead_code)]` until 05.
- **Measured**: `entry` on a 64-interest set takes **28 ns** an entry (release build, the developer's machine; bar 50
  ns), by the by-hand test `entry_takes_under_50_ns` (`#[ignore]`d: it reads the clock). The deterministic test
  `entry_counts_the_hits_of_a_64_interest_set` counts the hits instead. The criterion bench comes with 05/01.
- **The trie is deterministic.** The patterns are first added with their `*` branches, then the trie is rebuilt so
  that each node stands for every pattern node a path can reach (a literal child also takes the `*` branch). So
  `TrieState` is one `u32` (a node or `DEAD`), not a set, and `child_state` is one lookup. The trie starts at the file
  system root: an anchor's own names are literal segments, so a root above, at or below an anchor needs no special
  case (`root_state` walks the root's names down).
- **Semantics of the reference walk**, checked by unit tests and by the property test on random trees: a folder name
  matches a folder or a link; an extension (as `Path::extension` reads it) a file or a link; an anchored path any
  entry, `*` exactly one UTF-8 name; a marker a regular file in the folder's own listing (`folder` gives the marker
  interests, which match the folder itself; the walker attaches them to that folder). An entry with a read error is
  neither a marker nor a sibling.
- **Siblings** are a 64-bit mask per hit, bit `i` for the interest's `i`-th sibling; `FolderFacts::siblings_of`
  gives it for an entry of the listing. An interest with more than 64 siblings is skipped and logged, like an anchor
  that does not resolve (an anchor that is not an absolute path of plain names does not resolve either). The marker
  hits found in a folder's own listing take their sibling masks from the parent's listing, so the walker (05) carries
  the parent's masks in the subfolder's job.
- `claims_whole(interest)` and `checks_cachedir_tag(interest)` tell the walker when to stop matching below a folder
  and when to read a `CACHEDIR.TAG`.
- **The fingerprint** also covers each anchored interest's resolved segments (the anchor's path), not only the
  interests: a moved home folder gives another fingerprint. Every string and list is length-prefixed before
  `stable_hash`.
- **Found: the reference walk finds markers and siblings through the file system** (`symlink_metadata`), so on a
  volume that ignores case (APFS by default) `cachedir.tag` is a `CACHEDIR.TAG` marker for it and `Package.json` a
  `package.json` sibling, while folder names and extensions are compared exactly. The matcher compares every name
  exactly (the design: "exact names"). The random trees of the property test have no two names that differ only in
  case; aligning the reference walk is left to the main session (it lives in `fresher-testing`).
- `RawEntry` cannot tell a FIFO or a device from a regular file (`EntryKind::File`), so a FIFO named like a marker is a
  marker for the matcher and not for the reference walk; the walker's `CACHEDIR.TAG` read refuses it (`Invalid`).
- Dependencies added here: `tracing`; dev `fresher-testing`.
