# Testing

What every part is tested with, and the bars it must pass. Test structure (negative cases first, then positive, in
separate blocks) is in the root [`CLAUDE.md`](../CLAUDE.md). Set up by plan
[001/04](./plans/001-repository-setup/04-testing-architecture/readme.md).

## Bars

- **Coverage ≥ 90 %**: lines in the Rust crates (`cargo llvm-cov`, `app/tauri/src/main.rs` excluded); lines, branches,
  functions and statements in the UI (Vitest, `src/main.tsx` and the test helpers excluded). The pre-push hook and CI
  fail under it.
- **Every rule is tested both ways**: paths it must NOT claim come first (look-alike folders, a current version, a
  symlink out of the root), then the paths it must claim.
- **Deterministic**: no wall clock (the `Clock` port, `FakeClock` in tests), no network, no real home folder. Every test
  runs on a temporary fixture tree; the Trash and external commands are ports with fakes.
- **Nothing is ever deleted outside a temporary folder** in a test: `FakeTrash` moves items into a folder inside the
  fixture tree; every cleaner in a test is `Reach::Sandbox`, acting only below a fixture holding the sandbox marker
  (`FixtureTree::with_sandbox`), or `Reach::SandboxFolder`, acting only strictly inside a generated sandbox folder with
  the marker at its top (sandbox mode, the sandbox autotest, `crates/e2e/tests/temp.rs`); the tests of the system's
  Trash move items only on a disk image they mount, and no test lists or empties the user's Trash.

## Kinds of tests

| Kind           | What it proves                                                                   | Rust core                                    | UI                                         |
| -------------- | -------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------------------ |
| Unit           | one function or module                                                           | `#[cfg(test)] mod tests`, cargo-nextest      | Vitest + Testing Library + happy-dom       |
| Property-based | a rule holds for ANY valid input (sizes add up, versions order, no double count) | proptest (`mod property`)                    | fast-check (where logic exists)            |
| Compile-fail   | a misuse of a safety type does not compile                                       | trybuild (`tests/types.rs`, `compile_fail/`) | —                                          |
| Race           | a swap made by another thread during an act never sends it elsewhere             | `fresher-testing`'s `Swapper`, 100 rounds    | —                                          |
| Integration    | one crate through its public API, on fixture trees                               | the crate's `tests/`                         | —                                          |
| Golden reports | a fixture home gives exactly the expected report                                 | `crates/e2e` against `fixtures/golden/`      | —                                          |
| Sandbox        | every case of a feature's case file is offered, left alone or cleaned as declared | `app/tauri/tests/sandboxes.rs`, `fixtures/golden/sandbox-*.json` | —                      |
| Core e2e       | command → engine → modules → events → settings file, end to end                  | `crates/e2e` on fixture homes                | —                                          |
| UI e2e         | the user flows of the interface, on a mocked core                                | —                                            | Playwright on the Vite build, mocked IPC   |
| Visual         | the gallery's stories and the screens did not change unnoticed                   | —                                            | Playwright screenshots, ≤ 50 pixels differ |
| Accessibility  | keyboard, focus, contrast, page zoom and WCAG 2.1 AA on every story and screen   | —                                            | Playwright and axe; VoiceOver by hand      |
| Performance    | the scan stays within its time and memory budget; the UI within its frame budget | criterion benches on a large generated tree  | `test:perf`, run by hand                   |

The full app (WebView + Rust) cannot be driven by WebDriver on macOS (`tauri-driver` supports Linux and Windows only),
so the UI e2e run against the web build with the Tauri IPC mocked, and the core e2e cover the Rust side end to end.
The core e2e prove the harness (`smoke.rs`: a fixture home is built and read back against
`fixtures/golden/smoke.json`), the whole Rust path on the `demo` module (`demo.rs`: the engine, the `demo` module and a
real settings file, the events and the overview against `fixtures/golden/demo.json`; the settings survive a restart; a
settings file from a newer version is never overwritten), and since plan 004 scan → classify → cache: `scan.rs` (the
engine with the real scanner on fixture homes, `fixtures/golden/scan.json`), `rescan.rs` (a scan, a scripted change, a
rescan that reads only the changed folders and the check's sample, a restore in a new engine that reads no folder,
`fixtures/golden/rescan.json`), `footprint.rs` (below), and since plan 008 select → plan → confirm → clean:
`cleanup.rs` (the engine with the reference walk, the `trial` module and the real `DiskCleaner` in the sandbox on
`fixtures/homes/cleanup.toml`, the real journal; the whole path with history and put back, a cancel, an item replaced,
a permanent cleanup, and the refusals; `fixtures/golden/cleanup-*.json`). `footprint.rs` also writes a hundred
cleanups of a hundred items through the real journal.

## Where things live

- Rust unit tests sit next to the code (`#[cfg(test)] mod tests { mod negative {…} mod positive {…} }`); integration
  tests in each crate's `tests/`. Helpers of an integration test go in a `#[cfg(test)]` module, so Clippy treats them
  as test code.
- `crates/testing` — shared test helpers: `FixtureTree` (a fake home folder from TOML), `TreeReport` (a deterministic
  listing for golden files), `FakeClock`, `FakeTrash`, `FakeCommandRunner`. The ports they fake (`Clock`, `Trash`,
  `CommandRunner`) live in `fresher-core`. For the cleanup ([`architecture/removal.md`](./architecture/removal.md)):
  - `FixtureTree::with_sandbox` and `mark_sandbox(folder)` write the sandbox marker with its signature (opt-in, so no
    scan test or golden file sees it).
  - `FakeTrash` moves into a Trash folder of the fixture and puts back without replacing anything; `without_trash`
    (a volume with no Trash), `reporting_nothing` (no landed path) and `refusing` play the system's answers.
  - `FakeOpenFiles` (a fixed set of held paths, or a failure), `FakeVolumes::answering_available` (the space before
    and after), `MemoryCleanupJournal` (the journal in memory, or failing), `FakeCleaner` (records the `ExecutedPlan`
    the engine handed it).
  - `FakeAccess` (the state of each capability, the guarded places of a fixture home folder, scripted answers of
    `try_place`, which remember a `Granted` or `Denied` answer as the real probe does, and the places tried) and `FakeSystemUi` (records every pane opened and path revealed, or fails): the
    ports of permissions ([`architecture/permissions.md`](./architecture/permissions.md)); both in
    `crates/testing/src/fake_access.rs`.
  - `Swapper` swaps a folder for a link to another folder and back on its own thread until dropped, then puts the
    folder back: the race tests of the cleaner.
  For modules ([`architecture/modules.md`](./architecture/modules.md)):
  - `NaiveWalk` — the reference `DiskWalk` over a fixture root, with `std::fs`: links listed, never followed, nothing
    on another device; each match carries its entry's facts for removability (owner, write access from the mode bits,
    flags), read for the owner of the fixture root; the scanner is tested against it (the same fixture gives the same
    matches, with the same entry facts).
  - The fakes of the scan's ports: `FakeVolumes` (fixed facts per root), `FakeChangeJournal` (a script of positions,
    changed folders and gaps, so no test waits on the system's journal), `MemoryScanCache`, `FixedDiskState` (a tree
    from paths and sizes, for the engine's queries).
  - `LargeTree` — the generated trees of the benches and the large tests ("Performance").
  - `settle_access_time(path)` — a file's access time set to its modification time: the system moves an access time
    on a read only while it is older, so a walk reading a `CACHEDIR.TAG` no longer moves it under the next walk's feet.
    `FixtureTree` settles every file it writes, `LargeTree` every tag.
  - `run_module(&module, &tree, &Ports::at(seconds))` — one module run as the engine would, on the fixture tree as the
    home folder: walk, matches, `finish` → `ModuleRun { findings, outcome }`; `ModuleRun::golden()` is its stable JSON
    report, paths relative to the fixture root. `Ports { restored: true, .. }` runs it as a scan restored from the
    cache; `Ports { order: MatchOrder::Reversed, .. }` hands it the matches last first, so a test can check that its
    answers do not depend on the walk's order; `Ports { avoid: vec!["Library/Logs".into()], .. }` (paths relative to
    the fixture root) leaves those folders unopened, so a place among them reaches `finish` as `PlaceSeen::NotScanned`.
  - `assert_module_contract(&module, &Catalogue)` — the descriptor is valid, every category has its five texts and every
    setting (`<module>.enabled` included) its label and description, every key is in the module's namespace;
    `Catalogue::Required(docs/categories)` also wants each category's catalogue file with the same danger level
    (`Catalogue::DevOnly` for the dev-only `demo` and `trial`). `assert_texts_used(&module, &runs)` — every text is a
    descriptor key or used by a finding of the runs, and every finding's text exists with the args its placeholders
    name. Their `check_…` forms return the list of `ContractProblem`s instead of failing.
    `check_removals_matched(&runs)` lists every finding whose removal names a path no match of its run reported: the
    engine takes an item's removability from that match, so such an item needs an administrator (fail closed) and can
    never be removed by the app.
