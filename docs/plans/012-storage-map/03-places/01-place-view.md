# 012/03/01 — The place view

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** engine

## Files

- `crates/engine/src/storage/place.rs` (new) — `place_view(state, shares, id, offset, limit) -> Option<PlaceView>`,
  pure over the `DiskState` and the `Shares`:
  - **The descent**: from `id`, while the node is a folder whose largest child (one `children(BySize, 0, 1)` call) is a
    folder holding at least 90 % of the node's allocated size, go down — at most 8 levels (`DESCENT_LIMIT`), never
    into an unreadable, avoided or other-volume folder; `PlaceView.path` lists the node opened and every folder gone
    through. The descent applies only at `offset` 0: a next page continues the folder the first page settled on.
  - **The rows**: one `children(BySize, offset, limit)` page of the last folder of the path, each row with
    `explained = shares.explained(state, row)`; `total` from the page.
  - A node that is not a folder opens as itself, with no rows.
  - Completeness is the tree's: a row with unreadable folders inside is `Partial` (`42 GB+`).
  The constants `DESCENT_SHARE = 90` (percent) and `DESCENT_LIMIT = 8` are named, with a comment on why (a chain of
  single-child folders says nothing on its own).
- `crates/engine/src/storage/mod.rs` — `mod place;`.

## Produces

`place_view` for `Engine::storage_place` (02).

## Checks

- Negative first: `a_folder_with_no_dominant_child_does_not_descend` (two children of 50 %),
  `the_descent_stops_at_an_unreadable_folder`, `the_descent_stops_after_eight_levels` (a chain of 12),
  `a_next_page_does_not_descend_again`, `a_node_of_another_state_is_none`, `a_file_has_no_rows`.
- Positive: `a_folder_that_is_almost_one_child_opens_through_it` (`Library › Containers › com.example.app`, 95 %),
  `every_row_knows_its_explained_bytes`, `rows_come_largest_first`, `a_partial_folder_is_partial`; a **property test**:
  for a random tree, the rows of every page of a folder add up, with its own files, to the folder's size, and each
  row's explained is at most its size.
- `cargo nextest run -p fresher-engine`.

**Changed by 04/05:** the place view takes `descent: Descent` (`through` | `stay`): `through` opens down the folders that are almost all one child, `stay` opens exactly the folder asked (a step of the breadcrumb).

## Conclusions

Built in `989550e`, reviewed clean; the missing negative tests came with the final fix wave.

- `place_view` descends at offset 0 while one child holds at least 90 % of its parent, at most 8 levels, only through
  scanned folders; `explained(state, row)` is asked for the returned rows only.
- Deviations: a zero-size folder never descends; `Descent` (`through` or `stay`) was added in 04/05 so a breadcrumb
  step does not descend again. Final wave tests: stops at an avoided folder and at another volume's folder, no descent
  of a zero-size folder, `Stay` with limit 0 gives the root and no rows.
- Measured: property test (pages add up to the folder); `cargo nextest run -p fresher-engine` 314 passed.
