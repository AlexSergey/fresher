# 008/06/03 — History and put back

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** engine

## Files

- `crates/engine/src/cleanup/history.rs` (new):
  - `Engine::cleanups(offset, limit) -> CleanupsPage` (limit capped at 100) from the journal; each summary's "in the
    Trash: n of m" comes from `cleaner.locate` over its items, and the page's "waiting in the Trash from cleanups"
    total sums the sizes of the items still there; an unreadable journal answers what it can, with its state;
  - `Engine::cleanup(id) -> CleanupDetail` (`UnknownCleanup` for an id not in the journal): every item, its outcome or
    skip reason, its `TrashState`;
  - `Engine::put_back(cleanup, items)`: a short operation (`Busy` while a scan or a cleanup runs); `None` puts back
    every item still in the Trash; each item through `cleaner.put_back` with the current context, each outcome
    journalled (`put-back` record); `UnknownCleanupItem` for an index the cleanup does not have; the answer is a
    `PutBackReport` with each item's outcome and text;
  - `Engine::reveal_in_trash(cleanup, item)`: the landed path from the journal, still `InTrash`, then plan 006's
    `SystemUi::reveal` (the UI never sees the path).
- `crates/engine/src/engine.rs` — the four methods.

## Produces

The history and put back for the commands (04) and the Cleanups screen (07/04).

## Checks

With `FakeCleaner`, `MemoryCleanupJournal`, the fake `SystemUi` of plan 006 (negative first):

- `put_back_while_cleaning_is_busy`, `an_unknown_cleanup_is_refused`, `an_unknown_item_is_refused`,
  `an_item_gone_from_the_trash_is_not_put_back`, `an_occupied_place_is_reported_and_the_item_stays`,
  `reveal_of_an_item_no_longer_in_the_trash_is_refused`, `a_permanent_cleanups_items_cannot_be_put_back`.
- Positive: `history_is_newest_first_with_trash_counts`, `waiting_in_the_trash_sums_only_items_still_there`,
  `put_back_all_restores_every_item_still_in_the_trash_and_journals_each`,
  `reveal_asks_the_system_ui_for_the_landed_path`.

## Conclusions

- Built as written: `crates/engine/src/cleanup/history.rs` and the four methods on the engine, `cleanups(offset,
  limit) -> CleanupsPage`, `cleanup(id) -> Result<CleanupDetail, _>`, `put_back(id, items: Option<&[u32]>) ->
  Result<PutBackReport, _>`, `reveal_in_trash(id, item)`. Every check of the step passes.
- Decided where the step was silent, each the safer or the simpler way:
  - **Where an item is now** is asked of `cleaner.locate` for its trashed entries (an emptied folder has one per
    entry that went): in the Trash when one entry is, else replaced when one is, else gone. An item put back (a
    `put-back` record of `PutBack`) is not located again: its `trash` is `None`, and putting it back again answers
    `Gone` (**changed by 06's fixes**: `PutBack` again, with no attempt and no record). An item that never went to
    the Trash (deleted, a tool's, skipped) answers `NotTrashed`.
  - **"Waiting in the Trash from cleanups"** sums the allocated sizes of the items still there over the page read
    (at most 100 cleanups, the newest first): the journal port reads one cleanup at a time, and summing the whole
    history at every page would read the file once per cleanup.
  - **An unreadable journal**: `blocked` says `journal` whenever it is not `Ready` (also with no cleanup ports); a page
    that cannot be read is empty; a cleanup that cannot be read keeps the summary the journal gave. A read error of
    `cleanup(id)` is `Internal`.
  - **Put back** holds the state's lock for its whole run (no scan or cleanup starts meanwhile), refuses with
    `CleaningBlocked { reason: journal }` when the journal cannot be written (no put back goes unrecorded), checks
    every index before acting on any (`UnknownCleanupItem`), takes each index once, puts back only the entries still
    in the Trash, and journals every item's answer (a failed record is logged, the answer still given).
  - **Reveal** names an item by its index; an index the cleanup does not have is `UnknownCleanupItem`; a system that
    cannot show it is `Internal`.
