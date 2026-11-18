# 019/01/01 — Interests that match only bundle-id names

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** core, scanner, testing, docs

An anchored interest's last `*` may match only a name shaped as a bundle id, so a whole claim of `Caches/*` never
swallows `Caches/JetBrains`, `Caches/Homebrew` or `Caches/Yarn` (the other modules' places). The design:
[`../00-design.md`](../00-design.md), section 1.1.

## Files

- `crates/core/src/system/ids.rs` — `pub fn is_bundle_id_shaped(name: &[u8]) -> bool`: what `BundleId::new` accepts
  (ASCII letters, digits, `.`, `-`, `_`; at most 255 bytes; no leading or trailing dot; no `..`) **and** at least one
  dot. On bytes, so a name that is not UTF-8 is simply not shaped. `BundleId::new` is not changed (an id without a dot
  stays valid elsewhere). Re-exported from `crates/core/src/lib.rs` (add the line only).
- `crates/core/src/interest.rs` — `Interest` gains `only_bundle_ids: bool`:
  - `pub fn only_bundle_ids(self) -> Result<Self, InterestError>`: refused with the new
    `InterestError::NoStarToShape` for any pattern but `Pattern::Under`, and for an `Under` path whose last segment is
    not exactly `*`.
  - `pub fn matches_only_bundle_ids(&self) -> bool`.
  - `Hash`/`Eq` cover it (derived).
- `crates/scanner/src/matcher/trie.rs` — `Segment` gains `BundleId` (the shaped `*`); `TrieBuilder::insert` takes it
  as a third kind of branch (`Branches.bundle_id`); `build` makes a deterministic node's branches: `literal` (each
  literal target with the `any` targets, and with the `bundle_id` targets when the literal name is shaped), `any` (the
  `any` targets alone) and `bundle_id` (the `any` and `bundle_id` targets together, interned as one node); a name that
  is not UTF-8 still takes no `*` of either kind. `TrieNode` gains `bundle_id: Option<TrieIndex>`.
- `crates/scanner/src/matcher/state.rs` — `child_state`: a literal first; else `bundle_id` when the node has one and
  `is_bundle_id_shaped(name)`; else `any` (UTF-8 only, as today).
- `crates/scanner/src/matcher/compile.rs` — `Segment::parse` is told the interest's shape: an interest with
  `matches_only_bundle_ids` gets `Segment::BundleId` for its last segment.
- `crates/scanner/src/matcher/fingerprint.rs` — the shape is part of each interest's fingerprint, so a cache built
  with a plain `*` is refused (`OtherFingerprint`, one full scan).
