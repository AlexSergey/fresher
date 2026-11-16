# The first real cleanups, by hand, in a sandbox folder

**Agreed:** 2026-11-16, by the user (plan 008 merged without it) · **Due at:** before the first release, or earlier
when a cleanup behaves unexpectedly

## What is postponed

The by-hand part of plan 008's acceptance, its step [08/02][sandbox-run]: a development build (`Reach::Sandbox`)
cleaning a sandbox folder made for the run, with the real Trash, the real process table and the real window, never on
real data. The user runs the app and confirms every cleanup; the agent prepares the folders and writes the results
(counts, times, what happened; no other path) into the step's conclusions. Everything automated is done: the property
tests, the compile-fail tests, the core e2e, the UI flows on the mocked core, and the macOS layer's tests on disk
images.

## What doing it needs

A script and a checklist written for the run live locally in `NO_COMMIT/008-sandbox/` (`make-sandbox.sh`,
`CHECKLIST.md`), never committed; if they are gone, they are made again from the steps below.

1. **Prepare**: `~/FresherSandbox` with the sandbox marker (its first line the core's `SANDBOX_SIGNATURE`), two trial
   folders of a few hundred megabytes, a look-alike that must never be found, and `held/` with its own marker and a
   trial folder held open from a second terminal; `~/FresherSandboxImage`, a read-only APFS image (FAT32 and exFAT
   images no longer mount) with a trial folder in a `sandbox/` folder beside a marker; `~/FresherNoMarker` with a
   marker file that lacks the signature, a trial folder, and a bare trial folder with no marker. The scan roots are
   set to the three folders in the app's `settings.json` (read-only in the interface until plan 005), a copy kept.
2. **Scan** with `npm run tauri dev`: the five trial folders are found; the look-alike and the bare folder are not.
3. **Move to the Trash**: the confirmation lists what will not be removed and why (the held folder in use, naming its
   holder; the image's folder; the unmarked folder as "Development build"); the progress; the result "Moved to the
   Trash", the measured space about 0, the Trash sentence; the items are in the Trash.
4. **Put back**: the Cleanups screen shows "In the Trash: n of n"; one item put back from the app, another with
   Finder's own Put Back; the history then shows it no longer in the Trash.
5. **Never touched**: the held folder, the image's folder (skipped, no Trash), the unmarked folder, the look-alike and
   the bare folder are all still in place.
6. **Permanent delete** of the trial folders left: "Freed", the measured space next to the estimate; nothing in the
   Trash.
7. **Quit during a long cleanup** (a few thousand small trial folders): ⌘Q asks, **Keep cleaning** changes nothing,
   **Stop and quit** stops after the current item and the history shows the cleanup as stopped; closing the window
   asks the same; the Dock's Quit does not ask (the known issue
   [`app-quit-during-cleanup.md`](../known-issues/app-quit-during-cleanup.md)) and the next start shows the cleanup
   as interrupted.
8. **Release build** (`npm run build`): it starts and scans and offers nothing in the sandbox (neither `demo` nor
   `trial` is registered); nothing is confirmed in it.
9. **Afterwards**: the sandbox folders removed and the image detached, the sandbox's items emptied from the Trash by
   hand, the scan roots set back; nothing else changed.
10. Anything unexpected becomes a fix in its plan or a known issue. Also wanted when at hand: a run with Full Disk
    Access, the sound of the Trash call, a network share, macOS before 27.

[sandbox-run]: ../plans/008-safe-removal/08-finish/02-sandbox-run.md