- `crates/scanner/tests/reference.rs` — the scanner against `NaiveWalk`: the same matches and the same summary on
  every fixture home and on random trees (proptest, its failing seeds kept in `reference.proptest-regressions`), with
  the portable reader; `macos_reader.rs` runs the same with the macOS reader. `CountingReader` counts the folder reads,
  so a test holds "one bulk read per folder" and "a rescan with nothing changed reads only the sample and the
  unreadable folders".
- `crates/macos` — tested on temporary folders and on disk images the tests mount with `-nobrowse` (a test skips itself
  when `hdiutil` is missing); nothing reads the home folder.
- `crates/rules` — the shared matcher of rule data (`RuleSet::parse`, `interests`, `evaluate`, `stale`), tested on
  `FixtureTree`s and `FakeClock`, with property tests of the glob and of the version order.
- `crates/cleaner` — every test runs on a temporary tree with the sandbox marker and the fakes above (disk images it
  mounts inside the tree, skipped without `hdiutil`). Unit tests sit next to each check and removal, negative first
  (`paths.rs`, `guard/`, `remove/`, `put_back.rs`, `pipeline.rs`); `remove/permanent.rs` races a delete against a
  `Swapper` 100 times (a parent and a folder inside swapped for a link) and mounts a volume inside a folder.
  `tests/never.rs` is the property test of what the cleaner never does, for any plan (random trees with links in, out
  and up and a protected, carved-out, hard-blocked and ignored folder; random plans, items of several paths among
  them: one goes whole or not at all, unless the Trash refuses a part at its act; the swap case); its default run
  is in the hooks, `PROPTEST_CASES=10000` by hand. `tests/types.rs` holds the compile-fail cases of `Verified` and
  `Act` (`tests/compile_fail/`); `tests/reach.rs` fails when the real-disk token is named outside its three places;
  `tests/deletion_point.rs` fails on a second deletion point or a deleting system call outside `ffi/beneath.rs`.
- `crates/core/tests/types.rs` — the compile-fail cases of the plan's types (`compile_fail/`: a `ConfirmedPlan`,
  `Confirmation`, `Act` or `PermanentConsent` made by hand, a confirmed plan cloned or used twice, an unconfirmed one
  executed); the echo's property test is in `cleanup/confirm.rs`.
- `crates/store/src/journal/tests.rs` — the cleanup journal on a temporary data folder: a torn last record cut at every
  byte never corrupts the records before it, `begin` is synced before it returns, the file is `0600`, a newer or
  unreadable file is read-only, an older one is migrated line by line, rotation by rename.
