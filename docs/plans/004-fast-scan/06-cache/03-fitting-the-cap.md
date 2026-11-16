# 004/06/03 — Fitting the cap

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner

## Files

- `crates/scanner/src/codec/fit.rs` — `fit(tree, header, cap) -> Vec<u8>`:
  1. encode; if the body fits, done;
  2. otherwise compute, for every folder, the encoded size of its subtree (one post-order pass, from the encoder's
     own size function) and whether the subtree is collapsible — no hit, no owner, no unreadable or avoided node inside
     it;
  3. collapse collapsible subtrees, the smallest by total allocated size first, until the estimate fits: a collapsed
     folder is written as one node flagged `collapsed` with its totals, folder and file counts; nothing below it;
  4. encode again and check; a body still over the cap (a tree whose hits alone exceed it) is not saved, and the
     reason is logged — the next scan is a full scan, never a wrong one.
- `crates/scanner/src/codec/decode.rs` — a collapsed node decodes as a leaf flagged `collapsed`, with its totals and
  counts; 07/01 makes `TreeState` report it as `NodeKind::Collapsed` with no children (a screen reads it live when
  opened).
- No bench here (05 owns the bench file in this wave): 07/01 adds `cache/fit` on `LARGE` with the cap at 8 MiB and
  at 2 MiB — the time, the share of nodes collapsed, the share of bytes still attributed to an uncollapsed folder.

## Produces

The guarantee that a save never exceeds the cap; collapsed nodes the rescan (07/02) re-reads when a change falls
inside them.

## Checks

- Negative: `a_subtree_with_a_hit_is_never_collapsed`, `a_subtree_with_an_unreadable_folder_is_never_collapsed`,
  `a_tree_whose_hits_exceed_the_cap_is_not_saved`.
- Positive: `a_tree_under_the_cap_is_not_collapsed`, `the_smallest_subtrees_collapse_first`,
  `collapsing_keeps_every_total` (the frozen totals of the decoded tree equal the original's at every remaining node).
- Property: `the_fitted_body_is_never_over_the_cap` (random trees, random caps down to the size of their hits).
- `cargo nextest run -p fresher-scanner`; coverage ≥ 90 %.

## Conclusions

- `crates/scanner/src/codec/fit.rs`: `fit(tree, header, Limits { file, body }) -> Result<Vec<u8>, CodecError>`,
  tests in `fit_tests.rs`. A collapsed folder is the codec's ordinary folder record with `Collapsed`, no children,
  its totals as its own bytes and its folders as its folders not kept as nodes; the decoder of 02 already reads it as
  a leaf (it refuses `Collapsed` with children). 07/01 makes `TreeState` report it as `NodeKind::Collapsed`.
- **The cap is compared compressed** (the user's decision: the cap applies to the file on disk). The fitter
  compresses the body with `miniz_oxide` at the store's level (deflate 1) and accepts it when the compressed length
  plus a reserve of 64 bytes for the envelope (30 bytes today) is within `Limits::file`, and the body itself within
  `Limits::body` (the store's body limit, 64 MiB, which bounds a load's memory). So a home whose body compresses
  under the cap is never collapsed. The scanner takes `miniz_oxide` as a dependency (already in the workspace, MIT
  OR Zlib OR Apache-2.0; `docs/dependencies.md` updated). The port's `cap()` gives `Limits::file`; the body limit is
  not on the port (the core is frozen), so 07/01 passes the store's `CachePolicy` body limit from the composition.
- **The loop**: encode; if it fits, done. Otherwise the candidates — folders other than a root, with something below
  them, whose whole subtree has no hit, no owner, nothing `Partial`, unreadable, avoided or on another volume — are
  sorted by total allocated size, smallest first (deeper first on a tie, so a folder comes before its ancestors).
  Each try aims at a body length (the body limit, and the length the last body's compression ratio brings under the
  file cap, less a twentieth), collapses the next candidates until the estimate reaches it, encodes, compresses and
  checks. After eight tries the last collapses every candidate; a body over the cap even then is `TooLarge`, logged
  with its sizes and not saved. The estimate uses each record's length from the encoder and the length of the
  folder's record written as collapsed; the bytes saved below a folder are carried up its ancestors.
- **Groups across a collapsed folder**: a hard link or clone member held inside a collapsed folder is written as the
  folder's (the folder keeps the member's depth-first place relative to every other node), so each group is still
  credited to the same subtree and every remaining node keeps its total (`collapsing_keeps_every_total`).
- **Measured** (release, the developer's machine; a synthetic tree of 556 000 folders with the heap test's name
  shapes, 12.8 MB of body, 2.7 MB compressed): encode 18 ms, decode 44 ms, compress 25 ms; `fit` to a 2 MiB file
  86 ms, to a 6 MiB body 77 ms. The `cache/fit` bench on `LARGE` stays with 07/01.
- Tests: the step's checks, plus `a_body_over_the_cap_that_fits_compressed_is_not_collapsed`; the property also
  checks that a body limit one byte under the fully collapsed body is refused and that a file cap at the fully
  collapsed body's compressed size is met.
