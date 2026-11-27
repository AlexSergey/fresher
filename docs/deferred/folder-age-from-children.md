# A folder's age from its direct children

**Agreed:** 2026-11-17, by the controller of plan 016 (review of step 02/04) · **Due at:** the run by hand of plan
016 (04/02), or the first report of a removed IDE's data offered while in use

## What is postponed

The `jetbrains` module offers the data of an IDE that is no longer installed once none of its version folders changed
for the setting's days. A folder's age is its own modification time, as the rules' age is: it changes when an entry
is added, removed or renamed directly in it (an IDE's lock and port files at its start, its logs rotated), not when a
file inside a subfolder is written in place.

## What doing it needs

- The age of a folder as the newest of its own time and its direct children's times: the walk reads them already;
  a `Match` would carry the newest (`Match::newest_child` or a field of `EntryFacts`), filled by the aggregator for a
  `Claim::Whole` folder.
- The module's `age` taking it; a fixture whose folder is old but whose child is fresh, kept.

Plan [044](../plans/044-largest-folders/readme.md) keeps each folder's newest change in the scan tree (`Node.newest`).
It is known only after the walk, so it does not serve a module's `Match` during it; the item stays deferred.
