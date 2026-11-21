# 029/01/03 — The journal and put back

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** core, store, engine

Every path of a together item on record before the first moves, and put back that never brings a subject back without
its other parts. The design: [`../00-design.md`](../00-design.md), section 3.

## Files

- `crates/core/src/cleanup/journal.rs` — `JournalItem.more: Vec<JournalPart>`; `pub struct JournalPart { pub path:
  PathBuf, pub identity: ItemIdentity }`; `BeginRecord::new` fills it from the plan item's parts after the subject and
  their identities (empty for any other action).
- `crates/store/src/journal/records.rs` — `VERSION` 2; `ItemForm.more` (a list of `{ path, identity }` in the file's
  own forms, `StoredPath` for the path); converts both ways.
- `crates/store/src/journal/mod.rs` — `MIGRATIONS` gains the step 1 → 2: every begin line's items get `more: []`. A
  version-1 file opens as the previous generation, as the existing rule says.
- `crates/engine/src/cleanup/history.rs` — `put_back` of an item whose begin record has `more`: the entries still in
  the Trash other than the subject (the first `TrashedItem`, whose original is the item's path) in reverse order of
  `trashed`; then the subject, only when every other entry came back (or was never moved); otherwise the subject stays
  in the Trash and the outcome is the first refusal. An item without `more` puts back as today.

## Produces

The journal's version 2 and a together item's put back, for the e2e flows.

## Checks

- `crates/store/src/journal/tests.rs`, negative first: `a_version_one_file_reads_with_no_more_parts`,
  `a_newer_version_than_two_is_read_only`; positive: `a_together_item_round_trips_with_its_parts`,
  `a_part_path_that_is_not_utf8_round_trips`.
- `history.rs`, negative first: `a_together_item_whose_layer_cannot_come_back_keeps_its_subject_in_the_trash`
  (the fake cleaner answers `Occupied` for a part: the subject stays, the outcome is `Occupied`);
  `a_together_item_whose_subject_is_gone_puts_back_the_rest_and_says_gone`; positive:
  `a_together_item_comes_back_parts_first_and_the_subject_last` (the fake records the order);
  `a_partly_moved_item_puts_back_what_went`.
- `cargo nextest run -p fresher-store -p fresher-engine -p fresher-core`; clippy; fmt.

## Conclusions

Built, with the largest change from the design in this step. The journal's version is 2: the begin record lists
every part after the subject (`more`, with identities), a version-1 file migrates by adding `more: []`, and a newer
version stays read-only. Two fields the design did not have came out of the reviews. `went` on an item record says how
many parts moved (a failing part whose entry landed elsewhere counts: it left its place), because a part that moved
with an unknown landing has no trashed entry and would make the item look never moved. `back` on a put-back record
lists the parts brought back so far, so a retry after a partial put back skips them and never stalls on the subject.
Put back brings the other parts back in reverse order, then the subject, and the subject only when every part that went
is back; a part that is gone, replaced or of unknown landing keeps the subject in the Trash. The record check is
fail closed and typed: a `more` list and the part identities must agree in count. Known limit, written in
`docs/architecture/removal.md`: a part put back and then deleted by the user counts as back. The ten e2e journal
goldens were regenerated at version 2 after the module's task found them stale (only the version, `more`, `went` and
`back` lines changed). Review: needed fixes (the landed-elsewhere part was not in `went`), fixed in round 1 and
re-reviewed.
