# 016/02/04 — The data of an IDE that is no longer installed

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** modules,
core, system, testing, sandbox, app, docs

The user's decision of 2026-11-17 ([`../00-design.md`](../00-design.md), "Decisions", 10): the leftovers of a
JetBrains IDE that is no longer installed (the user's own case: DataGrip removed, its settings stayed) are offered as a
separate category, `jetbrains.uninstalled`, once none of the IDE's folders changed for 30 days, a number the user can
change in Settings. Design sections 4, 5 and 7.

## Files

- **The installed apps** (`crates/core/src/system/{apps,facts}.rs`, `crates/system/src/apps.rs`,
  `crates/testing/src/system_fakes.rs`):
  - `InstalledApps::list` answers an `InstalledList`: the apps and `unread`, every apps folder, folder in one and app
    bundle it could not read (no readable `Info.plist`, another volume's app), never dropped silently; a link named
    like an app is still skipped, not unread (`/Applications/Safari.app` is a link on every Mac).
  - `MacInstalledApps` also looks in the older Toolbox app's own folder, `~/Library/Application
    Support/JetBrains/Toolbox/apps`, through the known folder `AppSupport`, up to four folders below
    (`apps/<IDE>/ch-0/<build>/<IDE>.app`), never through a link, never inside a bundle. The newer Toolbox's
    `~/Applications/JetBrains Toolbox/` is one folder below the user's apps folder, which the reader already reads.
  - `SystemFacts::installed_apps_complete()`: the apps only when nothing was unread, else an error; `installed_apps()`
    keeps answering what was read (the storage map). `FakeInstalledApps::partial`.
- **The clock** (`crates/system/src/clock.rs`, `crates/sandbox/src/{world,generate}.rs`, `app/tauri/src/sandbox.rs`,
  `app/tauri/tests/sandboxes/harness.rs`, `crates/testing/src/tree_description.rs`):
  - the generator always writes `world.toml`, with `now`, its own "now" in seconds, which every age counts back from;
    `World::now()`;
  - sandbox mode's clock is `StartedClock::at(now)`: it starts at that moment and runs at the system's pace (the
    system's clock for a world without `now`); the harness's `FakeClock` stands at it; a normal or release build keeps
    `SystemClock`;
  - a tree's `[[dir]]` takes `age` or `modified`, set after everything is written, the deepest folder first, so a case
    can make a version folder old.
- **The module** (`modules/jetbrains/src/{decide,uninstalled,settings,places,lib}.rs`, `locales/en.json`):
  - row 4 of `decide` answers `Verdict::Uninstalled` instead of keeping the folder; `on_match` asks
    `installed_apps_complete()`, so a partial list is row 1 (`no-system`);
  - `uninstalled.rs`: `Gathered` keeps the row-4 folders of one scan with the scan's `CancelToken` (`is_same`, new in
    core) and `finish` takes them by product; `age(times, now, days)` is `Old` when every folder's own modification
    time is at least the days before now (an unknown or future time is recent);
  - `Old`: one finding per folder in `jetbrains.uninstalled` (Medium), identity `<place>/<folder>`, title by place
    (`jetbrains.finding.uninstalled-{settings,caches,logs}`), reason `jetbrains.finding.uninstalled-reason` with
    `{product}`, `{version}`, `{days}`, `MoveToTrash`, `for_app` the product's release bundle id; `Recent`: every
    folder left out in `jetbrains.uninstalled` with `jetbrains.left-out.used-recently`;
  - the setting `jetbrains.uninstalled-after-days` (number, days, 7 to 3650, 30 by default) with its label and
    description; `jetbrains.left-out.not-installed` is gone (no row gives it any more);
  - the review's minors: the test imports at the top of the positive module, the unreachable arm commented, two
    installed apps of one id, `no-system` through `on_match`.
- **Docs**: `docs/categories/jetbrains-uninstalled.md` (new) and its row; row 4 of the three other JetBrains category
  files; `docs/architecture/{README,modules}.md`, `docs/testing.md`, `docs/deferred/sandbox-world.md`,
  `docs/dependencies.md` (`serde_json` for the module's tests); the design's sections 4, 5, 7, 8, decisions and
  agent's calls; 03/01's brief.

- **Fix round 1** (the controller's rulings of 2026-11-17, after review): an app link in the apps folders is read where
  it points (unreadable: unread), a folder link or another volume's folder is unread; `InstalledApps::find` and
  `SystemFacts::find_apps` (Spotlight by bundle id through the runner, `AppSearch::Spotlight`; the sandbox's
  `AppSearch::Listed`), asked before offering (a hit: `installed-elsewhere`; a failure: `no-system`);
  `ModuleContext::restored`/`is_restored`, set by the engine for a restore, in which row 4 never offers; an installed
  unknown app of the vendor keeps row 4 (`unknown-ide`); the setting's minimum 7; the module's tests in
  `src/tests.rs`; two deferred items.

## Produces

The fourth category, its setting and its left-out reason, for 02/03's fixtures (a home with a removed IDE, old and
fresh) and 03/01's case file; a sandbox whose ages hold however long ago it was generated.

## Checks

- Unit, negatives first: the reader (an unreadable apps folder beside a readable one, an unreadable folder below one,
  a bundle with a broken or missing `Info.plist`, the Toolbox folder through a link, deeper than four folders, not
  readable; then the user's Toolbox folder and the older Toolbox's folder listed); `SystemFacts` (a partial list is an
  error only when asked complete); `World` (`now` written and read, refused when not a number); the folder ages;
  `StartedClock`; sandbox mode's clock with and without `now`.
- `decide` and `on_match`, negatives first: installed through the Toolbox, a partial list, the IDE or its early
  access build running, an unknown vendor app running, a folder changed 29 days ago, a time unknown, a longer setting,
  a scan cancelled before its `finish`; then a removed IDE's three folders offered, a shorter setting, the two orders;
  a property test of `age` (old exactly when every folder is old enough, whatever the order).
- `cargo nextest run -p fresher-core -p fresher-system -p fresher-testing -p fresher-sandbox -p fresher-module-jetbrains
  -p fresher-app -p fresher-engine`; `npm run sandbox:test`; the module's contract test (four catalogue files).

## Conclusions

Built (the user's decision of 2026-11-17): the category `jetbrains.uninstalled` (Medium), the setting
`jetbrains.uninstalled-after-days` (7 to 3650, 30 by default), the reader looking in the Toolbox folders and
reporting what it could not read (`InstalledList::unread`, `installed_apps_complete`), links read through, Spotlight
(`mdfind` by bundle id) as a second source, the sandbox's clock (`StartedClock`, `now` in `world.toml`), `[[dir]]
age`. The review found three paths to offering a live IDE's data; all three were closed (links, Spotlight, restored
scans) and an installed unknown JetBrains app keeps everything. Limits: an IDE reachable only through a link counts
as not installed when the link does not read; the storage map names a removed IDE by its bundle id
(plan 019/01/04 added the name); a Mac with Spotlight's index off answers nothing, so only
the other guards hold; the age is the folder's own modification time
([deferred](../../../deferred/folder-age-from-children.md)).
