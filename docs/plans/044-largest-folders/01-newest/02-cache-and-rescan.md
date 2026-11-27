# 044/01/02 — The cache's format 7 and the rescan

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** scanner

`newest` survives the cache and stays true after a rescan. The design: [`../00-design.md`](../00-design.md),
section 2; agent's calls 3 and 4.

## Read first

01/01's commit; `crates/scanner/src/codec/{header.rs,encode.rs,decode.rs,testing.rs,tests.rs,fit_tests.rs}` (how
`modified` is a delta from the parent's, the round-trip property, the bytes a node); `crates/scanner/src/rescan/
{patch.rs,patch_tests.rs,frozen.rs,testing.rs}` (`copy`, `name_hits`, `reread`, the rigs).

## Files

- `crates/scanner/src/codec/header.rs` — `TREE_FORMAT = 7`; its doc lists what 7 adds (each record's newest).
- `crates/scanner/src/codec/encode.rs` — after `modified`: `write_i64(out, node.newest.wrapping_sub(base_newest))`
  with `base_newest` the parent's `newest` (0 for a root), the frozen (subtree) value; a collapsed record writes its
  own node's value, which is its subtree's.
- `crates/scanner/src/codec/decode.rs` — `Frame` and `Place` carry the parent's `newest` as they carry `modified`;
  `Record.newest` read with `wrapping_add`, refused `OutOfRange` when negative and not `i64::MIN`; `add` calls
  `tree.join_newest(index, record.newest)`. The final `freeze` leaves the values as read.
- `crates/scanner/src/codec/testing.rs` — the node specs and the per-node comparison carry `newest`.
- `crates/scanner/src/rescan/patch.rs` — `copy` joins the cached node's `newest` into the new node; `reread` takes the
  new batch's (`apply` already does, through `Tree::apply`); the module doc says: a copied folder keeps its cached
  newest, the patched tree's `freeze` joins what is below into every ancestor, so an ancestor is never older than the
  truth.

## Interfaces produced

- A cache holds every folder's newest; a patched tree's `newest` is exact for every folder read or walked again and
  below, never older than the truth above.

## Tests (negatives first)

`crates/scanner/src/codec/tests.rs`:
- `mod negative`: `a_body_of_format_6_is_refused` (`OtherFormat { found: 6 }`, the old pinned `found: 5` case moved
  along), `a_negative_newest_is_refused`.
- `mod positive`: `the_newest_survives_a_round_trip` (and the round-trip property, which now compares `newest`),
  `a_collapsed_record_keeps_its_subtrees_newest`.

`crates/scanner/src/codec/tests.rs`: `cache_bytes_per_folder_node_stay_under_32` (was `..._under_24`, raised to the
bar of `docs/testing.md`: the new field costs one byte a node on its synthetic tree, 24.06 bytes).

`crates/scanner/src/rescan/patch_tests.rs`:
- `mod negative`: `a_folder_read_again_forgets_the_time_of_a_file_that_went`,
  `an_ancestor_of_a_folder_whose_new_file_went_is_never_older_than_the_truth`.
- `mod positive`: `a_new_file_makes_every_ancestor_newer`, `an_unchanged_folder_keeps_its_newest`,
  `a_patch_with_nothing_removed_matches_a_full_scans_newest` (every folder's `newest` equal to a full walk's).

## Checks

- `cargo nextest run -p fresher-scanner`
- `cargo nextest run -p fresher-e2e` (the cache's and the rescan's end-to-end tests read format 7)
- `cargo clippy -p fresher-scanner --all-targets -- -D warnings`, `cargo fmt --all --check`

## Docs

None yet: 04/01 updates "The scan tree" and "The cache body" of `docs/architecture/README.md`.

Commit: `feat(scanner): the newest change kept in the cache and by a rescan`.
