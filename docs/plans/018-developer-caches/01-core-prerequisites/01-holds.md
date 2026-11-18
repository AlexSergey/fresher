# 018/01/01 — What a matched folder holds

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** core, scanner, testing, docs

An anchored interest names entries; the walk reports which of them the matched folder's own listing holds, and
whether a named `CACHEDIR.TAG` is valid. The design: [`../00-design.md`](../00-design.md), section 1.

## Files

- `crates/core/src/interest.rs` — `Interest` gains `holds: Vec<String>`:
  - `pub fn holding(self, names: Vec<String>) -> Result<Self, InterestError>`: each name checked as a sibling's is
    (`check_name`); refused with `InterestError::HoldsNotAnchored` for any pattern but `Pattern::Under`, and
    `InterestError::TooManyHolds { count }` above 64 (a hit carries them as a 64-bit mask, as siblings).
  - `pub fn holds(&self) -> &[String]`.
  - `Match` gains `pub holds: Vec<String>`: which of the interest's names the matched folder itself holds, in the
    interest's order; empty for a file, a link, an `Other` entry, a folder not entered (another volume, another
    user's, not scanned) and an interest that names none.
  - `Match.marker`'s doc: also `Some(Valid | Invalid)` for an interest whose `holds` names `CACHEDIR.TAG` (`Invalid`
    when it is absent, not a regular file, unreadable or without the signature); `None` otherwise, as today.
  - Every `Match { … }` literal of the workspace (the modules' and the engine's tests, `crates/rules`, `crates/testing`)
    gains `holds: Vec::new()`.
- `crates/scanner/src/matcher/` — per interest, its held names compiled like its siblings (a name → interest and bit
  map); a query "the anchored interests that end at this trie state and hold names" for a folder's own job; the
  fingerprint covers the held names, so a cache made with other interests is refused.
- `crates/scanner/src/walk/worker.rs` — in `folder()`, for a job that is not a scan root, whether the folder is an
  opaque root (claimed whole) or a plain folder (claimed as an item): the mask of each such interest over the job's own
  listing (an entry with an error is absent; any kind counts, but `CACHEDIR.TAG` only as a regular file, whose first
  bytes are read with `reader.head` as `own_hits` reads a marker's). Reported with the folder's report (the opaque
  report and the folder report each gain the masks), never a second read of the folder. In a walked place the anchored
  interests match as everywhere, so held names work there too.
- `crates/scanner/src/walk/aggregator.rs`, `crates/scanner/src/tree/` — the masks joined to the folder's hit
  (`Hit.holds: u64`, with `marker` set for a held tag), kept on the node, turned into `Match.holds` names when the match
  is emitted.
- `crates/scanner/src/codec/` — the hit's mask encoded after its siblings; `TREE_FORMAT` 5 → 6 (an older cache is
  refused, the next scan is full).
- `crates/scanner/src/rescan/` — a folder read again recomputes its masks from its fresh listing; a folder copied from
  the cache keeps its hit's; `emit.rs` turns them into names as the aggregator does.
- `crates/testing/src/naive_walk.rs` — the reference walk computes `holds` and the held tag's validity the same way.
- `docs/architecture/modules.md` — "What a module sees of the walk": `holds`, and `marker` for a held tag; the
  interest's `holding`.

## Produces

