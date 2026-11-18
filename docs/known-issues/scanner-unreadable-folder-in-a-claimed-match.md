# An unreadable folder inside a claimed match is not counted as not read

**Status:** open · **Updated:** 2026-11-18

## Symptom

A developer cache with a folder the scan cannot read inside it (npm's `_cacache/content-v2/sha512/ab` at mode 000) is
not offered, which is safe: the module `dev-caches` refuses a cache it read in part. But the user is not told why. The
scan's coverage lists no unreadable folder and `disk_state().unreadable()` is empty; the only trace is the cache's
node in the storage map, which is `Partial` and has no owner. The cache is silently missing from the results.

## Cause

The scanner sums a `Claim::Whole` match without listing it again as a folder of its own, and a folder it fails to read
inside such a match is not counted in the scan's coverage, whether or not the module then claims the match. This is on
purpose in `crates/scanner/tests/reference.rs` ("the one outside the whole claim"): the claim's size and its `inside`
facts are what matter to the module.

## What was tried

Nothing in the scanner. End-to-end flow 5 of plan
[018](../plans/018-developer-caches/03-end-to-end/02-core-e2e.md)
(`crates/e2e/tests/dev_caches.rs`, `a_cache_read_in_part_is_not_claimed_and_its_node_is_partial`) proves the safe half:
the cache is not claimed and its node is `Partial`.

## Options left

- Count an unreadable folder inside a whole-claimed match as not read in the coverage, so the diagnostics list it, and
  add that assertion to flow 5.
- Let the module report a cache it read in part as a left-out with its own reason (needs a reason text and the scanner
  to hand the module the fact: `Inside.complete` already says it).
