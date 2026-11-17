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
  fixture tree; every cleaner in a test is `Reach::Sandbox` and acts only below a fixture holding the sandbox marker
  (`FixtureTree::with_sandbox`); the tests of the system's Trash move items only on a disk image they mount, and no
  test lists or empties the user's Trash.

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
  - `run_module(&module, &tree, &Ports::at(seconds))` — one module run as the engine would, on the fixture tree as the
    home folder: walk, matches, `finish` → `ModuleRun { findings, outcome }`; `ModuleRun::golden()` is its stable JSON
    report, paths relative to the fixture root.
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
  and up and a protected, carved-out, hard-blocked and ignored folder; random plans; the swap case); its default run
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
  the same data folder keeps the lists, a cleanup plan over an ignored module drops its items).
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
  (`state/cleanup.test.ts`), its screens and the e2e mock.
- UI e2e and visual tests: `app/ui/e2e/`, the core mocked with `e2e/core-mock.ts` (answers, refusals, the events a
  scan streams through the channel, a scan held mid-stream until `releaseScan` or Stop, a scan paced by `interval`
  (one event every so many ms, each asked of Node, as the engine streams), the overview during and after the scan,
  `get_findings` paged from the findings streamed so far, a second scan refused while one runs, and the
  `setting-changed` event of an update); its cleanup part (`e2e/support/cleanup-mock.ts`) answers a plan per
  selection, compares the echo, streams a cleanup item by item (held after `hold` items until `releaseCleanup` or
  Stop), keeps a history and puts back, and `requestQuit` plays the close guard's `quit-requested`. `SCRIPTS` there are
  the scripts the flows and screenshots play (`cleaning`, `cleaningHeld`, `cleaningLeftOut`, `cleanupsEmpty` for the
  cleanup); `flows.spec.ts` walks select → review → confirm to the Trash → progress → result → Cleanups → put back,
  the permanent path, Stop mid-cleanup, the items left out, a refused plan and the quit question;
  its ignore part (`e2e/support/ignore-mock.ts`, `SCRIPTS.ignoring`) answers `ignore` and `restore_ignored` for
  modules, groups and items and keeps the lists for the session; `e2e/core-mock.spec.ts` proves its paging. Screenshots in `app/ui/e2e/screenshots/`, per browser. WebKit is the
  reference engine (the app runs in WebKit on macOS); Chromium runs too. The suite runs on `npm run build:e2e` (the
  app and the gallery in `dist-e2e/`), served with the app's production CSP; every spec checks that the policy
  refused nothing.
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
  the scan's screens or motion, and write the numbers in the plan's conclusions. `test:e2e` never runs it.

## Performance

The scan's timings and peak memory are measured by criterion benches run by hand, never in a hook (the UI's frame budget
is `perf.spec.ts`, above): `npm run bench` (`cargo bench -p fresher-scanner --bench scan`, a release build). The benches
live in `crates/scanner/benches/scan.rs`, grouped as `matcher/*`, `scan/*` (the full scan by thread count, the rescan
with nothing changed and after 1 000 changed folders, the restore: the cache decoded and the matches emitted), `cache/*`
(encode, decode, fit to the cap) and `state/*` (largest files, search by name); after each group the bench prints the
process's peak memory (`fresher_macos::peak_memory`, bytes).

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
# first, so the folder keeps it whatever the tree writes into it.
[[dir]]
path = "Library/Caches/JetBrains/DataGrip2024.3"
age = "40d"

# A symlink: listed, never followed. The target may point outside the root.
[[symlink]]
path = "Library/Caches/system"
target = "/Library/Caches"

# A hard link: a second name of a file of the tree; both paths are relative to the root.
[[hardlink]]
path = "Library/Caches/copy.db"
target = "Library/Caches/com.example.editor/Cache.db"

# A file may carry an age instead of `modified`: "<n>d" or "<n>h" counted back from the "now" of the build
# (FIXTURE_NOW for `FixtureTree`), so a stale file stays stale. `age` and `modified` never go together.
# `fill = "data"` writes real bytes of a fixed pattern instead of a sparse file, so freed space can be measured
# (at most 64 MiB a file and 512 MiB a description; a larger one is refused).
[[file]]
path = "Library/Caches/com.example.editor/old.bin"
size = 1048576
age = "30d"
fill = "data"

