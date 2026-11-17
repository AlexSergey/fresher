# 016/03/01 — The sandbox case file

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
fixtures, app

## Files

- `fixtures/sandboxes/jetbrains.toml` (new) — `feature = "jetbrains"`, `module = "jetbrains"`; the thirteen negative
  cases, then the five positive ones, of the design, section 8, "Sandbox": each with its apps (`[[case.app]]`,
  `[[case.running]]`), its files under `home/Library/{Application Support,Caches,Logs}/JetBrains/` (positives' files
  `fill = "data"`, so the map's numbers are blocks), its `[[case.left-out]]` counts, and a `[case.map]` on the version
  in use (`your-folders`), the similar names (`your-folders`) and two positives (`explained`). Its paths are its own
  across every case file.
- `fixtures/golden/sandbox-jetbrains.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings with their apps, the
  left-out counts, the map, the cleanup's outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — `jetbrains` in `case_files!`.

The cases of 02/04 (the data of an IDE that is no longer installed; design, section 8): the negatives "a removed
IDE whose data changed recently keeps every version" (fresh folder kept, `used-recently` under
`jetbrains.uninstalled`), "a removed IDE installed by the Toolbox app is not removed" (DataGrip in
`home/Applications/JetBrains Toolbox/DataGrip.app` kept as `current`) and "a removed IDE keeps its data while an
unknown app of the vendor runs" (`[[case.running]]` `com.jetbrains.gateway`, `running`); the positive "the data of a
removed IDE goes to the Trash" (no DataGrip app, `DataGrip2024.3` in the three places with `[[case.dir]]`
`age = "40d"`, offered in `jetbrains.uninstalled`, medium, `in-trash`). Folder ages count back from `world.toml`'s
`now`, which the harness's clock and sandbox mode's clock read, so they hold however late the sandbox is opened.
Added by fix round 1 of 02/04 (2026-11-17), negatives: "DataGrip.app symlinked from elsewhere is kept" (a link in
`Applications/` to a declared bundle outside the apps folders; the reader reads its target), "an IDE Spotlight finds
elsewhere is kept" (in a sandbox the search answers the sandbox's own apps, so the case declares the app where only the
search finds it, or the core e2e covers it with `FakeInstalledApps::with_elsewhere`), "a restored scan does not offer a
removed IDE's data" (the harness's restore after the first scan: `used-recently`), and "an unknown `com.jetbrains`
app installed keeps a removed IDE's data" (`[[case.app]]` `com.jetbrains.newide`, `unknown-ide`).

What the code does today for a case is written down when the file is made (as plans 007 and 012 did), and the file
follows the code only where the code is right by the design; a difference from the design is fixed in the module, not
in the case file.

Built (2026-11-17): the ruling of the review of 03/01 moved "a restored scan does not offer a removed IDE's data"
to 03/02's core e2e (`crates/e2e/tests/jetbrains.rs`, b9e7ccf), with "an IDE Spotlight finds elsewhere": the harness
scans once and its search answers only the sandbox's apps. The linked IDE's case proves the list of apps stays whole;
reading through the link is covered by `fresher-system`'s unit tests. The cases of an unknown app of the vendor, open
or installed, keep every case beside them, so each has a case file of its own
(`jetbrains-unknown-app-open.toml`, `jetbrains-unknown-app-installed.toml`); the Toolbox case uses DataSpell, since the
removed IDE's positive is DataGrip.

## Produces

The module's sandbox: its autotest and the run by hand (04/02).

## Checks

- `npm run sandbox:test -- jetbrains` passes; it fails, once each by hand, when a positive's danger, a left-out's
  `items`, a negative's file (moved under a claimed name) and a `[case.map]` part are changed on purpose (written in the
  conclusions).
- `npm run sandbox -- jetbrains` writes `sandbox/jetbrains/` with its apps, `world.toml` and an `EXPECTED.md` that
  reads well; `npm run sandbox -- all` writes it beside the other case files without a clash.
- `npm run lint`.

## Conclusions

Built: `fixtures/sandboxes/jetbrains.toml` (15 negatives, 8 positives, `[[case.left-out]]` on every recognized
folder, `[case.map]` on four cases) and two files of their own, `jetbrains-unknown-app-open.toml` and
`jetbrains-unknown-app-installed.toml`, with their goldens. Changed from the brief because a sandbox has one world for
all its cases: the Toolbox case uses DataSpell (a removed DataGrip is a positive), and the unknown-app cases are in
files of their own. Not expressible in a sandbox: Spotlight elsewhere (the sandbox's search answers from its listed
apps) and a restored scan (the harness scans once); both are in the core e2e. The linked-app case proves the list
stays whole, not that the link is read where it points (the reader's unit tests do). Running `all` makes the worlds
interact ([known issue](../../../known-issues/repo-sandbox-all-worlds-interact.md)). The check that changing an
expectation on purpose fails each kind of case is still to do by hand (see 04/02).
