# 008/08/02 — The sandbox run by hand

**Status:** ✔️ done (the run by hand is [deferred](../../../deferred/removal-sandbox-run.md)) · **Version:** 0.5.0 ·
**Date:** 2026-11-16 · **Modules:** app (by hand)

The real Trash, the real process table and the real window, never on real data: a development build (`Reach::Sandbox`)
scans a sandbox folder made for this run, holding folders the `trial` module claims (06/05), and only that folder is
cleaned. The agent prepares the folder and the checklist; the user runs the app and confirms (a cleanup is the user's
own action).

## Files

- `NO_COMMIT/008-sandbox/make-sandbox.sh` (never committed) — builds `~/FresherSandbox/` with the sandbox marker,
  `fresher-trial-cache` and `fresher-trial-kept` folders of a few hundred megabytes (real data, so the freed space can
  be measured), one of them held open by a small script; an exFAT disk image (no Trash, as 01 measured) mounted at
  `~/FresherSandboxImage` with its own marker and one trial folder; and a folder `~/FresherNoMarker` with a trial
  folder and no marker.
- The results go into the conclusions of this step (counts, times, what happened; no other path).

## The checklist

1. `npm run tauri dev`; set the scan roots to the three folders above, and only them; scan: the trial categories are
   found in each.
2. Select both categories, review, confirm to the Trash: the progress, the result ("Moved to the Trash", the measured
   space about 0, the Trash sentence), the items in the Trash.
3. The Cleanups screen: the cleanup, "In the Trash: n of n"; put back one item; then Finder's own **Put Back** on
   another.
4. The held folder is skipped as in use, naming the holder; the folder on the disk image is skipped (`NoTrash`);
   nothing on the image is deleted; the folder in `~/FresherNoMarker` is left out as "Development build".
5. A permanent delete of the remaining trial folders: "Freed", the measured space next to the estimate (after the
   delay 01 measured).
6. Quit during a long cleanup (many small trial folders): the question; **Stop and quit** stops after the current item;
   the history shows the cleanup as stopped.
7. A release build (`npm run build`) starts and scans; it registers neither `demo` nor `trial`, so it offers nothing in
   the sandbox — the release composition is checked by its test, not by cleaning real data.
8. Afterwards: the three folders removed and the image detached by the user, the Trash emptied of its items by the
   user, the scan roots set back.

## Produces

The by-hand part of the acceptance, written in the conclusions.

## Checks

- Every item of the checklist has its result; anything unexpected becomes a fix in its plan or a known issue.
- Nothing outside the three folders changed (the user's word, and the Trash holds only sandbox items).

## Preparation

Done on 2026-11-16 by the agent; the step waits for the user's run. `NO_COMMIT/008-sandbox/make-sandbox.sh` and
`NO_COMMIT/008-sandbox/CHECKLIST.md` (the checklist above with a blank for each result) are written, never committed.
Changed from the files above, by what 01 and the code say:

- **The "no Trash" volume** is an APFS image attached `-readonly`, its items written before (FAT32 and exFAT images no
  longer mount on this macOS; 03's tests do the same). Its marker lies in a folder `sandbox/` of the image, not at its
  root: every mounted volume is a whole folder of the protection list, and a marker never counts there.
- **`~/FresherNoMarker`** holds a `.fresher-sandbox` without the signature: the `trial` module claims a folder only
  right next to a file of that name, so with no file at all nothing would be found, and nothing left out as
  "Development build". A trial folder with no marker at all (`bare/`) and a look-alike (`fresher-trial-cache2`) check
  that nothing else is found.
- **The held folder** is `~/FresherSandbox/held/`, with its own marker; `--hold` holds its file open from a second
  terminal until Ctrl-C.
- **The long cleanup** of item 6 comes from `--add-many N` (2 000 small trial folders by default; a move to the Trash
  takes 0.4–2 ms, so it needs many), run after item 5. Item 6 checks the app menu's Quit (⌘Q) and the window's close,
  which ask, and the Dock's Quit as the known gap
  ([`app-quit-during-cleanup.md`](../../../known-issues/app-quit-during-cleanup.md)).
- **The scan roots** are read-only in the interface until plan 005, so the checklist sets them in `settings.json`,
  with a copy kept to put back.
- `--remove` checks each marker's first line and the script's stamp, detaches the image, removes only the three places
  and the image file (`rm -x`, never across a mount), and leaves the Trash to the user.

## Conclusions

Prepared and not run: the user decided on 2026-11-16 to merge plan 008 (with the built parts of plan 006) into `main`
without the run by hand. The run is [deferred](../../../deferred/removal-sandbox-run.md) to before the first release;
that file summarises the checklist, and the script and the checklist with a blank for each result stay in
`NO_COMMIT/008-sandbox/`. Its results go into this step when it is run. Until then the by-hand items of the acceptance
(the real Trash, put back, Finder's own Put Back, a held folder and a disk image without a Trash skipped, quitting
mid-cleanup) are held only by the automated tests named in the plan's conclusions.
