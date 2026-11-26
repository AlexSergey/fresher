# Known issues

A known issue is a defect we live with for now. Each one records the symptom, the cause, what was tried and why it
failed, and the options left, so that nobody repeats the work. A known issue is not deferred work
([`../deferred/`](../deferred/README.md)): deferred work is agreed and waits for its turn, a known issue has no agreed
fix yet.

## Format

One file per issue, named `<part>-<name>.md` (`core-`, `scanner-`, `rules-`, `store-`, `cleaner-`, `app-`, `ui-`,
`repo-`). Each opens with a status line and the date it was last updated:

```markdown
# <Issue>

**Status:** open | parked | fixed (<link to the change>) · **Updated:** YYYY-MM-DD

## Symptom

## Cause

## What was tried

## Options left
```

When an issue is fixed, its file stays with the status `fixed` and a link to the change.

| Issue                                                                                     | Part    | Status | What                                                                                              |
| ----------------------------------------------------------------------------------------- | ------- | ------ | ------------------------------------------------------------------------------------------------- |
| [Quitting from the Dock during a cleanup does not ask](./app-quit-during-cleanup.md)      | app     | open   | The Dock's Quit cannot be held: the cleanup stops after the item in flight, no question; ⌘Q asks |
| [A setting changed during a scan does not ask for a rescan](./core-settings-changed-during-a-scan.md) | core    | open   | the running scan ends unmarked, so "Rescan to apply" is not shown for a change made during it     |
| [The scan offers an item inside the Trash](./app-scan-offers-an-item-in-the-trash.md) | app     | open   | the cleanup leaves it out as protected; whether the scan should skip the Trash is open            |
| [Spotlight does not see an IDE on a volume it does not index](./app-spotlight-misses-an-excluded-volume.md) | app | fixed | every mounted volume is checked indexed once per scan; one that is not keeps everything (plan 019/01/03) |
| [A sandbox scan says it was made days ago](./app-sandbox-scan-dated-at-generation.md) | app | open | sandbox mode's clock starts at the generation; the interface counts from the computer's clock |
| [The sandbox clock starts again at every launch](./app-sandbox-clock-restarts-each-launch.md) | app | open | sandbox mode only: after a relaunch the Trash line can count cleanups twice and the cache age guard is inert |
| [The Trash line's first number is stale after the user empties the Trash](./app-trash-waiting-stale-after-emptying.md) | app | open | the share drops at once, the first number keeps the last scan's measure until the next scan |
| [The sandbox autotests failed at times on an allocated size](./app-sandbox-storage-map-allocated-size.md) | app | fixed | a data-filled fixture file grown by appends kept blocks past its end under load; it is sized first now |
| [A compose test sets the real I/O policy of the test process](./app-compose-test-sets-the-io-policy.md) | app | open | the normal parts built with `MACOS_POLICY` change the test binary's policy; harmless under nextest only |
| [The disk chart keeps its numbers after a cleanup](./ui-chart-stale-after-a-cleanup.md) | ui | open | found in the 0.1.0 sandbox run: after a cleanup and a permanent delete the chart showed the old numbers |
| [Finder's own Put Back is unreliable for what Fresher moved](./cleaner-finder-put-back-unreliable.md) | cleaner | open | found in the 0.1.0 sandbox run: Finder restored a same-named folder to the wrong place and offered no Put Back for an emptied folder's file; Fresher's put back was right |
| [The confirmation of a long cleanup hides the choice of method](./ui-confirm-dialog-hides-the-method.md) | ui | open | found in the 0.1.0 sandbox run: two scroll bars, the permanent-delete choice below the fold |
| [The scan's progress goes to about 16 %, then starts over](./ui-scan-progress-starts-over.md) | ui | open | seen on the 0.1.0 release build: the bar rises, falls back to 0 and rises again; the scan is right |
| [The keyboard e2e test sees "not answering" on Largest under load](./ui-keyboard-e2e-largest-unreachable-under-load.md) | ui | open | WebKit, under load only: a mocked Largest command rejects; there before 0.1.0's fixes, fails more often without them |
| [Empty Trash refused a node_modules folder Fresher moved](./app-empty-trash-refuses-a-moved-folder.md) | app | open | Finder said "in use" with nothing holding it; Delete Immediately on the item worked |
| ["Reopen Fresher" right after a confirmed cleanup cuts it short](./app-relaunch-after-a-confirmed-cleanup.md) | app | open | the running-cleanup check and the shutdown are not one step; the shutdown stops after the item in flight |
| [A scan root on a volume that links into a guarded home folder](./app-volume-root-linking-into-a-home-place.md) | app | open | such a root gets only its volume's guard; the walk may enter the home folder behind the link and macOS may ask |
| [A look-alike Docker bundle can swap the signed CLI between its check and its run](./app-docker-cli-checked-then-run.md) | app | open | a copy of, or a link to, the genuine CLI passes the check, then is swapped before the run (audit SCAN-1) |
| [The cleaner acts on a path after checking a handle](./cleaner-act-by-path.md)            | cleaner | fixed  | a permanent delete, an emptying and the growth walk act beneath held folders now (plan 008/04)    |
| [The in-use check sees only the user's own processes](./cleaner-other-users-processes.md) | cleaner | open   | a file held only by another user's process, or opened after the last snapshot, is not seen        |
| [The system's Trash call takes a path](./cleaner-trash-call-by-path.md)                   | cleaner | open   | a parent swapped for a link at the call moves the item behind it; detected after the move         |
| [A forged journal record can send a real trashed item to a place of its choosing](./core-forged-journal-moves-a-trashed-item.md) | core | open | the journal is plain text; a record forged whole, naming a real trashed item, is put back where it says (audit REMOVAL-1) |
| [A FIFO is read as a regular file](./core-fifo-read-as-file.md)                           | core    | fixed  | a named pipe is `EntryKind::Other` now: never a marker, never claimed, refused by the cleaner (plan 017/01/02) |
| [Moderate audit advisories in the lint tooling](./repo-moderate-audit-advisories.md)      | repo    | open   | 9 moderate `minimatch` advisories through typescript-eslint, sonarjs, jsx-a11y-x (dev only)       |
| [Supply-chain hardening left for later](./repo-supply-chain-hardening-left.md) | repo | open | lifecycle scripts run on `npm ci`, no gitleaks in CI, an unused fs plugin compiled in, a moving runner image (audit SUPPLY-8) |
| [The `all` sandbox's checklist is wrong where worlds interact](./repo-sandbox-all-worlds-interact.md) | repo    | open   | one case file's apps change another's cases in `npm run sandbox -- all`; its `EXPECTED.md` does not say so |
| [An unreadable folder inside a claimed match is not counted as not read](./scanner-unreadable-folder-in-a-claimed-match.md) | scanner | open | a developer cache with a locked sub-folder is not offered, and the user is not told why; the coverage stays empty |
| [The rescan property test fails at times on a file's access time](./scanner-rescan-property-access-time.md) | scanner | fixed | the test's changes settle the access time of the files they write, as the fixtures do (plan 017) |
| [Stale temporary files in the data folder](./store-stale-temporary-files.md)              | store   | open   | a crash during an atomic write leaves a `.<target>.<random>.tmp` file that nothing removes        |
| [A project's last activity does not see a file edited in place](./core-project-activity-misses-edits-in-place.md) | core | open | only folder times count; an editor that saves in place moves none |
| [A project cleaned once looks active again](./core-cleaned-stale-project-looks-active.md) | core | open | removing an artifact moves the project folder's time, so the rest waits for the days again |
| [A link at a module's place reads as no folder](./scanner-link-at-a-place-reads-absent.md) | scanner | open | `~/Library/Caches` replaced by a link reads `Absent`; the data behind it is neither offered nor seen |
| [A mounted volume Spotlight does not index turns off removed-app detection](./app-unindexed-volume-turns-off-removed-app-detection.md) | app | open | a USB stick, an installer image or a share keeps everything for `jetbrains` and `app-leftovers` while mounted |
| [Spotlight's answer for a removed app misses a nested app and a privacy-list folder](./app-spotlight-children-and-privacy-list.md) | app | open | an app that is a child of the id, or kept in a privacy-list folder, is not found |
| [The reference walk and the scanner compare avoided paths differently](./scanner-reference-walk-avoided-paths.md) | scanner | open | an avoided path through a symlinked folder makes `NaiveWalk` and the scanner disagree |
| [The accessibility gallery test times out at times in WebKit](./ui-axe-gallery-part-3-webkit-timeout.md) | ui | open | under load "every story of the gallery, part 3" runs out of time in WebKit only; it passes alone |
| [The keyboard e2e test fails at times on a long dialog in Chromium](./ui-keyboard-e2e-dialog-long-body.md) | ui | fixed | the long body became focusable a frame after the test looked; the gallery tests wait two frames now |
| [A category whose items are all left out shows "0 bytes / 0 items"](./ui-category-all-left-out-shows-zero.md) | ui | open | Xcode build data while Xcode is open: the zero totals sit beside the left-out line; the UI change is deferred |
| [The "Reopen Fresher" button is forgotten when its card is rebuilt](./ui-reopen-button-state-is-local.md) | ui | open | the button's state is the component's; after a remount the user clicks Open System Settings again |
| [The UI keeps every finding of a scan in memory](./ui-findings-kept-in-memory.md)         | ui      | fixed  | the store held every streamed finding; it keeps totals now, lists are paged (plan 003/05)         |
| [The WebKit perf specs fail at times by one to three milliseconds](./ui-perf-webkit-jitter.md) | ui | open | the frame clock counts whole milliseconds and jitters; a rerun with the same code passes |
| [The protection lets a link shaped like an app bundle through](./core-protection-passes-a-bundle-shaped-link.md) | core | open | the protection is lexical (no Unicode folding or normalisation); the uninstall listing (036/01/03) and plan (036/02/01) refuse links and variants |
| [The cleaner checks protection on names as the plan writes them](./cleaner-protection-compares-names-as-written.md) | cleaner | open | a case, folding or normal-form variant of a `never` place's path is compared as written; put back checks its folder as the disk spells it |
| [An app opened after the confirmation is not asked about again](./cleaner-app-opened-after-the-confirmation.md) | cleaner | open | the uninstall is judged again at confirm; an app opened after that, or during a move of many parts, is not re-checked (the cleaner is frozen in plan 036) |
| [A small file's folder is opened through a link among its names](./core-small-file-folder-through-a-link.md) | core | open | `head` refuses a link only as the folder's last name; a pin read from elsewhere can only keep more |
| [A disk image named after an installed app is offered as its installer by its name alone](./rules-downloads-installer-proven-by-name.md) | rules | open | a user's own `.dmg` named like an app is offered at Low, to the Trash; nothing in the file is read |
