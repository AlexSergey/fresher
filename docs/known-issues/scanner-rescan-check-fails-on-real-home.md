# The rescan of the real home seems to become a full scan every time

**Status:** open · **Updated:** 2026-11-26

## Symptom

In the dev run records, every scan of the real home (`NO_COMMIT/dev-runs/*_mac.events.jsonl`) ends with
`kind: full`, and no rescan does. The scan that ran gives the right result, but the rescan runs first and its work is
lost: it loads the cache, walks again the changes and the places outside the roots, reads the check's sample, and then
a full walk reads everything anyway. Each scan the user starts takes longer than a full scan alone. The scan's progress
falling back to 0 (`fixed/ui-scan-progress-starts-over.md`) came from this fallback; the bar now holds instead.

## Cause

Likely found on fixture trees, not yet confirmed on the real home. The sampled check
(`crates/scanner/src/rescan/check.rs`) read its folders after the patch, but compared them with the journal's answer
from before the patch: a folder written after the rescan asked the journal, or just before it (the system numbers a
change a little after it is made), differed, and the check took it for a change the journal missed. The real home is
written all the time (caches, application data, logs), and the sample is drawn towards large folders with hits, the
ones written most; one such folder in the sample was enough. Reproduced by
`a_change_made_after_the_journal_answered_and_named_when_asked_again_passes_the_check`
(`crates/scanner/src/rescan/check_tests.rs`), which failed with `CheckFailed` before the fix.

## What was tried

- The fallback is now visible: `WalkSummary::fallback` carries the reason, and for `CheckFailed` the first folder that
  differed (a path inside its root), what differed in it (`CheckDifference`: an entry missing or added, a kind, a
  kept file's size, a part of the folded files' sums, an entry that failed, a folder not identified, not read or
  claimed whole) and how many folders differed. The engine logs it and keeps it for the last scan
  (`Engine::fallback`); the development run record writes it as each scan's `fallback`.
- The check now asks the journal again when a folder differs, and a folder the journal names by then is no missed
  change: the cached tree is patched again with that answer, so the result holds the folder as a full scan reads it.
  A difference the journal still does not name falls back as before
  (`a_change_the_journal_still_does_not_name_when_asked_again_fails_the_check`).
- Ruled out by reading the code: times (the check compares none), folders the walk does not enter (opaque, collapsed,
  avoided, other-volume and unreadable folders are never sampled; the places outside the roots are always walked
  again), the trie state the check reads a folder with (`root_state` follows the same names as the walk), and the
  sibling masks, held entries and access (not compared).
- Confirmed but kept as a fallback, since it is a real change of what the folder counts that no event in the folder
  reports: a small clone whose private size changed because its other copy elsewhere changed or went
  (`a_clone_whose_private_size_changed_unseen_fails_the_check`, `folded: allocated`); by reading, a small hard link
  whose other names went acts the same (`folded: logical`; not tested).

## Options left

- Read the `fallback` of the next real-home dev run record: `null` on the scans after the first confirms the fix; a
  `check-failed` names the folder and what differed. A file the system evicts to iCloud or downloads (`size` or
  `folded: allocated`) and the clone and hard-link cases above are the candidates left.
- A late change named only after the journal's own delay could still be missed by the second ask; waiting a moment
  before it is the next step if the record shows folders the journal names later.