# A sandbox marker written into that folder (the folder is created if missing).
[[marker]]
path = "work"

# An installed app, written as a real bundle: `Contents/Info.plist` (an XML property list with CFBundleIdentifier,
# CFBundleName, CFBundlePackageType APPL and, when given, CFBundleShortVersionString and CFBundleVersion; text
# escaped) and an empty `Contents/MacOS/`. Unsigned: its team id reads as none. The path ends in `.app`; the bundle id
# must be one the app reads (`BundleId::new`). `version` and `build` are optional. No text holds a control character,
# and no other entry of the tree lies inside the bundle: the bundle writes its own contents.
[[app]]
path = "Applications/WebStorm.app"
bundle-id = "com.jetbrains.WebStorm"
name = "WebStorm"
version = "2025.3.1"
build = "WS-253.1"
```

`FixtureTree::from_toml(path)` builds it in a temporary folder that is removed when the tree is dropped;
`tree.root()` is the only folder a test may scan or clean.

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
`.fresher-data/`, every case's files, `EXPECTED.md`, the checklist of a run by hand (one checkbox per case,
negatives first), and `world.toml`: `now`, the moment of the generation every age counts back from, and the apps the
cases declare open. Sandbox mode's clock starts at that `now`, and the autotest's clock stands at it. `all` writes every case file into one sandbox, joins their worlds, and refuses a path two of them
write.

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

- `trial.toml`: the `trial` module. A look-alike folder, a trial folder with no marker next to it, a file named like a
  trial folder and a plain file next to a marker are never offered; a cache next to a marker goes to the Trash, a kept
  folder is emptied.
- `cleanup-guards.toml`: the cleaner's guards, on the `trial` module. Links out of the sandbox are never followed; a
  file hard-linked into a trial cache keeps its content under its own name; the scan keeps out of `Documents` (a place
  that may prompt); an item inside the hard-blocked `Library/Caches` goes on its own; a trial cache inside the Trash is
  offered but left out as protected.
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

A case file's paths are its own across every case file, so `npm run sandbox -- all` can write them into one sandbox.

`npm run sandbox:test` (`app/tauri/tests/sandboxes.rs`) is the autotest of every case file, one `#[test]` each, listed
in its `case_files!`, run one at a time (the nextest test group `sandboxes`, `.config/nextest.toml`); a scan or a
cleanup not over within 180 s fails, naming the last event seen. The case file is generated into a temporary folder (the "real" home and data folder are
temporary too), run through the app's own sandbox composition (`sandbox_engine`) with fakes for the clock (standing at
the generated sandbox's `now`), volumes, open files, commands and access, the running apps of the generated sandbox's
`world.toml` (as sandbox mode takes them), and only the case file's `module` registered. Every negative must not
be offered (at or below its paths, nor at a folder above them that only it writes into) and must be unchanged after
the cleanup; every positive must be offered at its `item`, in its `category`, with its `danger`, and be as its
`after-cleanup` says once every positive is cleaned; nothing beside the sandbox may change. After the scan and before
the cleanup, the storage map is read: the sandbox sits on a fake volume of 1 GiB with the tree's size plus 64 MiB used
and 256 MiB free, so every part is the same on every Mac; the parts add up to the capacity, the by-meaning groups and
the by-place roots (with the parts that are no file of the scan) to the same total, and every `[case.map]` holds. The
report (findings with their app, the map, cleanup, the tree after) equals `fixtures/golden/sandbox-<name>.json` (`UPDATE_GOLDEN=1`
rewrites it); its lists are sorted by path, then category, and the Trash's entries lose the `<n>-` of the order they
were moved in, so the findings' ids, which differ from run to run, never show in it. The map's sizes are the space taken on disk, so the goldens assume the temporary folder is on an APFS
volume with 4 KiB blocks, as on every Mac and in CI (macOS): a byte of data takes one block and a sparse file none. A
golden check fails with that message on a volume that differs. A failure names its case. Guards: every case file has
a test, and every module the app registers (but `demo`, whose findings are made up) has a case file naming it.

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
`[[case.hardlink]]`, `[[case.marker]]`, with `age` and `fill`, and `[[case.app]]`). A `[[case.app]]` is an installed
app the real installed-apps reader sees: it lies in `Applications/`, `System/Applications/` or `home/Applications/`,
directly or one folder below, and takes `running = true` when the app is open (`false` by default); a case writes
nothing but apps into `home/Applications/`, and nothing inside a declared app. A `[[case.running]]` (`name`,
`bundle-ids`: at least one, each a valid bundle id) is an app open from no apps folder of the sandbox. `EXPECTED.md`
lists a case's apps under it ("Installed: WebStorm 2025.3.1 (open)", "Open, not installed: CLion") and its left-outs
("Under 'trial.caches', 2 left out: trial.error.failed"; the generator has no dictionary, so the category and the reason
are named by their ids). A negative
expects nothing to be offered at or below its paths, nor at a folder above them that only it writes into, and its paths untouched after the cleanup; it has no
`[case.expect]`. A positive's `item` is one of its own paths or their folders. Checked when read, by the generator and
the autotest alike: unique names, every negative before every positive, plain relative paths inside the sandbox, no
two cases writing a path, and `category` and `danger` that exist in the module's descriptor (the danger not below the
category's); a `[[case.left-out]]` has `items` of at least 1, a category of the descriptor and a reason the category's module
declares. `EXPECTED.md` in the generated sandbox is the same cases as a checklist, negatives first, one plain
sentence each, and one more under a case with a `[case.map]`.

