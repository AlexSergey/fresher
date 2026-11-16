# 004/05/05 — Claims and commit

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** engine, scanner

## Files

- `crates/engine/src/claims.rs` (new) — `resolve(findings: &[Finding], registry_order) -> Claims`, pure:
  - findings whose subject is a path take part; a tool object (a Docker image) does not;
  - the deepest finding owns its subtree; an outer finding's counted size is its sizes minus those of the findings
    nested in it (by path components, never string prefixes);
  - two findings of the same path: the higher danger level owns it, then the module registered first, then the
    category id — a total order, so the result never depends on arrival order;
  - `Claims::counted(finding) -> Sizes` and `Claims::paths() -> Vec<ClaimedPath>`.
- `crates/engine/src/results.rs` — category totals add counted sizes; a finding keeps its own sizes for display.
- `crates/engine/src/session.rs` — the engine's interest set holds every registered module's interests (design call
  22); a match of a disabled module is dropped before `on_match`. After every module's `finish`, when the scan was
  not cancelled: `resolve`, then `walk.commit(claims.paths())`, the returned state kept in the results.
- `crates/scanner/src/state/mod.rs` (new) — `TreeState` implements `DiskState` over a frozen `Tree` and its claims
  table: `id`, `taken`, `roots`, `node`, `children` (by size or name, paged), `path`, `unreadable`. `largest_files`
  and `search` return nothing until 08/01.
- `crates/scanner/src/scanner.rs` — `DiskWalk::commit`: records each claimed path's category on its node (looked up
  with `Tree::lookup`), wraps the tree in an `Arc<TreeState>`, keeps it for the cache write (06/02 adds the write)
  and returns it.

## Produces

No double counting in the category totals; the owner category on every node (plan 010 and 012); the `DiskState` the
engine holds.

## Checks

- Negative: `a_nested_finding_is_not_counted_twice`, `a_tool_finding_takes_no_part_in_claims`,
  `a_disabled_modules_match_reaches_no_module`, `a_path_that_only_shares_a_prefix_is_not_nested`
  (`/a/cache` and `/a/cache2`), `a_cancelled_scan_commits_nothing`.
- Positive: `the_deepest_finding_owns_its_subtree`, `the_same_path_goes_to_the_higher_danger_then_the_first_module`,
  `the_committed_state_answers_children_by_size`, `a_claimed_node_knows_its_category`.
- Property: `category_totals_never_exceed_the_sum_of_distinct_paths` (random nested findings: the totals equal the
  sizes of the union of the claimed subtrees); `resolve_does_not_depend_on_finding_order`.
- `cargo nextest run -p fresher-engine -p fresher-scanner`; the demo e2e passes unchanged (its findings are not
  nested).

## Conclusions

- `crates/engine/src/claims.rs`: `resolve(findings, order) -> Claims`, pure; `Claims::counted(finding)` and
  `Claims::paths()` (sorted by path). Same path: the higher danger, then the module registered first, then the
  category id, then the lower finding id (a total order); the loser counts nothing. Nested: an owner counts its sizes
  minus the owners nearest inside it (by `Path::ancestors`, so by components), each size stopping at zero, its own
  completeness kept. A finding about a tool object keeps its own sizes and claims no path.
- `results.rs`: the category totals add each finding's counted size once the claims are kept (`keep_claims`, cleared
  when a scan begins); a finding keeps its own sizes for display. `session.rs`: the interest set holds every registered
  module's interests; a match of a module that is off is dropped before `on_match` (it is not in the plan). After every
  module's `finish`, when the scan was not cancelled, the claims are resolved and kept, and a walk that returned a
  summary is committed with `claims.paths()`. `fresher-engine` takes `proptest` for its tests.
- `crates/scanner/src/state/mod.rs`: `TreeState` (crate-private) over the frozen tree and its table of owner
  categories: `id`, `taken`, `roots`, `node`, `children` (by allocated size then name, or by name; paged), `path`,
  `unreadable` (depth first, a folder not scanned is listed and not descended; `hidden_folders` its folder count).
  `largest_files` and `search` answer nothing until 08/01. An id of another state, or past the last node, is `None`.
- `Scanner::commit` takes the last walk's tree, records each claim's category on its node (`Tree::set_owner`, allowed
  on a frozen tree; a claim on a path that is no node, inside a whole claim say, is left out), numbers the states from 1
  and keeps the last one for the cache write of 06/02. Without a walk to commit: `Commit { NotFound }`.
- Not spelled out, taken the simplest way: `taken` comes from a `Clock` — the system's by default,
  `Scanner::with_clock` for the tests — so no test reads the wall clock.
- Checks: the step's tests (the cancelled scan's check is the existing `a_cancelled_scan_is_not_committed`), plus a
  tool finding counted whole in the overview, the loser of a path counting nothing, a stale node id, a commit with no
  walk; the properties hold (`category_totals_never_exceed_the_sum_of_distinct_paths` checks equality with the union of
  the claimed subtrees). The demo e2e passes unchanged.