`Interest::holding`, `Interest::holds`, `Match.holds` and a held tag's `marker`, for 02/01 and every later module
(the stale-project idea may use it for a project's signs).

## Checks

- `crates/core/src/interest.rs`, negatives first: `holding` on `FolderName`, `MarkerFile` and `Extension` refused
  (`HoldsNotAnchored`); an empty name, `a/b`, `.`, `..` and a name with a NUL refused; 65 names refused
  (`TooManyHolds`); then 64 names accepted and `holds()` in order.
- Scanner unit tests (`walk/tests.rs` or a new `walk/holds_tests.rs`), negatives first: a folder without the names
  holds none; an unreadable entry with a held name is absent; a folder named like a held name beside the match is not
  held (it is a sibling); `CACHEDIR.TAG` as a folder, a link, empty, or with a wrong signature is `Invalid`; a held name
  on a file match gives nothing; an interest without names gives an empty `holds` and `marker` `None`; then a whole
  folder holding `index-v5` and `content-v2`; an item folder holding a valid tag (`Valid`); two interests ending at
  the same folder, each with its own names.
- Codec: a tree with masks reads back equal; a format 5 body is refused (`OtherFormat`).
- Rescan (`rescan/patch_tests.rs`): a held name added inside an unchanged-parent folder appears after the rescan; a
  tag rewritten invalid turns `Valid` into `Invalid`; a folder copied from the cache keeps its mask.
- The reference property test (`crates/scanner/tests/reference.rs`): the walk and `NaiveWalk` agree on `holds` and
  `marker` over generated trees with held names and tags.
- The scan bench before and after (`cargo bench -p fresher-scanner`), on the same Mac, quiet: within 5 % (rule 4),
  the numbers in the conclusions.
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-testing -p fresher-rules -p fresher-engine`;
  `cargo nextest run --workspace` once (every `Match` literal); clippy.

## Conclusions

Built (6f39819, d639488): `Interest::holding(names)` and `Match.holds` in the core, refused on any pattern but
`Under` (`InterestError::HoldsNotAnchored`), over 64 names (`TooManyHolds`) or a name `check_name` refuses; the matcher
compiles the held names into one map and `Matcher::held` builds the masks of every interest ending at a folder in one
pass over its listing (an entry with an error is absent, a `CACHEDIR.TAG` counts only as a regular file, an empty map
returns without allocating); the walk reads a held tag once with `reader.head` and sets `marker`; `Hit` gains a mask
(16 to 24 bytes); the codec's `MIN_HIT` is 4 and `TREE_FORMAT` is 6 (an older cache is refused and rescanned in full);
the rescan reads the masks again with the folder's listing and keeps a copied folder's; `NaiveWalk` computes the same.
`docs/architecture/modules.md` describes it.

Deviations and calls:

- A matched folder whose listing was not read (a file, a link, an entry not entered, another volume or user, an
  unreadable folder) has empty `holds` and `marker` `None` (preflight F10); an entered folder's hit is emitted when its
  own report arrives, not its parent's, so the mask is never stale.
- The fingerprint covers the held names, so every interest's fingerprint changed once; harmless beside the format bump.
- The rescan's two splits (F11) are tested: a matched folder copied under a parent read again keeps its mask; a matched
  folder walked again under a copied parent gets the fresh one (reachable in this code only through an unreadable folder,
  which every rescan walks again).
- `Interest::holding` accepts a duplicate name; the module's table refuses it (`ToolsError::DuplicateSign`, 02/01).

Tests (negatives first): the core's refusals; 11 walk tests (`holds_tests.rs`: a missing, unreadable or look-alike
name, a tag absent, a folder, a link, empty or with the wrong signature, an unread folder, then whole and item folders,
two interests at one folder, the 64th name); codec tests (a format-5 body refused, masks survive a round trip, random
masks in the property trees); seven rescan tests; the reference property test in `tests/reference.rs` and
`tests/macos_reader.rs` with five held interests and four tags valid or not; a `NaiveWalk` test. Mutation checks: each
piece of the code, broken on purpose, failed a test. `cargo nextest run --workspace`: 2432 passed, 5 skipped; clippy
and fmt clean.

**Bench (rule 4): inconclusive, not proven.** The before and after runs (`cargo bench -p fresher-scanner --bench scan`,
the small tree) were taken on a busy Mac (load average 5 to 9, `fseventsd` near 96 % CPU, Spotlight indexing): the same
bench moved by 10 % between rounds of the same build, in both directions, over four rounds. What is measured and
sure: `Hit` grew from 16 to 24 bytes, the state bench's tree heap by 4.6 % (2,695,040 to 2,819,288 bytes), the cache
body by one byte per hit (96,498 to 96,614 bytes). The bench has no interest with held names, so only the
empty-holds path is measured. The bench is to be taken again on a quiet Mac (`npm run bench`), ideally with one held
interest added; it is on 04/02's checklist.

Parked minors: the worker computes masks for every interest ending at a state, a cost to watch with many held
interests.
