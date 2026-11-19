# 020/01/02 — Interests that match only names with a suffix

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, scanner, testing, docs

`downloads` must sum Safari's `.download` bundles and `Install macOS ….app` whole, and leave every other folder of
the user's in Downloads open: claimed whole, `Under(Downloads, "*")` would make them opaque to the storage map and
to every other module (a project with `node_modules` unpacked there). A `*` that takes only names ending with given
suffixes, as plan 019 made one that takes only bundle ids. The design: [`../00-design.md`](../00-design.md), section
1.2 (agent's call 5).

## Files

- `crates/core/src/interest.rs` — `Interest` gains `suffixes: Vec<String>` (empty by default);
  `pub fn only_suffixes(self, suffixes: Vec<String>) -> Result<Self, InterestError>` and `pub fn name_suffixes(&self)
  -> &[String]`, documented as in the design. Refused with `InterestError::NoStarToShape` (not `Under`, last segment
  not `*`), the new `InterestError::BadSuffix { suffix }` (an empty list, reported with an empty suffix; a suffix
  that does not start with `.`, holds `/` or `\0`, or has an empty part between its dots — `.tar.gz` is valid, `.`
  and `.a..b` are not; a suffix listed twice, ASCII case ignored) and the new `InterestError::TwoShapes`
  (`only_suffixes` on an interest already `only_bundle_ids`, and `only_bundle_ids` on one already suffixed). Stored
  lowercased.
- `crates/scanner/src/matcher/trie.rs` — `Segment::Suffixes(Box<[Box<[u8]>]>)` beside `Segment::BundleId`;
  `Branches` gains `suffixed: Vec<(Box<[Box<[u8]>]>, usize)>` and `TrieNode` `suffixed: Vec<(Box<[Box<[u8]>]>,
  TrieIndex)>`; in `build`, a literal name that ends with one of a branch's suffixes (ASCII case ignored) gets that
  branch's targets too; the node gains, per distinct suffix set, a branch node holding the suffixed targets and `any`,
  as `bundle_id` does; `child_state` (in `state.rs` / `entry.rs`, wherever the walk picks the child node today) takes
  a literal first, then the bundle-id
  branch for a shaped name, then the first suffix branch whose suffixes end the name, then `any`. A name that is both
  bundle-id shaped and suffixed (`com.example.App.app`) reaches both branches' interests: the step makes the node of
  that combination (bundle-id targets, suffixed targets and `any`) or documents why it cannot happen in one folder.
- `crates/scanner/src/matcher/compile.rs` — an `Under` interest with suffixes gets `Segment::Suffixes` as its last
  segment, as `matches_only_bundle_ids` gets `Segment::BundleId`.
- `crates/scanner/src/matcher/fingerprint.rs` — a suffixed `Under` encodes as kind 5 followed by its suffixes,
  length-prefixed: a cache made with other interests is refused once (a full scan).
- `crates/testing/src/naive_walk.rs` — the reference walk's last-name check takes the suffixes too (ends with one,
  ASCII case ignored).
- `crates/scanner/tests/reference.rs` — `suffixed_interests()` beside `bundle_id_interests()`, and the property test
  below; the generator draws names with and without the suffixes, in either case.
- `crates/scanner/benches/scan.rs` — a suffixed interest beside the shaped one (`Downloads` with `.download` and
  `.app` among plain names), run before and after the change on the same Mac.
- `docs/architecture/modules.md` — "What a module sees of the walk": `only_suffixes` beside `only_bundle_ids`, what
  it is for (a whole claim of a few kinds of names in a folder of the user's), and that a plain `*` of the same place
  takes the name too.

## Produces

`Interest::only_suffixes`, `Interest::name_suffixes`, `InterestError::{BadSuffix, TwoShapes}`, for 03/01 (interest 1
of `downloads`).

## Checks

- `interest.rs`, negatives first:
  - `mod negative`: `only_suffixes_refuses_a_pattern_without_a_last_star` (`FolderName`, `Under(Downloads, "a/b")`),
    `only_suffixes_refuses_an_empty_list_and_a_bad_suffix` (`[]`, `[""]`, `["download"]`, `["."]`, `[".a/b"]`,
    `[".a..b"]`), `only_suffixes_refuses_a_suffix_listed_twice` (`[".app", ".APP"]`),
    `an_interest_takes_only_one_shape` (both orders → `TwoShapes`);
  - `mod positive`: `only_suffixes_keeps_its_suffixes_lowercased` (`[".Download", ".tar.gz"]`).
- `crates/scanner/src/matcher/tests.rs`, negatives first: `a_suffixed_star_takes_no_name_without_its_suffix`
  (`report.pdf`, `download`, `.download` alone — a name that is only the suffix is not taken, as the design says the
  name must be longer), `a_plain_name_hits_only_the_plain_star_beside_a_suffixed_one`; then
  `a_suffixed_star_matches_a_name_with_its_suffix_in_any_case` (`movie.download`, `Install macOS Sonoma.APP`),
  `a_suffixed_name_hits_both_a_plain_and_a_suffixed_star`, `a_literal_beside_a_suffixed_star_still_leads_inside`,
  `the_fingerprint_changes_with_the_suffixes`.
- `crates/scanner/tests/reference.rs`: `the_scanner_and_the_reference_agree_on_suffixed_names` (proptest, as the
  bundle-id one).
- The aggregator's behaviour for a folder ending a `Whole` and an `Item` interest (summed whole, both hits emitted, the
  `Item` hit with `Sizes::ZERO`) is asserted once in the scanner's tests:
  `a_folder_matched_whole_and_as_an_item_is_summed_and_reported_twice`.
- The bench: `npm run bench` before (on `main`'s code) and after, on a quiet Mac; the numbers and the difference in
  this step's conclusions (rule 4: at most 5 %).
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-testing`; clippy; fmt.

## Conclusions

- Per the pre-flight ruling, a suffixed `*` takes a name only when it is longer than the suffix, and a name that fits
  several branches (`trip.zip.download` is also shaped as a bundle id; `.gz` ends `.tar.gz`) reaches every interest
  it fits. The suffixes a name ends with are all suffixes of its longest one, so the node keeps one branch per
  distinct suffix (`SuffixBranch`, longest first), each with two interned nodes, without and with the shaped `*`'s
  targets: one node per combination that can happen, linear in the suffixes, not one per subset.
- `suffixed_interests()` and `random_download_tree()` live in `crates/scanner/tests/support/mod.rs`; the property
  test runs with the portable reader (`reference.rs`) and the macOS reader (`macos_reader.rs`). The matcher's own
  property test also draws suffixed interests.
- The bench's suffixed interest is `Data/*` with `.download` and `.app` beside the shaped one: the generated tree has
  no `Downloads`, and a new folder would change the tree between the two runs. None of `Data`'s names ends with a
  suffix, so every one pays the check. On a Mac that was not quiet (small tree, before → after): `matcher/entry`
  25.30 → 25.40 µs (+0.6 %), `scan/full` 1 thread 230.9 → 229.1 ms (−0.8 %), 12 threads 62.7 → 61.9 ms (−1.0 %);
  24 to 48 threads vary by ±10 % between runs of the same code. Within rule 4's 5 %; to be run again on a quiet Mac.