- `crates/testing/src/naive_walk.rs` — the reference walk matches a shaped `*` the same way (`is_bundle_id_shaped`).
- `crates/scanner/benches/scan.rs` — `scan_interests()` gains one shaped interest whose place the bench tree holds
  (a folder with both shaped and plain names; the generator's tree is extended if it has none), measured first as a
  plain `*` (before this step's code) and then shaped; the other benches unchanged.
- `docs/architecture/modules.md` — "A module never writes an OS path": `only_bundle_ids()`, with the reason (a whole
  claim of `*` would swallow other modules' places).

## Produces

`is_bundle_id_shaped`, `Interest::only_bundle_ids`, `Interest::matches_only_bundle_ids`, for 02/01 (the module's nine
interests) and 01/02 (which reads the anchored patterns).

## Checks

- `ids.rs`, negatives first: `Notes`, `JetBrains`, `.DS_Store`, `com.example.`, `.com.example`, `com..example`,
  `com example.App` (a space), a name with `/`, a 256-byte name, a name that is not UTF-8, an empty name are not
  shaped; then `com.example.App`, `com.example.App.plist`, `org.example-tool_2.x` are.
- `interest.rs`, negatives first: `only_bundle_ids` on `FolderName`, `MarkerFile`, `Extension`, on `Under(_, "a")`,
  `Under(_, "*/a")` and `Under(_, "a*")` refused (`NoStarToShape`); then `Under(_, "*")` and `Under(_, "HTTPStorages/*")`
  accepted and `matches_only_bundle_ids()` true; a plain interest's false.
- Matcher unit tests (`matcher/tests.rs`), negatives first: under a shaped `Caches/*`, `JetBrains`, `Homebrew`, `Yarn`,
  `go-build`, `.DS_Store` and a non-UTF-8 name match nothing; a deeper interest under a literal (`Caches/JetBrains/*`)
  still matches inside `JetBrains` beside the shaped `*`; a plain `*` and a shaped `*` at one node: a shaped name hits
  both, a plain name only the plain one; a literal that is shaped takes the literal and the shaped branch; then
  `com.example.App` and `com.example.App.plist` match.
- Fingerprint: a shaped and a plain interest of the same pattern give different fingerprints.
- The reference property test (`crates/scanner/tests/reference.rs`): the walk and `NaiveWalk` agree over generated
  trees with shaped and plain names under a shaped interest, a plain one and a literal one at the same place.
- `jetbrains`' and `dev-caches`' fixture tests and goldens unchanged (`cargo nextest run -p fresher-module-jetbrains
  -p fresher-module-dev-caches`).
- The scan bench before and after (`cargo bench -p fresher-scanner --bench scan`), on the same Mac, as quiet as it can
  be (note the load average): within 5 % (rule 4); the numbers, the load and the method in the conclusions. If the Mac
  is busy, say so and leave the retake to 04/02, as 018 did.
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-testing`; clippy; fmt.

## Conclusions

Built (ddd9acd): `is_bundle_id_shaped` (what `BundleId::new` accepts, with at least one dot, on bytes) and
`Interest::only_bundle_ids()` in the core (`InterestError::NoStarToShape` unless the pattern is a `Pattern::Under`
whose last segment is `*`); the matcher compiles a `Segment::BundleId` branch beside the plain `*` (a literal wins, a
shaped name takes the literal and the shaped branch, a non-UTF-8 name takes neither star); the fingerprint encodes the
shape only for a shaped interest (kind byte 4, segment tag 2), so the plain sets keep their fingerprints and no cache is
refused needlessly; `NaiveWalk` applies the same shape. `docs/architecture/modules.md` describes it.

Deviations and calls:

- The shape is applied in `Matcher::compile` (the last resolved `Any` becomes `BundleId`), not in `Segment::parse`: the
  `Anchor` closure sees only the pattern, and changing it would touch both resolvers.
- `BundleId::new` and `is_bundle_id_shaped` share one private byte check; behaviour unchanged.
- `crates/rules/src/rules.rs` (the exhaustive `match` on `InterestError`) and `crates/scanner/tests/macos_reader.rs`
  (the shared test support would be dead code there) were touched, though the brief does not list them.
- "At least one dot" shapes version-like names (`IntelliJIdea2019.3`, `Python3.11`): the decision of 02/02 never treats
  a shaped name that is not installed as proof, and the fixtures have negatives for such names.

Tests (negatives first): 26 in `ids.rs` (names refused: no dot, a leading or trailing dot, `..`, a space, a slash, 256
bytes, non-UTF-8, empty); the interest's refusals and the shape surviving `holding`; seven matcher tests (a plain name
such as `JetBrains`, `Homebrew`, `Yarn`, `.DS_Store` matches nothing under a shaped `Caches/*`; a literal beside it
still leads inside); a fingerprint test; the reference property test over shaped and plain names, in `tests/reference.rs`
and `tests/macos_reader.rs`. A mutation (the shape check replaced by "any non-empty name") failed the property test
and three unit tests. `cargo nextest` on the touched crates: 1144 passed; the jetbrains and dev-caches goldens did not
change.

**Bench (rule 4): no slowdown measured, not proven.** The before and after builds ran in alternating rounds on a busy Mac
(load average 19 to 26, mostly the bench's own threads): `scan/full` at 12 threads 69.60 ms before, 68.97 ms after
(-0.9 %); at 1 thread -0.3 % over four order-swapped rounds (a first round read +7.6 %, noisy); the matcher bench within
noise. Rounds of the same build spread by about 8 %, so the 5 % bar is not proven. The bench's shaped `Data/*` shapes
only a few entries (the generated tree's `Data` holds few shaped names), so it measures the trie and node change more
than the shape check. The bench is to be taken again on a quiet Mac (`npm run bench`), ideally with a shaped interest
over many names; it is on 04/02's checklist.

Parked minors: the `shaped()` test helper is repeated in `matcher/tests.rs`; `compile.rs` falls back to the plain `*`
silently if the last segment is not `Any` (a comment or a `debug_assert!`).
