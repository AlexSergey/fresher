# 017/02/02 — The decision and the setting

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules

The design's section 7, row by row, as a pure function, and `on_match` around it.

## Files

- `modules/temp/src/names.rs` — `is_lock(name)` (the fixed list of row 2: `.X*-lock`, `.X11-unix`, `.ICE-unix`,
  `.font-unix`, `.XIM-unix`, `quota.user`, `quota.group`, `.vfs_rsrc_streams_*`, `.AddressBookLocks`, `.LINKS`,
  `*.lock`, `*.pid`, `LCK..*`) and `is_system(name)` (starts `com.apple.`); bytes, not UTF-8 (a non-UTF-8 name is
  neither).
- `modules/temp/src/settings.rs` — `older_than(settings) -> Days`: the setting `temp.older-than-days`, number, 7 to 30,
  7 by default; a value outside it is refused by the schema, and a value read below 7 counts as 7.
- `modules/temp/src/decide.rs` — `decide(item: &Item, now, days, running: Result<&RunningSet, _>) -> Verdict`, where
  `Item` is the plain values of a `Match` (name, kind, facts, own times, `inside`) and `Verdict` is `LeftOut(reason)`,
  `NotClaimed` or `Offered { age_days }`; rows 1 to 10 in order. The age is `now − newest` with `newest` the
  subtree's (a folder) or the entry's own `max(modified, changed)` (a file, a link); unknown or after `now` is recent;
  offered only when strictly more than `days × 86,400` seconds; a file access (the entry's or `newest_file_access`)
  within the days is recent.
- `modules/temp/src/lib.rs` — `on_match`: the place from `matched.interest` (`user-temp` or `shared-temp`), `decide`,
  then `out.left_out(LeftOutItem { category: temp.old-files, reason, sizes })` or one `NewFinding` (design section 7:
  identity `<place>/<name>`, `Subject::Path`, `Removal::MoveToTrash`, the match's sizes, the title with `{name}`, the
  place's reason with `{days}`, `for_app` when an installed app's bundle id names the item). `finish` has nothing to
  do: every answer is per item, so the walk's order never matters.

## Produces

The module's answers, for 02/03's fixtures and every later test.

## Checks

- Unit, negatives first (`names.rs`): `com.apple` and `com.applex.y` are not system; `lockfile`, `x.locked`, `.X1`,
  `LCK` alone are not locks; a non-UTF-8 name is neither. Positive: each listed name.
- Unit (`decide.rs`, negatives first, each row's keep before its offer): restored keeps everything, even what would be
  offered; a lock name; an `Other` entry and a folder with `special`; another owner, a system flag, `restricted`, a
  folder with `foreign`, a `com.apple.` name; a user lock; a failed running list; a running app named by the item;
  an incomplete inside (not claimed); exactly N days, a future time, an unknown time, a file read yesterday. Then the
  offered cases: N days and one second; an old link; an old folder. Precedence: restored over every row, special over
  system, system over locked, every structural row over age.
- Property (`mod property`): for any times, a later `now` never turns an offered item into a kept one by age; a
  higher `days` never offers what a lower one keeps.
- `settings.rs`: 6, 31 and a non-number refused; 7, 14 and 30 read; the default 7.
- `cargo nextest run -p fresher-module-temp`; clippy.

## Conclusions

Built (d8afd7b, review fixes eb7d0f2): `names.rs`, `decide.rs` and `on_match`; `finish` does nothing. `decide` takes one
`Scan` (restored, now, days, running) instead of loose arguments.

Calls taken while building: an age is unknown, so recent, when either the modification or the status change is unknown;
a folder with no `inside` is not claimed; names are compared ignoring ASCII case (a wider match keeps more); a file
access within the days is inclusive, a future access keeps, and a link's own access counts; the reason's `{days}` is the
item's own age in whole days, not the setting.

The review's ruling, built in the fix round:

- The app match strips one leading dot (`.com.example.Editor.XXXXXX`) and compares bytes, so a name that is not UTF-8 is
  matched too (`names::named_after`, for row 7 and for `for_app`).
- A folder holding an entry named `*.lock`, `*.pid`, `LCK..*` or `.#*` anywhere inside is kept as a lock: a new
  `Inside.lock_named`, joined by the scanner and the reference walk over every listed name, stored by the codec (tree
  format 5). Row 2 gained `.#*` at the top level too.
- A finding's identity keeps the name's UTF-8, doubles a backslash and writes every other byte as `\xNN`, so two names
  never share one; one `PLACES` table builds the interests, identities and reasons.
- The cleaner's pipeline test: an item of a walked place held open at the act is skipped `InUse`.

61 tests of the module at the time, among them two property tests. The design's row 2 and row 7 and the category file
were brought up to date in 04/01. The rescan property test's rare failure seen here is a [known
issue](../../../known-issues/scanner-rescan-property-access-time.md).