`[case.map]` says where the overview's by-place view shows the case's files. `at` is one of the case's paths or their
folders, and is required when the case's first file is at the sandbox's root; the autotest opens it as the overview opens a folder row (`storage_place` with `Descent::Through`) and checks
the rows that hold the case's files: `explained` in full, or for `your-folders` nothing explained and no finding at,
below or above `at`. A place that opens with no rows — a folder the scan does not go into, such as a claimed one, a
folder whose small files the scan folds into it, or a file — is checked by its own row in its folder's view. The
golden report holds the map's parts and, for every `at`, the folders the view went through and its rows.

The sandbox's home, `home/`, stands for the user's home: its own places that prompt (`Documents`, `Desktop`, …) are not
scanned, as on the real Mac before access is granted (plan 006). A case file therefore puts its positive cases under
`home/Library` or in a folder of its own in `home/`, never under those places; a negative may use them to show the
scan keeps out.

### Sandbox mode

`npm run dev:sandbox -- <feature|path> [--real-trash]` (`scripts/sandbox-dev.ts`) generates `sandbox/<feature>/` when it
is missing (a path with a separator is used as it is) and starts the development build (`npm run dev`) with
`FRESHER_SANDBOX` set to it. The app then lives entirely in the sandbox
([`architecture/removal.md`](./architecture/removal.md), "Reach and the sandbox"): the scan, the settings, the history
and the Trash are the sandbox's, and the cleaner reaches only inside it. `--real-trash` moves items into the system's
Trash instead of the sandbox's `home/.Trash`. A banner names the sandbox. A development build always runs as
`io.github.alexsergey.fresher.dev` (`npm run dev` adds `app/tauri/tauri.dev.conf.json`; a debug build with the release
identifier refuses to start), so its data and permission grants are apart from the release's. `npm run dev:sandbox` and
`npm run dev` share that identifier, so single-instance hands a sandbox launch to a development window that is already
running: that window keeps its own mode (its banner, or the lack of one, shows which) and the log warns; quit it first.
Before a plan that adds or changes a module or a category is done, its finish runs `npm run dev:sandbox -- <feature>`
and walks `EXPECTED.md` by hand.

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
| `npm run dev:sandbox -- <feature>` | the development build in sandbox mode on `sandbox/<feature>/` (`--real-trash`) |
| `npm run sandbox:test`         | the autotest of every case file of `fixtures/sandboxes/` ("Sandboxes")                |
| `npm run build:e2e -w app/ui`  | the app and the gallery into `app/ui/dist-e2e/` (the e2e suite builds it itself)      |
| `npm run test:perf -w app/ui`  | the performance spec, headed, by hand on a quiet Mac                                  |
| `cargo nextest run -p <crate>` | one crate                                                                             |
| Golden files                   | `UPDATE_GOLDEN=1 npm run test:e2e:core`; review the diff before committing            |
| IPC bindings                   | `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings`; review the diff |
| Screenshots                    | `npm run test:e2e:update --workspace app/ui`; look at the new image before committing |