- `crates/e2e` — the core's end-to-end and golden tests (a crate with tests only). `tests/ignore.rs` scans a fixture
  home with the real settings file: ignored categories and items leave every answer, Restore brings them back
  without a rescan (`fixtures/golden/ignore.json`); the flows of ignoring a module and a group (an unknown module
  refused, the findings, totals, storage map and `hiddenModules` follow, restore without a rescan, a new engine on
  the same data folder keeps the lists, a cleanup plan over an ignored module drops its items). `tests/places.rs`
  walks the temporary places beside a fixture home (named in the map, cleaned to the Trash, refused once no module
  names them); `tests/temp.rs` runs the `temp` module's flows on the real scanner, settings file and cleaner, two of
  them on real old modification times, a real socket and a real pipe, nothing stated; `tests/dev_caches.rs` runs the
  `dev-caches` module's flows the same way, with a scripted change journal so its rescans read only what it names;
  `tests/downloads.rs` runs the `downloads` module's flows on the real scanner, settings file, cache, journal and
  cleaner over `fixtures/homes/downloads-flow.toml`, the access included: nothing is tried or read in Downloads before
  the opt-in (`Unavailable`, `FakeAccess::tried()` empty, the coverage names the folder); the first scan after the
  opt-in (the switch on, the try answering `Granted`) learns the access (`AccessChanged`) and offers in that same scan,
  its report the golden `fixtures/golden/downloads-flow.json`; a denied folder keeps the module unavailable; scan,
  confirm with High, Trash and put back byte for byte; a browser that quits, days that pass and an app that is
  updated turn a left-out into an offer on a rescan; the storage map counts an installer in its app's line and opens
  the user's folder; ignoring the big files; and Full Disk Access, which makes the module available without a try.
  `FakeAccess` remembers a try that answered `Granted` or `Denied`, as the real probe does, so the cleanup's own access
  check agrees with the scan. `tests/stale_projects.rs` runs the `stale-projects` module's flows beside the other modules the app registers over
  `fixtures/homes/stale-projects-flow.toml`, every folder dated (an undated one counts as changed now), the module asking
  the real scanner's tree after the walk: an active project and one with a recent commit keep their packages
  (`recent`), the first scan's report is the golden `fixtures/golden/stale-projects-flow.json`, scan, confirm with
  Medium, Trash and put back byte for byte, a change in a stale project kept by a rescan, the setting moving the days, a
  restore at launch that offers nothing (`scan-again`), the storage map and the browse view opening `target`,
  `incremental` and `build` folders, ignoring the build output, and the known risk that a cleaned stale project looks
  active on the next scan. `tests/app_leftovers.rs` runs the `app-leftovers` module's flows beside the other modules the app registers (the other
  modules' caches under `Library/Caches` still theirs), with scripted installed and open apps, Spotlight's answers, the
  clock and the change journal, every rescan asserted to be one (`fixtures/golden/app-leftovers-flow.json`).
  `tests/app_caches.rs` runs the `app-caches` module's flows beside the other modules of `Library/Caches` (`jetbrains`,
  `dev-caches`, `app-leftovers`) over `fixtures/homes/app-caches-flow.toml` with a scripted open app: no folder is
  offered by both `app-caches` and `app-leftovers`, an open app's folders wait (`app-open`) until a rescan after it
  quits, a restored scan judges the open apps again, a file held open skips its folder in use with nothing of it moved,
  a cleanup empties each folder and keeps it with its entries in the Trash, put back returns them beside a file the app
  wrote since, and the storage map counts both folders in the app's line (`fixtures/golden/app-caches-flow.json`).
  `tests/xcode.rs` runs the `xcode` module's flows beside `dev-caches`, the other module of `Library/Developer`, over
  `fixtures/homes/xcode-flow.toml` with Xcode installed and the open apps scripted per scan: the derived data project
  is offered by `dev-caches` only, every candidate waits (`app-open`) while Xcode or the Simulator is open until a
  rescan after it quits, a file held open inside the simulators' caches skips that item in use while the rest go, a
  cleanup moves the device support folders and the archive to the Trash and empties the previews and the simulators'
  caches, keeping the folders, put back returns the archive byte for byte and the emptied caches beside a file written
  since, a restored scan offers the same with Xcode closed and leaves everything out with it open, and the storage map
  counts every finding of both modules in Xcode's line (`fixtures/golden/xcode-flow.json`).
  `tests/logs.rs` runs the `logs` module's flows beside the other modules of `Library/Logs` (`jetbrains`,
  `app-leftovers`) over `fixtures/homes/logs-flow.toml`: no path is offered by two modules (the removed app's and the
  older IDE's log folders are theirs, a state file no one's), a fresh log stays through a cleanup of everything, a log
  held open is skipped in use, a restored scan leaves every log out `scan-again` until a scan, fourteen days in the
  settings file keep a ten-day-old log, a cleanup moves the old files to the Trash and keeps their folders, and put
  back returns them byte for byte (`fixtures/golden/logs-flow.json`, the first scan's report).
  `tests/app_caches_browsers.rs` runs the same module's browser and web-engine rows beside the same modules over
  `fixtures/homes/app-caches-browsers-flow.toml` (Google Chrome's caches of two profiles under `Library/Caches` beside
  a profile folder the table does not name, Chrome's profile data in its support folder, Redis Insight's caches beside
  its cookies and local storage) with a scripted open browser: no folder is offered by two modules, a profile's and an
  app's data are never offered and stay byte for byte through a cleanup of every finding, an open browser's caches
  wait (`app-open`) while Redis Insight's are offered, until a rescan after it quits, a file held open skips its cache
  in use with nothing of it moved, a cleanup empties each cache and keeps it with its entries in the Trash and put
  back returns them, and the storage map counts the caches in their app's line
  (`fixtures/golden/app-caches-browsers-flow.json`).
  `tests/trash.rs` runs the Trash view's flows (plan 037) over `fixtures/homes/trash.toml` on the real scanner,
  cleaner and journal with a fake Trash and a fake Finder: what the user put in the Trash counts in `waiting` only, a
  cleanup's items count in both numbers, an item put back, emptied from the Trash, moved permanently or skipped
  counts nothing, a rescan takes the cleanup into its measure and adds nothing twice, a Trash on a second folder
  gets its own line, `open_trash` opens the folder it names and refuses an index it did not list, and Finder's
  30-day setting dates the share.
  `tests/local_ai_models.rs` runs the `local-ai-models` module's flows alone over `fixtures/homes/local-ai-models-flow.toml`
  (a model whose snapshot names are links into its blobs, a dataset, an LM Studio model, a Space and a project holding
  a link into the model's blobs): a Space and the project's link are never offered, a model with a blob held open is
  skipped in use, the project's link stays a link after the cleanup, the sizes of each category, scan, confirm with
  Medium, Trash and put back byte for byte (`fixtures/golden/local-ai-models-flow.json`), and a restore at launch that
  offers the same models.
  `tests/local_ai_models_ollama.rs` runs the same module over `fixtures/homes/local-ai-models-ollama-flow.toml` (an
  Ollama store: two tags of one model, a model of its own, and a base model with one made from it sharing two layers):
  a model goes together, its manifests and unshared layers as one item, or nothing of it moves (a layer held open or
  mapped, a layer replaced after the plan); a shared layer and the sharer's manifest stay; the journal holds every
  part, put back brings the subject last, byte for byte (`fixtures/golden/local-ai-models-ollama-flow.json`); a restore
  reads the manifests again.
  `tests/unused_toolchains.rs` runs the `unused-toolchains` module's flows alone over
  `fixtures/homes/unused-toolchains-flow.toml` (rustup's default, a moving channel, an old toolchain, a dated nightly and
  one a project's `rust-toolchain.toml` names; nvm's default, the newest Node version and an old one; no stored
  settings, so the flow turns the Node switch on through `update_setting`): the default, the channel, the pinned
  toolchain and nvm's default and newest are never offered, a `.nvmrc` written between two scans keeps its version on
  the rescan, a version with its `bin/node` held open is skipped in use, the sizes of each category, scan, confirm with
  Medium, Trash and put back byte for byte, `settings.toml` and the pins untouched
  (`fixtures/golden/unused-toolchains-flow.json`), and a restore at launch that reads the pins again (a pin changed
  since moves the offer from one toolchain to another).
  The homes' shared helpers live one per concern in `tests/support/` (`events.rs`, `report.rs`, `tree.rs`), each
  pulled in with `#[path]` only by the tests that use it. The sandbox flow homes (`temp_home`, `dev_caches_home`,
  `app_leftovers_home`, `app_caches_home`, `app_caches_browsers_home`, `local_ai_models_home`,
  `local_ai_models_ollama_home`, `unused_toolchains_home`) share one harness there, one file per concern: `home.rs`
  (the sandbox's layout and the
  `Flow`), `home_engine.rs` (the engine assembled from the parts a home gives: its modules, its fakes, its avoid list),
  `home_scan.rs` (the scans and the findings), `home_rescan.rs` (the scripted change journal and a scan that must be a
  rescan) and `home_cleanup.rs` (the cleanup, the outcomes, the journal's records and the relative report); each home
  keeps only its module's own.
- `crates/engine/src/ignore.rs` — the ignore lists, Reset all settings and "Rescan to apply" on the engine's fakes;
  `crates/engine/src/ignore_module_tests.rs` — ignoring a module and a group, and restoring them without a rescan;
  `crates/core/src/settings/core_schema.rs` holds the guard that no core setting names the confirmation, a danger
  level or the removal.
- `app/tauri/tests/bindings.rs` — the IPC types: every type the commands and events carry is rendered by `ts-rs` and
  compared byte for byte with `app/ui/src/ipc/bindings.generated.ts`; no listed type refers to an unlisted one, no
  number becomes a `bigint`, no field is in snake_case. `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test
  bindings` rewrites the file; the diff is reviewed like a golden file.
- `fixtures/homes/<name>.toml` — fixture home folders; `fixtures/golden/<name>.json` — the expected reports.
- UI unit tests: `*.test.ts(x)` next to the component; the core is mocked with `mockCore` from `src/ipc/mock.ts`, by
  the command keys of `src/ipc/index.ts` (the only place the Rust command names are spelled); `refuse(refusal)` answers
  with a `Refusal` (the error and the text the core built for it), `pushEvents(...)` streams events through the
  subscribed channel. `src/test/demo-scan.ts` is a demo scan as the core streams it, with its texts, shared by the unit
  and the e2e tests; `src/test/large-scan.ts` generates a scan of N findings in M categories (the same on every run),
  `src/test/page-of.ts` answers `get_findings` over a list of findings as the engine does, and
  `src/test/fake-clock.ts` is the clock the store's timers are tested on. `src/test/cleanup-plan.ts` builds the
  core's plan, events and report over the demo scan's findings, shared by the unit tests of the cleanup state
  (`state/cleanup.test.ts`), its screens and the e2e mock. `src/test/local-ai-models-scan.ts` is a scan of the
  `local-ai-models` module with its own texts read from its locale file: models offered, one kept (not the user's),
  and a second script with LM Studio open, where every LM Studio model is left out; the e2e mock plays both
  (`SCRIPTS.localAiModels`, `SCRIPTS.localAiModelsLmStudioOpen`) on Scan's cards and a category's Details.
  `src/test/app-caches-scan.ts` is the scan of the `app-caches` module in the group System (three Low findings of
  two apps, each naming its app, and one folder left out `app-open`, texts read from the module's `en.json`); the e2e
  mock plays it (`SCRIPTS.appCaches`) on the card, the category's Details (visual and axe), the review naming Low, a
  cleanup that empties and the put back; the "Emptied. The folder itself stays." line is the cleanup detail row of an
  item whose entries were not trashed, since the result summary counts what went to the Trash.
  `src/test/app-caches-browsers-scan.ts` is the same module's browser and web-engine caches (Chrome folders titled by
  browser, profile and folder, Redis Insight's by app and folder, Firefox's cache left out as open); the e2e mock plays
  it (`SCRIPTS.appCachesBrowsers`) on the cards, the Details, the review and a cleanup that empties.
  `src/test/xcode-scan.ts` is the scan of the `xcode` module in the group Development (two device support folders, the
  previews, the simulators' caches and an archive, each for the app Xcode, and one device support folder left out as
  special; texts read from the module's `en.json`); the e2e mock plays it (`SCRIPTS.xcode`) on the cards, the Details
  (visual and axe), the review naming High, a cleanup that empties the caches and moves the archive, and the put back.
- `src/test/trash-view.ts` holds the samples of the Trash view (a line with a share, a dated one, one not measured, two
  Trashes); the e2e mock plays them (`SCRIPTS.trash`, `trashDated`, `trashCleaning`, `trashNotMeasured`, `trashTwo`,
  `get_trash` and `open_trash`, which refuses an unlisted index) on the scan screen (visual and axe), a cleanup that
  grows the line, Open Trash, and a result with no button named "empty".
- UI e2e and visual tests: `app/ui/e2e/`, the core mocked with `e2e/core-mock.ts` (answers, refusals, the events a
  scan streams through the channel, a scan held mid-stream until `releaseScan` or Stop, a scan paced by `interval`
  (one event every so many ms, each asked of Node, as the engine streams), Scan's totals during and after the scan,
  `get_findings` paged from the findings streamed so far, a second scan refused while one runs, and the
  `setting-changed` event of an update); its cleanup part (`e2e/support/cleanup-mock.ts`) answers a plan per
  selection, compares the echo, streams a cleanup item by item (held after `hold` items until `releaseCleanup` or
  Stop), keeps a history and puts back, and `requestQuit` plays the close guard's `quit-requested`. `SCRIPTS` there are
  the scripts the flows and screenshots play (`cleaning`, `cleaningHeld`, `cleaningLeftOut`, `cleanupsEmpty` for the
  cleanup); `flows.spec.ts` walks select → review → confirm to the Trash → progress → result → Cleanups → put back,
  the permanent path, Stop mid-cleanup, the items left out, a refused plan and the quit question;
  its ignore part (`e2e/support/ignore-mock.ts`, `SCRIPTS.ignoring`) answers `ignore` and `restore_ignored` for
  modules, groups and items (`ignore` with a finding, `restore_ignored` with a path) and keeps the lists for the
  session; `get_findings` leaves out the findings at or inside an ignored path, but the category totals are not
  recomputed; `e2e/core-mock.spec.ts` proves its paging. Screenshots in `app/ui/e2e/screenshots/`, per browser. WebKit is the
  reference engine (the app runs in WebKit on macOS); Chromium runs too. The suite runs on `npm run build:e2e` (the
  app and the gallery in `dist-e2e/`), served with the app's production CSP; every spec checks that the policy
  refused nothing. The screenshots are compared only on the developer's Mac, by the pre-push checks: the CI runner has
  another macOS version, which draws text a few pixels differently, so CI runs the suite with `--ignore-snapshots`
  (every other check still runs) and turns on full keyboard access (`AppleKeyboardUIMode` 3), without which WebKit's
  Tab skips buttons. When the runner has the developer's macOS version, CI can compare the screenshots again.
- **The Scan screen's flows** (`e2e/scan.spec.ts`, `e2e/flows.spec.ts`): the ring and the legend highlight each other
  and the legend's "i" opens the core's description; a card's settings menu opens, closes on Escape and works by
  keyboard (Ignore category, Ignore module); a finding row's menu on a category's details closes on Escape,
  ignores one item by keyboard (the row leaves and the focus goes to its neighbour), Restore in Settings brings it
  back, and Review's rows have no menu; a zone's Select all ticks only that zone; a card's Details opens the
  category's details with every finding; the selection bar shows on Scan and on the details; the short window's
  sidebar scrolls to Settings (`e2e/layout.spec.ts`).
- **The gallery** (`app/ui/gallery.html`, `src/gallery/`): every component in every state, each `*.stories.tsx`
  found by `import.meta.glob`; built only by `build:e2e`, never into the app's `dist/`. `gallery.spec.ts` photographs
  every story in WebKit; `visual.spec.ts` photographs every screen in each of its states in WebKit and Chromium.
- **Motion is off in the visual tests**: `e2e/support/motion.ts` sets `data-motion="off"` before the app runs (every
  duration 0, every loop at its first frame), so a screenshot is the same on every run. `motion.spec.ts` checks the
  system's "Reduce motion" and full motion in the engines.
- **Guards of the design system**, unit tests: the CSS guard (`src/test/css-guard.test.ts`: every stylesheet stays
  inside the Safari 16.0 baseline and animates only the compositor-friendly properties and the named paint-only
  exceptions), the contrast test
  (`src/design/tokens/contrast.test.ts`: every text and level pair of the tokens meets WCAG AA, computed from the
  token files), the motion tokens' mirror (`src/design/tokens/motion.test.ts`: `motion.css` and `motion.ts` agree);
  the import boundaries are ESLint's ([`architecture/`](./architecture/README.md), "UI").
- **Accessibility**: `axe.spec.ts` (every screen state and every story, no WCAG 2.0/2.1 A or AA violation; its one
  exemption is written in the spec), `keyboard.spec.ts` (every focus stop reached by Tab with a visible ring, a
  control answers its key, a dialog keeps and returns the focus, a screen change never drops it), `contrast.spec.ts`
  (under `prefers-contrast: more` no glow and no noise, stronger borders), `zoom.spec.ts` (125 % and 150 %: nothing
  cut or overlapping). VoiceOver and the real window's materials are checked by hand.
- **Performance**: `perf.spec.ts`, run only by `npm run test:perf --workspace app/ui`, by hand, never in the hooks or
  CI (frame timing on a shared machine is noise). Two projects run it with one worker, headed, full motion: `perf`
  (WebKit, where the budget is held: p95 ≤ 17 ms or the idle page's p95, no frame over 50 ms, at most about 60 list
  rows) and `perf-chromium` (reports long tasks and the JS heap). It streams a 50,000-finding scan, changes screens,
  scrolls 100,000 rows and checks the bundle's gzipped size; run it on a quiet Mac when a change touches the lists,
  the scan's screens or motion, and write the numbers in the plan's conclusions. A plan that touches none of them
  (a module, a core rule, docs) does not run it and its steps do not list it (the user's decision of 2026-11-21).
  `test:e2e` never runs it.

## Performance

The scan's timings and peak memory are measured by criterion benches run by hand, never in a hook (the UI's frame budget
is `perf.spec.ts`, above): `npm run bench` (`cargo bench -p fresher-scanner --bench scan`, a release build). The benches
live in `crates/scanner/benches/scan.rs`, grouped as `matcher/*`, `scan/*` (the full scan by thread count, the rescan
with nothing changed and after 1 000 changed folders, the restore: the cache decoded and the matches emitted), `cache/*`
(encode, decode, fit to the cap) and `state/*` (largest files, search by name); after each group the bench prints the
process's peak memory (`fresher_macos::peak_memory`, bytes). They are run only by a plan that changes the walk itself
(the scanner, the engine's scan path, the cache), not by a plan that adds a module or a category (the user's decision
of 2026-11-21).

They run on a generated tree (`LargeTree` in `crates/testing`): a seeded shape of folders, sparse files (declared sizes
cost no data blocks), hard links, `node_modules`-like dependency trees and folders tagged with `CACHEDIR.TAG`. Two
specs: `TreeSpec::SMALL` (100 000 entries, also used by tests) and `TreeSpec::LARGE` (3 000 000 entries), picked with
`FRESHER_BENCH_TREE=large` (the small one by default). A tree is built once under `target/bench-trees/<name>-<hash>/`
with a `.complete` marker written last, reused by later runs, and removed by `cargo clean`; the generator never removes
anything. Peak memory is a high-water mark: measure it on a run that does not build the tree.

The timing bars (plan 004's design, section 13) are held by hand: each step that can move them writes the numbers in
its plan; the last measured table is in plan 004's conclusions. Run them on a quiet machine on power, after one warm-up
run. `LARGE`'s 3 million entries do not fit the kernel's metadata cache, so its full scan reads cold; the warm bar is
measured on `SMALL`. What can be checked deterministically is held by tests in `npm run pre-push` instead: bulk reads
per folder (`CountingReader`), the folders a rescan reads, the tree's heap bytes per node (≤ 96 B,
`heap_bytes_per_node_stays_under_96`), the cache's bytes per node (≤ 32 B) and its cap.

The storage map's `Shares` (budget 50 ms and 5 MB, plan 012) are built over a state the real scanner commits on
`LARGE`, with the dependency trees and cache folders claimed, by a test ignored in the hooks and run by hand in
release; it prints the time and the heap (the hash tables' bucket count, not their capacity):
`cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture shares_of_the_large_tree`.

**The footprint test** (`crates/e2e/tests/footprint.rs`): ten scans of a generated tree through the real scanner and
`FileScanCache` on a temporary data folder, with a scripted journal and changes between the scans; the data folder
stays within `FOOTPRINT_BUDGET` and holds only the settings file and the cache's slot files. On `SMALL` it runs in the
hooks; on `LARGE` it is ignored and run by hand:
`cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture ten_scans_of_the_large`.

## Fixture homes

A fixture home is a TOML file; every path is relative and must stay inside the root.

```toml
# A file with a logical size (sparse: it costs no disk) and a modification time in seconds since the Unix epoch.
[[file]]
path = "Library/Caches/com.example.editor/Cache.db"
size = 52428800
modified = 1735689600

# An empty folder.
[[dir]]
path = "Library/Logs"

# A folder with its own time: `age` (or `modified`) is set once everything inside it is written, the deepest folder
# first, so the folder keeps it whatever the tree writes into it. A project that must look stale dates EVERY folder
# (the parents of its files and `.git/refs/heads` included): an undated folder counts as changed now and turns it active.
[[dir]]
path = "Library/Caches/JetBrains/DataGrip2024.3"
age = "40d"

# A symlink: listed, never followed. The target may point outside the root. `age` sets the link's own modification
# and access time (its target is never touched); `changed` states its status change, as for a file.
[[symlink]]
path = "Library/Caches/system"
target = "/Library/Caches"

[[symlink]]
path = "Library/Caches/old-link"
target = "../Documents"
age = "30d"
changed = "30d"

# A hard link: a second name of a file of the tree; both paths are relative to the root.
[[hardlink]]
path = "Library/Caches/copy.db"
target = "Library/Caches/com.example.editor/Cache.db"

# A file may carry an age instead of `modified`: "<n>d", "<n>h" or "<n>s" counted back from the "now" of the build
# (FIXTURE_NOW for `FixtureTree`), so a stale file stays stale; "-2d" is a time after it. `age` and `modified` never
# go together.
# `fill = "data"` writes real bytes of a fixed pattern instead of a sparse file, so freed space can be measured
# (at most 64 MiB a file and 512 MiB a description; a larger one is refused). `fill = "cachedir-tag"` writes a valid
# cache directory tag (its standard signature line, then a comment), never with a `size` or a `kind`.
# `text = "..."` writes the file's bytes as UTF-8 (at most 4096, the small-file reader's limit; its size is theirs),
# never with a `size`, a `fill` or a `kind`: a pin, a manager's settings file or a pointer a module reads in `finish`.
[[file]]
path = "Library/Caches/com.example.editor/old.bin"
size = 1048576
age = "30d"
fill = "data"

# Stated facts: what no tool or no user can make on disk (a status-change time, an owner, a system flag, a socket
# whose path is too long). The entry is written as a plain file or folder; the facts go to a list (`StatedFacts`)
# that `FactsLens` and the reference walk report instead of the disk's, for exactly these paths.
# `changed` and `accessed` take an age; `owner` is "system" (user id 0) or "other-user" (`nobody`, 4294967294, never
# the user running the tests); `flags` take "user-lock", "system-lock", "no-unlink" (read as the system's lock) and
# "restricted"; a `[[file]]` (never a `[[dir]]`) takes `kind` = "socket" or "pipe", never with `fill`.
[[file]]
path = "Library/Caches/com.example.editor/listener"
kind = "socket"
age = "30d"
changed = "30d"
owner = "system"
flags = ["no-unlink"]

# A sandbox marker written into that folder (the folder is created if missing).
[[marker]]
path = "work"

# An installed app, written as a real bundle: `Contents/Info.plist` (an XML property list with CFBundleIdentifier,
# CFBundleName, CFBundlePackageType APPL and, when given, CFBundleShortVersionString and CFBundleVersion; text
# escaped) and an empty `Contents/MacOS/`. Unsigned: its team id reads as none. The path ends in `.app`; the bundle id
# must be one the app reads (`BundleId::new`). `version` and `build` are optional. No text holds a control character,
# and no other entry of the tree lies inside the bundle: the bundle writes its own contents. `changed` states the
# bundle's status change as an age, as for a file; the bundle is written now, so a case says "installed 10 days ago"
# this way (`StatedInstalledApps` puts it on the installed app).
[[app]]
path = "Applications/WebStorm.app"
bundle-id = "com.jetbrains.WebStorm"
name = "WebStorm"
version = "2025.3.1"
build = "WS-253.1"
changed = "10d"
```

`FixtureTree::from_toml(path)` builds it in a temporary folder that is removed when the tree is dropped;
`tree.root()` is the only folder a test may scan or clean. `tree.stated()` holds its stated facts, at the root as
given and at its canonical path; `run_module` walks with them (`NaiveWalk::with_stated`), and a test of the scanner
wraps its reader: `FactsLens::new(reader, tree.stated().clone())`. A stated fact is never what Finder or the shell
shows, and the cleaner reads the disk, never the lens: a fact that decides what the cleaner does is tested where it can
be made (a real pipe or socket under a short temporary folder, `make_pipe`). A file written now has a status change
of now: an item meant to look old states `changed` on every entry inside it.
A leftover that a module judges by its age (`app-leftovers`: a folder's newest change, a file's modification and
status change) states both `age` and `changed` on every entry, 40 days for an old one and 5 for a recent one; a missing
`changed` reads as now, so the leftover would count as recent.

The format is the public `TreeDescription`: `from_toml_str` checks it, `merge` joins two (a path written twice is
refused), `build_at(folder, now)` builds it in any existing folder and refuses a path that leaves the folder or passes
through a link. Built again into a folder it built before, it tops up: it rewrites its files and markers, writes what
is missing and keeps a link that is already as described.

## Sandboxes

A feature's cases live in its case file, `fixtures/sandboxes/<feature>.toml`; every module and every category ships
one (rule 8). `npm run sandbox -- <feature|all> [--into
<dir>]` (the binary of `fresher-sandbox`) writes it into a sandbox folder, `sandbox/<feature>/` at the repository's root
by default (gitignored): the marker `.fresher-sandbox` at its top (the signature, then a line naming the case files),
the fake home `home/` with its Trash `home/.Trash/`, `Applications/`, `System/Applications/`, the app's data folder
`.fresher-data/`, the two temporary places laid out as on a Mac from `/` (`private/var/folders/xx/sandbox/T/` with
`C/`, `0/` and `X/` beside it, and `private/tmp/`), every case's files, `EXPECTED.md`, the checklist of a run by hand
(one checkbox per case, negatives first), and `world.toml`: `now`, the moment of the generation every age counts back
from, the apps the cases declare open, what Spotlight answers (`spotlight = "off"`, `[[elsewhere]]`), what the guarded
folders allow (`[access]`), the macOS version (`macos`) and the installer receipts (`[[receipt]]`), and an `[[entry]]` per path whose files state facts (its path in the sandbox,
its times in seconds since the epoch, its owner, flags and kind). The sandbox's known folders `UserTemp` and
`SharedTemp` are those two places, never the Mac's, and a list of its own protects them as the Mac's list protects
the real ones (`Sandbox::places_protection`: every user's folders and `private/var/tmp` never, the sandbox's `T` carved
out and kept whole, its `TemporaryItems` never, `private/tmp` whole); the Mac's lists never carve the sandbox's
`T`. Sandbox mode's clock starts at that `now`, and the autotest's clock stands at it. `all` writes every case file
into one sandbox, joins their worlds, and leaves out whole a file that writes a path an earlier file wrote (named on
stderr and in `EXPECTED.md`).

- It refuses a folder that is not empty and holds no valid marker, and a folder the real Mac's protection list
  refuses (the home itself, a protected place or an ancestor of one).
- Links: it refuses a target folder that is itself a link, and any link inside the sandbox on a path it writes (a
  layout folder, a case's file or folder, the marker, `EXPECTED.md`, `world.toml`). Links above the target are
  resolved: the protection check runs on the real path, and a missing target is made there. A `.` or `..` in the part
  of the path that does not exist yet is refused.
- It never rewrites a file of its own that has a hard link it did not make (which may lie outside the sandbox): such
  a sandbox is refused before anything is written.
- It removes nothing: on a sandbox it made before it writes what is missing, resets its own files, and lists every
  entry it did not make (the app's data folder aside) without touching it. A clean start is the developer's own
  `rm -rf sandbox/<feature>`.

The case files:

- `app-caches.toml` and `app-caches-app-open.toml`: the `app-caches` module. A name that is not an app's id, Apple's
  caches, Fresher's own, a removed app's cache (`app-leftovers`' to decide), a vendor's folder above an installed app's
  id and a link named like an installed app's id are never claimed; a cache holding a socket is left out (`special`),
  and, in the second file, an open app's cache is left out (`app-open`); an installed app's cache and an updater's
  cache under the app's id are emptied (Low, the folder stays, its contents go to the Trash). An installed app is a
  `[[case.app]]` (the sandbox's apps declare no helpers, so a helper's folder is the fixtures' and the core e2e's);
  the names are apart from every other file's, so `all` leaves out none of them. The open Mailer of the second file is
  an installed app (`[[case.app]]` with `running = true`), so `EXPECTED.md` reads "Installed: Mailer (open)". Their
  checklist sentence names the module, which matters in `npm run dev:sandbox -- app-leftovers`, where the full app
  also offers that file's installed Editor's cache through `app-caches`.
- `app-caches-browsers.toml` and `app-caches-browsers-open.toml`: the `app-caches` module's browser and web-engine
  rows. A browser's cache with no installed browser (Firefox here), what a profile keeps beside its caches
  (`image_cache`, `Storage`), a web-engine app's own data (cookies, local storage, settings), a cache-named folder of an
  app not in the table and a link named like a cache are never claimed; in the second file an open browser's two
  caches are left out (`app-open`). A browser profile's `Cache` and `Code Cache`, a second profile's cache (its own
  item) and a web-engine app's cache, also while a browser is open, are emptied (Low, the folder stays, its contents
  go to the Trash). The world is shared by every case file of `npm run sandbox -- all`: Google Chrome is only the open
  browser (the second file, and `downloads-browser-open.toml`), Chromium is the closed one, Firefox is never installed
  and Redis Insight is installed and closed in both files (at `Applications/RedisInsight.app` in the second, so `all`
  leaves out neither file).
- `app-leftovers.toml` and `app-leftovers-spotlight-off.toml`: the `app-leftovers` module. The first: a name that is
  not an app's id, Apple's own files, a tool's files with no app behind them (no saved state proves an app), an
  installed app's files (its helper's and a related id's too), a link at a place, a group container and a container
  (`Library/Containers` is not opened in sandbox mode, so no container is offered here; the fixtures and the core e2e
  offer one) are never claimed, and an open app's files, files changed 5 days ago, an app Spotlight finds elsewhere
  and a leftover holding a socket or the system's file are left out (`running`, `used-recently`,
  `installed-elsewhere`, `special`, `not-yours`); a removed app's caches, logs and saved state go as Safe, its settings
  and its web data and cookies as Medium, its support folder as High. Every leftover's entries state their age and
  status change (40 days, or 5). The second runs with Spotlight off, a world of its own: a removed app's files are
  left out (`spotlight-unread`); in `npm run sandbox -- all` Spotlight is then off for every case file
  ([known issue](./known-issues/repo-sandbox-all-worlds-interact.md)).
- `trial.toml`: the `trial` module. A look-alike folder, a trial folder with no marker next to it, a file named like a
  trial folder and a plain file next to a marker are never offered; a cache next to a marker goes to the Trash, a kept
  folder is emptied.
- `cleanup-guards.toml`: the cleaner's guards, on the `trial` module. Links out of the sandbox are never followed; a
  file hard-linked into a trial cache keeps its content under its own name; the scan keeps out of `Documents` (a place
  that may prompt); an item inside the hard-blocked `Library/Caches` goes on its own; a trial cache inside the Trash is
  offered but left out as protected.
- `trash.toml`: the Trash line, on the `trial` module. What the user put in the Trash is never offered and counts in
  the first number only (what waits there); a trial cache's cleanup adds to both numbers.
- `trash-empty.toml`: the Trash line with an empty Trash, on the `trial` module. Nothing is said before the cleanup;
  after it, everything in the Trash is from cleanups here.
- `storage-map.toml`: the storage map, on the `trial` module, every case with a `[case.map]`. A folder no module
  claims, a look-alike of a trial cache, a hard-linked file (its two names take its blocks once) and the Trash are the
  user's own folders, never explained; a folder that is almost one child opens through it (`home/Library` down to
  `Data`); a trial cache is explained, and a trial cache inside one counts once, with the outer one. Its files are
  filled with data, so the map's numbers are blocks on disk.
- `jetbrains.toml`: the `jetbrains` module, with the sandbox's installed and open apps. The version in use, two versions
  installed side by side, a newer version, an open IDE (installed, or open from no apps folder), an app with no version,
  a removed IDE whose data changed recently, an IDE installed by the Toolbox app, an IDE linked into the apps folder (the list of apps stays whole), the
  Community edition beside the full one, names that only look like versions, the vendor's other folders, a version
  folder outside the vendor folder, and a file or a link named like one are never offered, each left out with its
  reason where the module recognizes it; older WebStorm, IntelliJ IDEA and Rider folders go to the Trash in their
  place's category, and a removed DataGrip's three folders, 40 days old, as the data of a removed IDE.
- `jetbrains-unknown-app-open.toml` and `jetbrains-unknown-app-installed.toml`: an app of the vendor that the module
  does not know, open (every folder kept) or installed (every removed IDE's data kept). Files of their own, since the
  app keeps the folders of every case beside it; in `npm run sandbox -- all` it keeps those of `jetbrains.toml` too
  ([known issue](./known-issues/repo-sandbox-all-worlds-interact.md)).

- `dev-caches.toml`, `dev-caches-unsigned.toml` and `dev-caches-xcode-open.toml`: the `dev-caches` module, in three
  files because its places are fixed, so a world where a sign is missing or Xcode is open cannot sit beside the
  signed one. The first: look-alike folders, a tool's settings, credentials and programs, stores that are not plain
  caches, Homebrew's own data, derived data that is no project, a link or a file in a project's place are never
  offered, and a project holding a socket or the system's file is left out (`special`, `not-yours`); each tool's cache
  goes to the Trash in its category (npm, Yarn, Homebrew with its links kept, Cargo, CocoaPods, Gradle, pip, Go, and a
  closed Xcode's project). The second writes every place without its sign: nothing is offered, nothing left out. The
  third runs with Xcode open: a project is left out (`app-open`), npm's cache still goes. Their worlds are their own:
  the autotest runs each alone and `npm run sandbox -- all` joins them for a look by hand only (there the unsigned
  files land in signed places), so no file writes a path another writes.

- `downloads.toml`, `downloads-denied.toml` and `downloads-browser-open.toml`: the `downloads` module, offered from
  `home/Downloads` only, with `access = { downloads = "granted" }` and `macos = "15.4"` in the first. A document, a
  disk image of no installed app, a copy of an installer, a link named like a download, a folder of yours and a big file
  inside one are never claimed; an unfinished download or a big file changed lately, an installer newer than the
  installed app or downloaded after it, and an installer for a newer macOS are left out (`recent`, `not-older`,
  `newer-macos`). An unfinished download (Chrome's, Firefox's with its empty placeholder), a Safari one at Medium, an
  installer of an installed app, a package with its receipt, an old macOS installer and a big file untouched for a
  long time (at High) go to the Trash. The big files are sparse: the map counts no blocks for them, and the harness
  caps the fake volume's used space to what it holds. The second denies access: nothing in Downloads is read. The
  third runs with Chrome open: its unfinished download is left out (`browser-open`). Denied access and an open browser
  are worlds of their own; in `npm run sandbox -- all` the most restrictive access wins, so Downloads is then denied for
  every case file ([known issue](./known-issues/repo-sandbox-all-worlds-interact.md)). The autotest does not opt in to
  Downloads in Settings, so a `not-asked` answer turning granted after the opt-in is not proven there but in the core
  e2e (`tests/downloads.rs`, below); a case file cannot opt in.

- `local-ai-models.toml` and `local-ai-models-lm-studio-open.toml`: the `local-ai-models` module. A Space, the hub's
  locks, a folder without blobs and snapshots, a name off the repository shape, a loose file in LM Studio's models
  folder, Ollama's folder without `manifests/` (its models are the next files'), a model file in a project and a
  project's link into the cache are never claimed, and a model holding a socket is left out (`special`); a Hugging
  Face model (with and without an owner, the model a project links to, whose link stays) and dataset go to the Trash
  as Low, an LM Studio model (the current place and the older) as Medium. The second runs with LM Studio open: its
  model is left out (`app-open`), a Hugging Face model still goes. Files and paths of their own, the open app being
  shared by every case of a file.

- `local-ai-models-ollama.toml`, `-ollama-unreadable.toml` and `-ollama-download.toml`: the module's Ollama store. A
  layer no manifest names, a manifest at another depth (a file below a tag's folder) and a model with a layer of the
  wrong size (left out `ollama-incomplete`) are never offered; a model, two tags with the same files (one finding), a
  model in another namespace and a model made from another that shares two layers go to the Trash as Medium, the
  shared layers and the other model's manifest staying where they were (the golden's tree after shows them). The
  other two files are worlds of their own, since a left-out of the whole store hides every model beside it: an
  unreadable manifest and a download in progress keep every Ollama model. Each writes the lone layer of the first file
  too, so in `npm run sandbox -- all` they are left out whole and named in `EXPECTED.md`; write them by name
  ([known issue](./known-issues/repo-sandbox-all-worlds-interact.md)).

- `logs.toml`: the `logs` module, offered from `home/Library/Logs`. A removed app's or a JetBrains IDE's folder (other
  modules' to decide), a link, an app's state and lock files, a folder named `Logs` outside the Library are never
  claimed; a log changed yesterday or 6 days ago, a socket, the system's log and a log you locked are left out
  (`recent`, `special`, `not-yours`, `locked`). An old log of an installed app (its folder stays), an old log with no app behind
  it and an old crash report (`DiagnosticReports`) go to the Trash as Safe. Every old entry states its age and status
  change (30 days).

- `stale-projects.toml`: the `stale-projects` module, offered from the folders under `home` the walk reads
  (`home/Code`). Source, `.git` and the project folder, `dist` and `.next`, a `node_modules` without a project, a link
  named `node_modules`, a `node_modules` in the home itself, a project in a hidden folder of the home and one inside an
  app bundle are never claimed; an active project's packages, a project with a recent commit and a Rust build cache
  used this week are left out (`recent`, `cache-recent`). An old Node project's packages, an old Rust project's `target`,
  an old Gradle project's `build`, an old Python project's `.venv` (at Medium), a monorepo's package and an active Rust
  project's unused incremental cache and coverage build go to the Trash. Every folder of a project is a dated
  `[[case.dir]]` (400 days for an old project, 3 for an active one, 10 for a cache unused for a week), parents of files
  and `.git/refs/heads` included: an undated folder counts as changed now and silently turns a stale case active.
  `home/Code` is dated once, in the first positive, because a negative owns every folder above its paths and none may
  own the folder holding another case's offer.

- `unused-toolchains.toml`, `unused-toolchains-unknown.toml`, `unused-toolchains-off.toml` and
  `unused-toolchains-documents-denied.toml`: the `unused-toolchains` module, with access to Desktop, Documents and
  Downloads granted in the first (the module needs them, since its pins may be there) and the Node switch on. Rustup's
  default and its override, a moving channel, a toolchain a project's `rust-toolchain.toml` (in Documents too), a legacy
  `rust-toolchain` or a `.tool-versions` names, a linked toolchain and a folder off the shape, nvm's default, the newest
  Node version, an alias and its chain, a project's `.nvmrc` (in Documents too) and a `.node-version` are never
  offered; a toolchain holding a socket is left out (`special`); an old toolchain and a dated nightly go to the Trash as
  Low, an old Node version (and one only a pin inside `node_modules` names) as Medium. The second has no rustup settings
  (`default-unknown`) and an nvm pin nothing resolves (`pin-unknown`); the third has the Node switch off (no Node
  version offered or left out, a toolchain still goes); the fourth denies Documents (the module is not run, nothing is
  offered). The settings and the access are shared by every case of a file and rustup's and nvm's pointer files sit
  at fixed places, so each world is a file of its own, and they write the same `home/.rustup/settings.toml` and
  `home/.nvm/alias/default`: `npm run sandbox -- all` orders the case files by file name without `.toml` (so a base
  file precedes its variants) and leaves out whole a file that writes a path an earlier file wrote, naming it on
  stderr and in `EXPECTED.md`; each runs by name. In `all` the main file is written but the module does not
  run there: `downloads-denied.toml` denies Downloads for every case file and the module needs it; a run by hand uses
  the feature's own sandbox
  ([known issue](./known-issues/repo-sandbox-all-worlds-interact.md)).

- `temp.toml`: the `temp` module, in the sandbox's two temporary places, with stated facts for what the disk cannot
  make. A fresh file, a folder unpacked with old modification times, an old folder with one file changed a day ago,
  an old file read a day ago, six days, a future time, an open app's items (also a hidden one), the system's and
  another user's items, a folder holding the system's file, an item you locked, a socket, a pipe and a folder holding
  one, lock names and a folder holding a lock-named entry are never offered, each left out with its reason; old files
  in `C/`, `0/`, `X/`, `T/TemporaryItems`, `~/Library/Autosave Information`, another user's `T` and editor files in
  the home are never claimed, nor the target of a link; an old file, an old folder, your old item in `private/tmp`,
  an installed app's folder, seven days and one second, and an old link (as itself) go to the Trash.
- `xcode.toml` and `xcode-open.toml`: the `xcode` module, offered from `home/Library/Developer/`. A folder that is not
  a system version, a device support folder without its symbols, a link where a version's folder would be (and its
  target), Xcode's own settings, a folder named like an archive without its contents list and the simulator devices
  are never claimed; a device support folder holding a socket is left out (`special`), and, in the second file, with
  Xcode open, a device support folder, an archive and the simulators' caches are left out (`app-open`); an iOS and a
  watch's device support version and an archive go to the Trash (Low, Low, High), the previews and the simulators'
  caches are emptied (Safe, the folder stays, its contents go to the Trash). `DerivedData` is `dev-caches`'. The
  open Xcode is a `[[case.running]]`, shared by every case of a file, so it has a file of its own. `all` writes every
  path but, with Xcode open in the joined world, `xcode.toml`'s positives are left out as `app-open`: walk `xcode` by
  name.

A case file's paths are its own across every case file, so `npm run sandbox -- all` can write them into one sandbox;
a file that writes a path an earlier file wrote is left out of `all` whole, named on stderr and in `EXPECTED.md`, and is
written by name (`npm run sandbox -- <name>`).

`npm run sandbox:test` (`app/tauri/tests/sandboxes.rs`) is the autotest of every case file, one `#[test]` each, listed
in its `case_files!`, run one at a time (the nextest test group `sandboxes`, `.config/nextest.toml`); a scan or a
cleanup not over within 180 s fails, naming the last event seen. The case file is generated into a temporary folder (the "real" home and data folder are
temporary too), run through the app's own sandbox composition (`sandbox_engine`) with fakes for the clock (standing at
the generated sandbox's `now`), volumes, open files and commands, the running apps of the generated sandbox's
`world.toml`, its access (a fake that knows nothing when it states none), its receipts and macOS version, and its
stated facts (as sandbox mode takes
them), and only the case file's `module` registered. Every negative must not
be offered (at or below its paths, nor at a folder above them that only it writes into) and must be unchanged after
the cleanup; every positive must be offered at its `item`, in its `category`, with its `danger`, and be as its
`after-cleanup` says once every positive is cleaned; nothing beside the sandbox may change. After the scan and before
the cleanup, the storage map is read: the sandbox sits on a fake volume of 1 GiB with the tree's size plus 64 MiB used
and 256 MiB free, so every part is the same on every Mac; the parts add up to the capacity, the by-meaning groups and
the by-place roots (with the parts that are no file of the scan) to the same total, and every `[case.map]` holds. The
report (findings with their app, the map, cleanup, the tree after, without the empty folders of the temporary places,
which every sandbox has) equals `fixtures/golden/sandbox-<name>.json` (`UPDATE_GOLDEN=1`
rewrites it); its lists are sorted by path, then category, and the Trash's entries lose the `<n>-` of the order they
were moved in, so the findings' ids, which differ from run to run, never show in it. The map's sizes are the space taken on disk, so the goldens assume the temporary folder is on an APFS
volume with 4 KiB blocks, as on every Mac and in CI (macOS): a byte of data takes one block and a sparse file none. A
golden check fails with that message on a volume that differs. A failure names its case. Guards: every case file has
a test, and every module the app registers (but `demo`, whose findings are made up) has a case file naming it.

Every report also holds the engine's Trash view (`trash`: `before` and `after` the cleanup, each line's `index`,
`volume`, `waiting` and `fromCleanups` in bytes on disk, and `goneBy`, never a path), and every run checks two
invariants on it. Before the cleanup, no line has a share from cleanups: nothing came from cleanups before the first
one. After it, the share is exactly the allocated sizes of the cleanup's items found in the Trash, all of it in line
0 (the sandbox's `home/.Trash`), whose `waiting` is known and not below it; with nothing moved, no line has a share.
A split of one Trash into two lines (the same folder spelled two ways) fails them, and so does a share counted
twice. `goneBy` is `null` in every golden, since the autotest's fake command runner answers no `defaults`. A failure
names the case file and both numbers.

A case file checks left-outs exactly: the scan's `ModuleFinished.left_out` of every registered module, summed per
category and reason, must equal the sum of the file's `[[case.left-out]]` expectations in items. A pair a module
reports that no case expects fails, naming it, and so does an expected pair that is not reported or differs in its
count (agent's call 6 of plan [011](./plans/011-diagnostics/00-design.md)). The report holds a `leftOut` list (the
engine's counts: category, reason, items, sizes) only when something was left out, so a case file that leaves nothing out
keeps its report.

### The case file

```toml
feature = "trial"   # the feature: a module's id, or a part of the core (`cleanup-guards`)
module = "trial"    # the module the autotest registers (absent: every module of the app)

# Negative cases first, then positive ones; names are unique; paths are relative to the sandbox folder, whose fake
# home is `home/`.
[[case]]
name = "a trial folder with no marker next to it is never offered"
kind = "negative"
[[case.file]]
path = "home/bare/fresher-trial-cache/blob.bin"
size = 1048576

[[case]]
name = "a trial cache next to a marker goes to the Trash"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "home/work/fresher-trial-cache/blob.bin"
size = 52428800
fill = "data"
[case.expect]
item = "home/work/fresher-trial-cache"
category = "trial.caches"
danger = "safe"
after-cleanup = "in-trash"   # in-trash | emptied | skipped (with `reason`, a SkipCode as serialized)
[[case.left-out]]            # optional, on any case: what the module recognizes and does not offer
category = "trial.caches"    # a category of the module's descriptor
reason = "trial.error.failed"   # one of the module's declared problems (`with_problems`)
items = 1                    # at least 1; the file's expectations of one category and reason add up
[case.map]                   # optional, on any case
part = "explained"           # explained (a positive) | your-folders (a negative)
at = "home/work/fresher-trial-cache"   # optional: by default the folder of the case's first file
```

A case's files use the fixture homes' vocabulary (`[[case.file]]`, `[[case.dir]]`, `[[case.symlink]]`,
`[[case.hardlink]]`, `[[case.marker]]`, with `age` and `fill`, the stated facts `changed`, `accessed`, `owner`, `flags`
and `kind`, and `[[case.app]]`). A path may start with `private/`: the temporary places. A stated fact goes to
`world.toml`, and `World::read` refuses an `[[entry]]` that states nothing, is not a plain path inside the sandbox, or
names a path the sandbox does not hold (or holds through a link; the entry itself may be a link, which a
`[[case.symlink]]`'s `changed` states); `World::stated` gives them at the sandbox's paths.
Sandbox mode and the autotest wrap the system's reader with them (`FactsLens`), so the scan and the modules see them;
Finder never does, and the cleaner reads the disk. `EXPECTED.md` names them under the case ("`private/tmp/x` stated:
owned by the system, changed 30 days ago."), since a run by hand cannot see them. A `[[case.app]]` is an installed
app the real installed-apps reader sees: it lies in `Applications/`, `System/Applications/` or `home/Applications/`,
directly or one folder below, and takes `running = true` when the app is open (`false` by default); a case writes
nothing but apps into `home/Applications/`, and nothing inside a declared app. A `[[case.running]]` (`name`,
`bundle-ids`: at least one, each a valid bundle id) is an app open from no apps folder of the sandbox. A
`[[case.elsewhere]]` (`name`, one plain file name; `bundle-id`, checked as a `[[case.app]]`'s) is an app Spotlight
finds outside the apps folders: sandbox mode and the autotest answer it at `<sandbox>/Volumes/Elsewhere/<name>.app`,
which is never written or read. `spotlight = "off"` on a case (the only value) means Spotlight cannot be asked, so no
app's absence can be proven; it holds for the whole sandbox, so a file with Spotlight off is a world of its own.
Both go to `world.toml` (`spotlight = "off"`, `[[elsewhere]]`) only when a case declares them; sandbox mode's
search answers the sandbox's listed apps and these, never the Mac's Spotlight. `access = { downloads = "granted" }` on
a case states what a guarded folder allows: the keys are `desktop`, `documents` and `downloads` (any other is
refused), the values `granted`, `denied` and `not-asked`. When two cases state one folder differently, the most
restrictive wins (`denied`, then `not-asked`, then `granted`), as `spotlight = "off"` does, so `npm run sandbox -- all`
stays valid; `world.toml` writes `[access]` only when a case states any. Sandbox mode and the autotest then answer
access from it (`World::access`): Full Disk Access `unknown`, a stated folder as stated, every other permission
`not-asked`; each stated folder is guarded at `home/<Folder>`, and trying it answers `granted` for `granted` and
`not-asked` (the simulated prompt is accepted once the folder's switch in Settings → Permissions is on), `denied` for
`denied`. As on a Mac without Full Disk Access, a stated folder is read only with its switch on, `granted` too; the
autotest turns on the switch of every stated folder. Without `[access]` they answer the stand-in that knows nothing,
as before. `macos = "15.4"` on a case states the macOS version (a version that does not parse is refused; two cases
stating different versions make `all` refuse, naming both), and each `[[case.receipt]]` (`package`, one plain file name
ending `.pkg`; `installed`, an age) an installer receipt installed that long before the sandbox was made; `world.toml`
writes `macos` and `[[receipt]]` only when stated. Sandbox mode and the autotest answer receipts and version from it
(`World::install_records`), never the Mac's: a world that states no version answers an error, "the sandbox states no
macOS version". `[[case.app]] changed = "10d"` states an app's change time, which `StatedInstalledApps` puts on the
installed app in sandbox mode and the autotest. `EXPECTED.md` names the version and each receipt. `EXPECTED.md` says which folders
a case allows, refuses or has not been asked for (and, for `not-asked`, that the switch is turned on first). `EXPECTED.md`
lists a case's apps under it ("Installed: WebStorm 2025.3.1 (open)", "Open, not installed: CLion", "Found by Spotlight
outside the apps folders: Far", and "Spotlight is off" for a case that turns it off) and its left-outs
("Under 'trial.caches', 2 left out: trial.error.failed"; the generator has no dictionary, so the category and the reason
are named by their ids). A negative
expects nothing to be offered at or below its paths, nor at a folder above them that only it writes into, and its paths untouched after the cleanup; it has no
`[case.expect]`. A positive's `item` is one of its own paths or their folders. Checked when read, by the generator and
the autotest alike: unique names, every negative before every positive, plain relative paths inside the sandbox, no
two cases writing a path, and `category` and `danger` that exist in the module's descriptor (the danger not below the
category's); a `[[case.left-out]]` has `items` of at least 1, a category of the descriptor and a reason the category's module
declares. `EXPECTED.md` in the generated sandbox is the same cases as a checklist, negatives first, one plain
sentence each (a negative reads "never offered by `<module>`", the module the case file names; only a case file that
declares no module would read "never offered", and every shipped file declares one, `cleanup-guards` and `storage-map`
the `trial` module), and one more under a case with a `[case.map]`.

`[case.map]` says where the core's by-place view shows the case's files (the UI no longer shows it, plan 027). `at` is
one of the case's paths or their folders, and is required when the case's first file is at the sandbox's root; the
autotest opens the case's folder with the core's place query (`storage_place` with `Descent::Through`) and checks
the rows that hold the case's files: `explained` in full, or for `your-folders` nothing explained and no finding at,
below or above `at`. A place that opens with no rows — a folder the scan does not go into, such as a claimed one, a
folder whose small files the scan folds into it, or a file — is checked by its own row in its folder's view. The
golden report holds the map's parts and, for every `at`, the folders the view went through and its rows.

The sandbox's home, `home/`, stands for the user's home: its own places that prompt (`Documents`, `Desktop`, …) are not
scanned, as on the real Mac before access is granted (plan 006). A case file therefore puts its positive cases under
`home/Library` or in a folder of its own in `home/`, never under those places; a negative may use them to show the
scan keeps out. The exception is a folder the world states `access` for: the standing avoid list then leaves its
place in `home/` to the engine's coverage, which decides it as on a Mac (walked when granted, tried once its switch is
on when not asked, skipped when denied); the real home's places stay avoided whatever the world says.

A case file may also have a file-level `[settings]` table, beside `feature` and `module`: the values of the
modules' settings, by key `<module>.<name>` in kebab case, each a bool, an integer or a string
(`"unused-toolchains.node-versions" = true`). A module's off-by-default switch is turned on this way. Like
`[[case.running]]` the table is shared by every case of the file. The generator writes it into the sandbox's data
folder's settings file, in the format the app's store reads, before the app starts (none is written when no file has the
table), and the autotest runs the engine on it: a key the registered modules do not know or a value of the wrong kind
fails the run, naming the key. In `npm run sandbox -- all`, two files that set one key to different values are
refused, as a path written twice is. A settings file left from an earlier run, or written by the app, stays when the
sandbox is regenerated; `rm -rf sandbox/<feature>` resets it.

### Sandbox mode

`npm run dev:sandbox -- <feature|path> [--real-trash]` (`scripts/sandbox-dev.ts`) generates `sandbox/<feature>/` when it
is missing (a path with a separator is used as it is) and starts the development build (`npm run dev`) with
`FRESHER_SANDBOX` set to it; the run is recorded as `sandbox-<name>` (below). The app then lives entirely in the sandbox
([`architecture/removal.md`](./architecture/removal.md), "Reach and the sandbox"): the scan, the settings, the history
and the Trash are the sandbox's, and the cleaner reaches only inside it; the walk's reader reports the facts of
`world.toml`'s `[[entry]]` lines instead of the disk's. `--real-trash` moves items into the system's
Trash instead of the sandbox's `home/.Trash`. A banner names the sandbox. A development build always runs as
`io.github.alexsergey.fresher.dev` (`npm run dev` adds `app/tauri/tauri.dev.conf.json`; a debug build with the release
identifier refuses to start, and so does a build without debug assertions with the development one), so its data and
permission grants are apart from the release's. `npm run dev:sandbox` and `npm run dev` share that identifier, so single-instance hands a sandbox launch to a development window that is already
running: that window keeps its own mode (its banner, or the lack of one, shows which) and the log warns; quit it first.
Before a plan that adds or changes a module or a category is done, its finish runs `npm run dev:sandbox -- <feature>`
and walks `EXPECTED.md` by hand.

### The development run record

`npm run dev` (`scripts/dev.ts`) starts the development build with `FRESHER_DEV_RUNS` (`NO_COMMIT/dev-runs/` of the
repository), `FRESHER_BUILD_COMMIT` and `FRESHER_BUILD_DIRTY` set, and takes no argument. Each run writes a pair named
with the start, the commit (`-dirty` when the tree has changes) and the mode (`sandbox-<name>` for `npm run
dev:sandbox`): `.events.jsonl` (append-only) and `.json` (the snapshot, rewritten atomically after each scan and
cleanup). The shape is in [`plans/026-dev-run-record/00-design.md`](./plans/026-dev-run-record/00-design.md), section
2.5. A record is never overwritten (a collision gets `-2`, `-3`, ...). Only debug builds have it; a bare
`tauri dev` and every test write nothing. `npm run dev` takes no argument; the bare `npm run tauri` script still
forwards anything, but a build without debug assertions (`tauri dev --release`, or debug assertions turned off) that
runs with the development identifier refuses to start, just as a debug build with the release identifier does
(`app/tauri/src/identity.rs`). A development build on the real Mac makes every cleanup
a dry run ([`architecture/removal.md`](./architecture/removal.md)).

`npm run dev-runs:check -- <record.json>` (`scripts/dev-runs-check.ts`) checks every finding and every cleanup item of a
record against the disk, read-only, and writes `<record>.check.md` beside it (never over an older report: `-2`, `-3`,
...). The agent runs it on the record the user hands over; the list of what it answers is in
[`manual-testing.md`](./manual-testing.md) section 2.0.

## Commands

| Command                        | Runs                                                                                  |
| ------------------------------ | ------------------------------------------------------------------------------------- |
| `npm test`                     | Rust tests, UI unit tests, the repository scripts' tests                              |
| `npm run test:rust`            | `cargo nextest run --workspace --locked`                                              |
| `npm run test:ui`              | `vitest run` in `app/ui`                                                              |
| `npm run test:scripts`         | `node --test scripts/*.test.ts`                                                       |
| `npm run test:cov`             | Rust and UI tests with coverage and the 90 % floors (reports in `coverage/`)          |
| `npm run test:e2e`             | `crates/e2e` and the Playwright suite                                                 |
| `npm run test:e2e:core`        | `cargo nextest run -p fresher-e2e --locked`                                           |
| `npm run test:e2e:ui`          | Playwright (needs `npx playwright install webkit chromium` once)                      |
| `npm run bench`                | the scan benches on a generated tree, by hand ("Performance")                         |
| `npm run sandbox -- <feature>` | writes the feature's case file into `sandbox/<feature>/` ("Sandboxes")                 |
| `npm run dev`                  | the development build on the real Mac, recorded; every cleanup a dry run              |
| `npm run dev:sandbox -- <feature>` | the development build in sandbox mode on `sandbox/<feature>/` (`--real-trash`) |
| `npm run dev-runs:check -- <record.json>` | checks a development run record against the disk, writes `<record>.check.md` |
| `npm run sandbox:test`         | the autotest of every case file of `fixtures/sandboxes/` ("Sandboxes")                |
| `npm run build:e2e -w app/ui`  | the app and the gallery into `app/ui/dist-e2e/` (the e2e suite builds it itself)      |
| `npm run test:perf -w app/ui`  | the performance spec, headed, by hand on a quiet Mac                                  |
| `cargo nextest run -p <crate>` | one crate                                                                             |
| Golden files                   | `UPDATE_GOLDEN=1 npm run test:e2e:core`; review the diff before committing            |
| IPC bindings                   | `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings`; review the diff |
| Screenshots                    | `npm run test:e2e:update --workspace app/ui`; look at the new image before committing |
