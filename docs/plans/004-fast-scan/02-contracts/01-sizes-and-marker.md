# 004/02/01 — Sizes and the marker check

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** core, rules, testing

## Files

- `crates/core/src/size.rs` — `Sizes` gains `reclaimable: ByteSize`: the bytes removing the item frees (its private
  size: not shared with a clone or another hard link). `Sizes::complete(logical, allocated)` keeps its meaning with
  `reclaimable = allocated`; a new `Sizes::new(logical, allocated, reclaimable)`. `checked_add` adds all three.
- `crates/core/src/cachedir.rs` (new) — `pub const CACHEDIR_TAG: &str = "CACHEDIR.TAG"`, the signature, and
  `pub fn is_valid_cachedir_tag(head: &[u8]) -> bool`; pure.
- `crates/core/src/interest.rs` — `Match` gains `marker: Option<MarkerCheck>`; `MarkerCheck { Valid, Invalid }`. The
  walk fills it for a `Pattern::MarkerFile` interest whose name is `CACHEDIR.TAG`; `None` otherwise.
- `crates/rules/src/cachedir.rs` — the file read goes; a `marker = "CACHEDIR.TAG"` rule claims a folder only when
  `matched.marker == Some(MarkerCheck::Valid)`. `evaluate` is now pure for every rule; the crate drops `libc` if
  nothing else uses it.
- `crates/testing/src/naive_walk.rs` — the reference walk fills `marker` (it reads the tag's first 43 bytes with the
  same `O_NOFOLLOW | O_NONBLOCK` open the rules used, regular file only) and `reclaimable` (`= allocated`: `std::fs`
  cannot see clones).
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (`UPDATE_BINDINGS=1`): `Sizes` has `reclaimable`.
- `docs/hacks/README.md` — the entry "CACHEDIR.TAG read by the rules" is removed.

## Produces

- `Sizes` with three sizes, used by the tree (04), the codec (06) and the findings.
- `Match::marker` and `is_valid_cachedir_tag`, used by the bulk reader (03/02) and the walker (05).

## Checks

- Negative: `a_tag_without_the_signature_is_not_valid`, `an_empty_tag_is_not_valid` (core);
  `a_marker_rule_does_not_claim_an_invalid_tag`, `a_marker_rule_does_not_claim_a_match_with_no_marker_check` (rules);
  the existing `a_fifo_named_like_the_tag_is_not_valid_and_does_not_block` and
  `a_tag_linked_to_a_valid_tag_is_not_valid` move to `NaiveWalk`'s tests and still pass.
- Positive: `a_tag_with_the_signature_is_valid`, `sizes_add_all_three` (core);
  `a_marker_rule_claims_a_valid_tag` (rules); `the_reference_walk_reports_a_valid_tag` (testing).
- Property: `checked_add_is_associative_and_keeps_the_weaker_completeness` extended to three sizes.
- `cargo nextest run -p fresher-core -p fresher-rules -p fresher-testing`; the bindings test passes;
  `rg "holds_valid_tag|O_NONBLOCK" crates/rules` finds nothing.

## Conclusions

- `Sizes` has `logical`, `allocated`, `reclaimable` and `completeness`; `Sizes::complete` sets `reclaimable =
  allocated`, `Sizes::new(logical, allocated, reclaimable)` tells the private part apart, `checked_add` adds all three.
  The engine's saturating sum of category totals adds `reclaimable` too.
- `fresher-core` exports `CACHEDIR_TAG`, `CACHEDIR_SIGNATURE` (public too: a reader needs its length, 43 bytes) and
  `is_valid_cachedir_tag`; `Match::marker: Option<MarkerCheck>` with `MarkerCheck { Valid, Invalid }`.
- `crates/rules/src/cachedir.rs` is removed rather than emptied: the decision is one line in `evaluate.rs`
  (`matched.marker == Some(MarkerCheck::Valid)`), with the name from the core. `fresher-rules` no longer depends on
  `libc`; `fresher-testing` takes it for the reference walk's `O_NOFOLLOW | O_NONBLOCK` open (already a workspace
  dependency, so no new entry; this branch has no `docs/dependencies.md` yet).
- The reference walk lists a folder as a marker match only when the marker is a regular file, so a FIFO or a link
  named `CACHEDIR.TAG` gives no match at all; the two moved tests also call the walk's tag check directly to show it
  answers `Invalid` and does not block. Two tests were added: an invalid tag is reported `Invalid`, another marker
  carries no check.
- The property test `checked_add_is_associative_and_keeps_the_weaker_completeness` is new (three sizes small enough
  never to overflow); the existing overflow property covers the third size.
- Regenerated with the project's commands: the IPC bindings (`Sizes.reclaimable`) and the golden file
  `fixtures/golden/demo.json` (a `reclaimable` per finding). The UI's demo scan fixture gained `reclaimable`.
- The hack "CACHEDIR.TAG read by the rules" is removed.
