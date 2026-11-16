# 004/08/02 — The engine's queries

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** engine

## Files

- `crates/engine/src/disk.rs` (new) — the engine's read-only API over the last committed `DiskState`, each answering
  `CommandError::NoScan` when there is none, and refusing a `NodeId` of another state (`CommandError::StaleNode`) —
  both added to the core with their texts in 02/03:
  - `Engine::disk_roots()`, `disk_node(id)`, `disk_children(id, order, offset, limit)` (limit capped at 500),
    `largest_files(limit)`, `search_names(query, limit)`, `unreadable_places(limit)`.
  - The state is read under its `Arc`, never under the engine's state lock, so a long query never blocks a scan's
    start; a scan that commits replaces the `Arc` and the old state lives until its last reader drops it.
- No IPC command is added (design call 32): plans 003 and 012 add the commands with the screens that use them.

## Produces

The API plans 003 (scan screen), 006 (what could not be read) and 012 (the storage map) call.

## Checks

- Negative: `queries_before_any_scan_answer_no_scan`, `a_node_of_an_older_state_is_stale`,
  `a_query_does_not_block_a_new_scan` (a slow fake state held by one thread while another starts a scan).
- Positive: `children_page_by_size`, `search_names_answers_from_the_last_state`,
  `unreadable_places_lists_the_top_folders_with_their_reason`, `a_new_scan_replaces_the_state`.
- `cargo nextest run -p fresher-engine`.

## Conclusions

- `crates/engine/src/disk.rs`: `Engine::disk_roots() -> Vec<NodeInfo>`, `disk_node(id) -> NodeInfo`,
  `disk_children(id, order, offset, limit) -> NodePage`, `largest_files(limit)`, `search_names(query, limit)`,
  `unreadable_places(limit)`. Each takes the last state's `Arc` under the results lock (`Engine::disk_state`) and reads
  after the lock is let go; `NoScan` before any commit; a `NodeId` of another state is `StaleNode`. Every limit is
  capped at 500 (`FindingsQuery::MAX_LIMIT`), `unreadable_places` and `largest_files` too.
- **Calls taken**: `disk_roots` answers the roots' nodes, not their ids (the screens need the names and sizes, and a
  second call per root would be a round trip each); an id of the last state that names no node is also `StaleNode`
  (the core has no other error for it, and the UI handles it the same way: reload). No IPC command and no binding
  changed (design call 32).
- Tests (`disk_tests.rs`): `ScriptedWalk::committing(states)` makes the scripted walk commit given states in turn; the
  slow state blocks its `search` until released, while a second scan starts, finishes and commits; the old reader
  still gets its answer from the old state. `a_page_holds_at_most_five_hundred_nodes` holds the cap.
