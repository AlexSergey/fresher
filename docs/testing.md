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
  fixture tree.

## Kinds of tests

| Kind           | What it proves                                                                   | Rust core                                   | UI                                         |
| -------------- | -------------------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------ |
| Unit           | one function or module                                                           | `#[cfg(test)] mod tests`, cargo-nextest     | Vitest + Testing Library + happy-dom       |
| Property-based | a rule holds for ANY valid input (sizes add up, versions order, no double count) | proptest (`mod property`)                   | fast-check (where logic exists)            |
| Integration    | one crate through its public API, on fixture trees                               | the crate's `tests/`                        | —                                          |
| Golden reports | a fixture home gives exactly the expected report                                 | `crates/e2e` against `fixtures/golden/`     | —                                          |
| Core e2e       | command → engine → modules → events → settings file, end to end                  | `crates/e2e` on fixture homes               | —                                          |
| UI e2e         | the user flows of the interface, on a mocked core                                | —                                           | Playwright on the Vite build, mocked IPC   |
| Visual         | the gallery's stories and the screens did not change unnoticed                   | —                                           | Playwright screenshots, ≤ 50 pixels differ |
| Accessibility  | keyboard, focus, contrast, page zoom and WCAG 2.1 AA on every story and screen   | —                                           | Playwright and axe; VoiceOver by hand      |
| Performance    | the scan stays within its time and memory budget; the UI within its frame budget | criterion benches on a large generated tree | `test:perf`, run by hand                   |

The full app (WebView + Rust) cannot be driven by WebDriver on macOS (`tauri-driver` supports Linux and Windows only),
so the UI e2e run against the web build with the Tauri IPC mocked, and the core e2e cover the Rust side end to end.
The core e2e prove the harness (`smoke.rs`: a fixture home is built and read back against
`fixtures/golden/smoke.json`), the whole Rust path on the `demo` module (`demo.rs`: the engine, the `demo` module and a
real settings file, the events and the overview against `fixtures/golden/demo.json`; the settings survive a restart; a
settings file from a newer version is never overwritten), and since plan 004 scan → classify → cache: `scan.rs` (the
engine with the real scanner on fixture homes, `fixtures/golden/scan.json`), `rescan.rs` (a scan, a scripted change, a
rescan that reads only the changed folders and the check's sample, a restore in a new engine that reads no folder,
`fixtures/golden/rescan.json`) and `footprint.rs` (below). Later plans extend it to diff → confirm → clean.

## Where things live

- Rust unit tests sit next to the code (`#[cfg(test)] mod tests { mod negative {…} mod positive {…} }`); integration
  tests in each crate's `tests/`. Helpers of an integration test go in a `#[cfg(test)]` module, so Clippy treats them
  as test code.
- `crates/testing` — shared test helpers: `FixtureTree` (a fake home folder from TOML), `TreeReport` (a deterministic
  listing for golden files), `FakeClock`, `FakeTrash`, `FakeCommandRunner`. The ports they fake (`Clock`, `Trash`,
  `CommandRunner`) live in `fresher-core`. For modules ([`architecture/modules.md`](./architecture/modules.md)):
  - `NaiveWalk` — the reference `DiskWalk` over a fixture root, with `std::fs`: links listed, never followed, nothing
    on another device; the scanner is tested against it (the same fixture gives the same matches).
  - The fakes of the scan's ports: `FakeVolumes` (fixed facts per root), `FakeChangeJournal` (a script of positions,
    changed folders and gaps, so no test waits on the system's journal), `MemoryScanCache`, `FixedDiskState` (a tree
    from paths and sizes, for the engine's queries).
  - `LargeTree` — the generated trees of the benches and the large tests ("Performance").
  - `run_module(&module, &tree, &Ports::at(seconds))` — one module run as the engine would, on the fixture tree as the
    home folder: walk, matches, `finish` → `ModuleRun { findings, outcome }`; `ModuleRun::golden()` is its stable JSON
    report, paths relative to the fixture root.
  - `assert_module_contract(&module, &Catalogue)` — the descriptor is valid, every category has its five texts and
    every setting (`<module>.enabled` included) its label and description, every key is in the module's namespace;
    `Catalogue::Required(docs/categories)` also wants each category's catalogue file with the same danger level
    (`Catalogue::DevOnly` for the `demo` module). `assert_texts_used(&module, &runs)` — every text is a descriptor key
    or used by a finding of the runs, and every finding's text exists with the args its placeholders name. Their
    `check_…` forms return the list of `ContractProblem`s instead of failing.
- `crates/scanner/tests/reference.rs` — the scanner against `NaiveWalk`: the same matches and the same summary on
  every fixture home and on random trees (proptest, its failing seeds kept in `reference.proptest-regressions`), with
  the portable reader; `macos_reader.rs` runs the same with the macOS reader. `CountingReader` counts the folder reads,
  so a test holds "one bulk read per folder" and "a rescan with nothing changed reads only the sample and the
  unreadable folders".
- `crates/macos` — tested on temporary folders and on disk images the tests mount with `-nobrowse` (a test skips itself
  when `hdiutil` is missing); nothing reads the home folder.
- `crates/rules` — the shared matcher of rule data (`RuleSet::parse`, `interests`, `evaluate`, `stale`), tested on
  `FixtureTree`s and `FakeClock`, with property tests of the glob and of the version order.
- `crates/e2e` — the core's end-to-end and golden tests (a crate with tests only).
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
  `src/test/fake-clock.ts` is the clock the store's timers are tested on.
- UI e2e and visual tests: `app/ui/e2e/`, the core mocked with `e2e/core-mock.ts` (answers, refusals, the events a
  scan streams through the channel, a scan held mid-stream until `releaseScan` or Stop, a scan paced by `interval`
  (one event every so many ms, each asked of Node, as the engine streams), the overview during and after the scan,
  `get_findings` paged from the findings streamed so far, a second scan refused while one runs, and the
  `setting-changed` event of an update); `SCRIPTS` there are the scripts the flows and screenshots play;
  `e2e/core-mock.spec.ts` proves its paging. Screenshots in `app/ui/e2e/screenshots/`, per browser. WebKit is the
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

# A symlink: listed, never followed. The target may point outside the root.
[[symlink]]
path = "Library/Caches/system"
target = "/Library/Caches"

# A hard link: a second name of a file of the tree; both paths are relative to the root.
[[hardlink]]
path = "Library/Caches/copy.db"
target = "Library/Caches/com.example.editor/Cache.db"
```

`FixtureTree::from_toml(path)` builds it in a temporary folder that is removed when the tree is dropped;
`tree.root()` is the only folder a test may scan or clean.

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
| `npm run build:e2e -w app/ui`  | the app and the gallery into `app/ui/dist-e2e/` (the e2e suite builds it itself)      |
| `npm run test:perf -w app/ui`  | the performance spec, headed, by hand on a quiet Mac                                  |
| `cargo nextest run -p <crate>` | one crate                                                                             |
| Golden files                   | `UPDATE_GOLDEN=1 npm run test:e2e:core`; review the diff before committing            |
| IPC bindings                   | `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings`; review the diff |
| Screenshots                    | `npm run test:e2e:update --workspace app/ui`; look at the new image before committing |
