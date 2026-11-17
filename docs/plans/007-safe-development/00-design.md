# 007 — Safe development: design

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** testing, sandbox, cleaner, core, app,
app/ui, repo

Approved by the user on 2026-11-16: the three open questions answered with the recommended options and every agent's
call accepted ("Decisions"). Built only after plans 005 and 009 are merged into `main`.

## Intent

The user's task of 2026-11-15 (the one-file draft): "We also need debug and emulation tools for tests and safe
development — so that we do not wreck something by accident during development."

The user's direction of 2026-11-16, translated: "The repository has scripts that generate fixtures for every feature —
its positive and negative cases of files — into a given folder (that folder can be in the repository under
.gitignore). After generating, the dev mode of the app takes the path to that folder as an argument, so the app works
as if inside that sandbox. Then we can write autotests that check all the cases, and also check by hand before moving
on."

So one description of a feature's cases serves three uses: a folder the developer can open the app on, the autotests,
and the checklist of a run by hand. The app in that mode lives entirely inside the folder: its home, its settings, its
Trash.

## Goals

1. **One source of truth per feature**: a declarative case file lists the feature's positive and negative cases —
   the files of each and what must happen to them. The generator and the autotests read the same file.
2. **A sandbox the app runs in**: a development build started on a sandbox folder sees that folder as the whole Mac's
   user side — home, known folders, scan roots, its own data and the Trash — and cannot touch anything outside it.
3. **Every case checked automatically**: scan, what is offered and with which danger level, what a cleanup does,
   what it leaves; the same path as the app, with deterministic fakes only where the world outside the disk comes in.
4. **The same cases checked by hand**: the generator writes a checklist next to the files.
5. **The remaining guards of the draft** that are cheap and real: a separate identity for development builds, the home
   folder looked up in one place, modules that start no process.

## Non-goals

- Anything of this in a release build: the sandbox mode, the generator and the fakes exist in development builds and
  tests only.
- Tools for end users (plan [011](../011-diagnostics/readme.md)) and the sample data of onboarding (plan
  [014](../014-onboarding/readme.md); it may reuse a case file's format later, but it runs in release builds, where
  this plan's mode does not exist).
- The debug panel and the developer command line beyond the generator (section 9: deferred, with reasons).
- New cleanup categories: the first case files cover the dev-only `trial` module and the cleaner's guards; each later
  module plan brings its own case file (section 8).

## What already exists (plans 004, 008, 009, 005)

| Draft promise                     | Built by                                                                                                                                                                                                 |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The sandbox marker                | 008: `SANDBOX_MARKER` (`.fresher-sandbox`) and `SANDBOX_SIGNATURE` (`crates/core/src/cleanup/sandbox.rs`); `Reach::Sandbox` admits an item only below a valid marker at or below its scan root (`crates/cleaner/src/reach.rs`) |
| "A debug build cannot clean the real disk" | 008: `RealDiskToken` is made only without `debug_assertions`, only in `app/tauri/src/compose.rs` (`crates/cleaner/tests/reach.rs`)                                                              |
| The dry run                       | 008: `rehearse` with `Depth::Full`                                                                                                                                                                       |
| Fixture trees                     | 002/004: `FixtureTree` (TOML → a temporary tree: sparse files with a logical size, times, folders, symlinks, hard links), `with_sandbox`, `mark_sandbox`; `fixtures/homes/*.toml`, `fixtures/golden/*.json` |
| Large generated trees             | 004: `LargeTree` (`TreeSpec::SMALL`, `LARGE`) for the benches                                                                                                                                           |
| Fakes of the ports                | 008: `FakeTrash`, `FakeOpenFiles`, `FakeVolumes`, `FakeCleaner`, `MemoryCleanupJournal`; 009: `FakeKnownFolders::home_layout`, `FakeInstalledApps`, `FakeRunningApps`, `FakeCommandRunner` with cancel   |
| Known folders                     | 009: `KnownFolders` and `KnownFolder::{Home, Library, AppCaches, …, Trash, Applications, …}`; modules name places by them (`Pattern::Under`)                                                              |
| Where the app keeps its data      | 005: the data folder (Tauri's `app_data_dir`): `settings.json` with the scan roots and the ignore list, the scan cache, the cleanup journal                                                              |

This plan extends these; it adds no second fixture format, no second fake and no second deletion point.

## Design

### 1. The case file

One file per feature: `fixtures/sandboxes/<feature>.toml`. A feature is a module (its id) or a part of the core
(`cleanup-guards`). Paths are relative to the sandbox folder; the fake home is `home/`.

```toml
# fixtures/sandboxes/trial.toml
feature = "trial"            # the module whose cases these are, or a core feature
module = "trial"             # the module the autotest registers (absent: every module of the app)

# Negative cases first, as in every test of the repository.
[[case]]
name = "a look-alike folder is never offered"
kind = "negative"
[[case.file]]
path = "home/work/fresher-trial-cache2/look-alike.bin"
size = 4194304
age = "30d"                  # modified 30 days before the sandbox's "now"

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
path = "home/work"           # writes the sandbox marker into this folder
[[case.file]]
path = "home/work/fresher-trial-cache/blob.bin"
size = 52428800
fill = "data"                # real blocks, so the freed space can be measured (default: sparse)
[case.expect]
item = "home/work/fresher-trial-cache"
category = "trial.caches"
danger = "safe"
after-cleanup = "in-trash"   # in-trash · emptied · skipped
```

- **The files of a case** use the `FixtureTree` vocabulary unchanged — `[[case.file]]`, `[[case.dir]]`,
  `[[case.symlink]]`, `[[case.hardlink]]` — plus `[[case.marker]]` (`mark_sandbox`). Two additions to the
  vocabulary, also open to `fixtures/homes/`: `age = "<n>d|h"` beside `modified` (relative to the tree's "now", so a
  stale-cache case stays stale; the generator's "now" is the time it runs, the autotest's is its `FakeClock`), and
  `fill = "data"` (real bytes instead of a sparse file, capped at 64 MiB a file and 512 MiB a sandbox).
- **The expectation** of a positive case: the finding's `item`, its `category` and `danger` (the category's level, or
  a raised one), and what a cleanup does: `in-trash` (moved into the sandbox's Trash), `emptied` (the folder kept, its
  contents in the Trash) or `skipped` with the expected `reason` (one of the core's `SkipCode`s, as they are
  serialized). A negative case expects nothing to be offered at or below any of its
  paths, and every path untouched after the cleanup of all positives; it needs no `[case.expect]`. (2026-11-17: a finding at a folder above a negative's paths that only this case writes into fails it too, so a look-alike offered at its own folder is caught.)
- **Checked when read**, by the generator and the autotest alike: names unique; every negative before every positive;
  every path relative, plain and inside the sandbox; no two cases writing the same path; a positive's `item` is one of
  its own paths or their folders; `category` and `danger` exist in the module's descriptor, and the danger is not below
  the category's.
- **The exception**: what TOML cannot describe — millions of entries for the benches — stays a generator in code
  (`LargeTree`); no feature gets a shell script.

### 2. The sandbox folder

```
<sandbox>/
  .fresher-sandbox        the marker, its first line SANDBOX_SIGNATURE, then a line naming the case files
  home/                   the fake home: every known folder of the user under it (FakeKnownFolders::home_layout)
    .Trash/               the sandbox's Trash (KnownFolder::Trash)
  Applications/           KnownFolder::Applications of the sandbox (empty until a case writes there)
  System/Applications/    KnownFolder::SystemApplications of the sandbox
  .fresher-data/          the app's data folder in sandbox mode: settings, scan cache, cleanup journal
  EXPECTED.md             the checklist of a run by hand
```

- `~` in the scan roots is `home/`: the default scan root of plan 005 needs no change in a sandbox.
- The marker sits at the sandbox's top, above `home/`, because a marker never counts in a home folder (plan 008).

### 3. The generator

`npm run sandbox -- <feature|all> [--into <dir>]` runs `cargo run -p fresher-sandbox -- generate …` (a dev-only crate,
section 7).

- Writes into `sandbox/<feature>/` at the repository's root by default (`sandbox/` added to `.gitignore`), or into
  `--into <dir>`. `all` merges every case file into one sandbox; a path two features both write is refused.
- **Refuses** a folder that is not empty and holds no valid marker, a folder inside a protected place of the real Mac
  (the macOS protection list for the real home: never the home itself, a hard-blocked folder or a system place), and a
  path through a link.
- **Never removes anything** (decision 4): on a folder it made before, it tops up — writes every missing entry,
  resets sizes and times of its own files — and lists any entry it did not make, without touching it. A clean start is
  the developer's own act (`rm -rf sandbox/<feature>`), as for any folder.
- Writes `EXPECTED.md`: one checkbox per case, negatives first ("never offered: `home/bare/fresher-trial-cache`; still
  there after the cleanup"; "offered in *Trial caches* (Safe): `home/work/fresher-trial-cache`; in the Trash after the
  cleanup"), and the command that opens the app on it.

### 4. Sandbox mode of the app

`npm run dev:sandbox -- <feature|path> [--real-trash]` builds the case's sandbox if it is missing, then runs
`tauri dev` with `FRESHER_SANDBOX=<path>` (and `FRESHER_SANDBOX_TRASH=real` for `--real-trash`). The app reads the two
variables in one place, its setup, in a debug build only; a release build never reads them (section 6).

In sandbox mode the composition (`app/tauri/src/compose.rs`) changes exactly these ports:

| Port / input        | Normal development build             | Sandbox mode                                                                    |
| ------------------- | ------------------------------------ | ------------------------------------------------------------------------------- |
| Known folders, home | `MacKnownFolders::new(real home)`    | the sandbox's layout (`home/`, `Applications/`, `System/Applications/`)          |
| Data folder         | the development build's own (§ 6)    | `<sandbox>/.fresher-data`                                                        |
| Trash               | `MacTrash` (the real Trash)          | `FakeTrash` into `home/.Trash`; `MacTrash` with `--real-trash`                   |
| Reach               | `Reach::Sandbox`                     | `Reach::SandboxFolder(<sandbox>)` (below)                                        |
| Protection          | the list for the real home           | the list for the real home **and** the list for the sandbox's home, both applied |
| Everything else     | real                                 | real and read-only: volumes, open files, running apps, commands, clock, access   |

- **Refusals at start** (the app shows the error and does not start): the path holds no valid marker, is a link, or
  is protected on the real Mac; `home/` is missing.
- **`Reach::SandboxFolder`** (new, `fresher-cleaner`): an item counts only when it is strictly inside the sandbox
  folder by components and the folder still holds a valid marker of the user, checked right before each item like
  today's check 1. The folder itself was checked at start against the real Mac's protection list. It is needed
  because the sandbox's own protection list makes `home/` a whole folder, so a marker in or above it would not count
  under `Reach::Sandbox`. It is narrower than `Reach::Sandbox` (one folder), never runs a tool (as the sandbox today),
  and grants nothing a release build lacks.
- **Both protection lists apply**: the real home's list keeps the real Mac safe whatever a bug does; the sandbox
  home's list makes the sandbox behave like a Mac (its `Documents` hard-blocked, its `.Trash` never touched). The
  core's `Protection` gains a way to join two lists.
- **A visible banner** in every development build: "Sandbox · <path>" in sandbox mode; "Development build · nothing
  can be removed outside a sandbox" otherwise. The core says which (a `build` field in the app's info answer); the UI
  renders it.
- **The real Trash** (`--real-trash`) is the only way out of the sandbox, and only for its own items: what the
  deferred [sandbox run](../../deferred/removal-sandbox-run.md) of plan 008 needs to see the system's Trash, put back
  and Finder's Put Back.
- Running apps, open files and commands stay real: they are read, never changed, and a tool removal never runs in a
  sandbox. A real running app that owns a sandbox folder's name makes it skipped, as on a Mac; the autotests use fakes
  and are not affected. Declaring them per case is deferred (section 10).

### 5. Autotests from the case files

`app/tauri/tests/sandboxes.rs`: one test per case file, through the app's own sandbox composition (a
`pub fn sandbox_engine` of `fresher-app`, debug builds only), with these ports replaced by fakes for determinism:
`FakeClock` (the case file's "now"), `FakeVolumes`, `FakeOpenFiles` (none), `FakeRunningApps` (none), `FakeCommandRunner`
(no answers), `FakeAccess`.

For each case file:

1. generate it into a temporary folder (the generator's own code, not a copy);
2. scan; **each negative** is in no finding, at or below any of its paths; **each positive** is found at its `item`,
   in its `category`, with its `danger`;
3. prepare a cleanup of every positive, confirm it, execute it;
4. **each positive** is as `after-cleanup` says (in `home/.Trash`, emptied, or skipped with its reason); **each
   negative** is byte for byte the same (`TreeReport` before and after); nothing outside the sandbox changed;
5. the report (findings, cleanup outcome, the tree after) equals `fixtures/golden/sandbox-<feature>.json`
   (`UPDATE_GOLDEN=1`, reviewed like every golden file).

A failure names the case by its `name`. A guard test fails when a case file has no test, and an app test fails when a
module the app registers has no case file (section 8).

The UI e2e stay on the mocked core: the full app cannot be driven on macOS (`docs/testing.md`), so a sandbox is checked
by hand in the real window (section 3's checklist), and its Rust side by these tests.

### 6. Guards kept from the draft

- **A separate identity**: `npm run dev` and `npm run dev:sandbox` run `tauri dev --config
  app/tauri/tauri.dev.conf.json`, which sets the identifier `io.github.alexsergey.fresher.dev`: its own data folder,
  its own single-instance lock and its own permission grants (what plan [013](../013-release/readme.md) expects). A
  debug build started with the release identifier refuses to start, so a forgotten flag never opens the release's
  data.
- **The home folder is looked up in one place**: Clippy's `disallowed-methods` gains `std::env::home_dir` and Tauri's
  `PathResolver::home_dir`; the one use is the app's setup, under `#[expect]`.
- **Modules start no process and read no environment**: the crate guard (`scripts/check-crate-deps.ts`) also refuses
  `std::process` and `std::env` in `modules/*/src`; a module runs a tool only through `CommandRunner`.
- **No dev code in release**: the sandbox mode is compiled under `cfg(debug_assertions)` only, like the dev-only
  modules today (decision 6); a test built with `--release` checks that `FRESHER_SANDBOX` is ignored, and plan
  013's release checklist looks for the variable's name in the shipped binary.

### 7. Where the code lives

- **`fresher-testing`**: the tree vocabulary made reusable — `TreeDescription` (today's private `Description`, public)
  and `build_at(folder, now)`, used by `FixtureTree` (into a temporary folder, as before) and by the generator; `age`,
  `fill = "data"`. No change for existing fixtures.
- **`fresher-sandbox`** (new, dev-only, `publish = false`, depends on core and testing, and on macos for its
  binary's protection list of the real Mac): the case format and its checks,
  the sandbox layout (`Sandbox::open(path)`: marker, home, known folders, data folder, Trash folder), the generator, the
  `EXPECTED.md` writer, and the binary `fresher-sandbox`. Allowed in the crate guard for the app and the tests only.
- **`fresher-cleaner`**: `Reach::SandboxFolder`. **`fresher-core`**: joining two `Protection`s.
- **`fresher-app`**: the sandbox composition, the start refusals, `sandbox_engine` for the tests, the `build` field;
  `tauri.dev.conf.json`. **`app/ui`**: the banner. **`scripts/`**: `sandbox-dev.ts` (the npm command), the crate guard's
  module check.
- No new third-party dependency: the generator's arguments are read by hand, as the other dev commands do.

### 8. A rule for every later plan

Added to [`../README.md`](../README.md) and to rule 8 of [`../../rules.md`](../../rules.md) when this plan is built:
**every plan that adds or changes a module or a category ships its case file `fixtures/sandboxes/<feature>.toml`, with
negative cases first and positive cases after, and its golden report**. Held by the app test of section 5 (a registered
module without a case file fails). Its run by hand before a plan is done: `npm run dev:sandbox -- <feature>` and
`EXPECTED.md`.

### 9. Cut and deferred from the draft

| Draft part                           | Now                                                                                                                                        |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Sandbox marker, debug-only removal   | built by 008; extended here (`SandboxFolder`)                                                                                              |
| "Real scanning needs a flag"         | **cut**: scanning is read-only by rule and by lint; a flag adds friction and protects no file                                              |
| Separate identity                    | **kept** (section 6)                                                                                                                       |
| `HOME` set to a temporary folder for tests | **cut**, replaced by the lint: the home is looked up in one place no test runs; changing `HOME` for the test run also changes it for cargo, rustup and build scripts |
| Lints on home lookup and processes   | **kept**, narrowed (section 6)                                                                                                             |
| Banner                               | **kept** (section 4)                                                                                                                       |
| Scenarios (one world in one file)    | **replaced** by case files (decision 1); the non-disk world is deferred (section 10)                                                                    |
| Change script ("a week passes")      | **deferred** with the world (section 10); the rescan's changes are already scripted by `FakeChangeJournal`                                   |
| Remaining fakes                      | most built by 008 and 009; the world's are deferred (section 10)                                                                                        |
| Large-tree generator                 | built by 004 (`LargeTree`): the one allowed exception to case files                                                                        |
| Developer CLI                        | **cut** to the generator and `npm run sandbox:test -- <feature>`; "scan a folder, explain a path, print a dry run" **deferred** with the panel: a case file and its test answer the same questions today |
| Debug panel                          | **deferred** (`docs/deferred/dev-debug-panel.md`, written by 05/01): its knobs — permissions, the clock, running apps — need the deferred world's fakes first, and the sandbox already gives a real window to look at |
| `dev-tools` cargo feature            | **cut** (decision 6): `cfg(debug_assertions)` stays the gate                                                                                                                            |

### 10. Later phase: the world outside the disk (deferred)

The same case file declares what is not files, served by the existing fakes in sandbox mode and in the autotests:

```toml
[world]
now = "2026-11-01T12:00:00Z"
running = [{ name = "Example Editor", bundle-ids = ["com.example.editor"] }]
open-files = ["home/Library/Caches/com.example.editor/db"]
access = { all-files = "granted" }
[[world.tool]]               # an answer of an external tool, by its exact arguments
program = "docker"
args = ["image", "ls", "--format", "json"]
stdout-file = "docker/images.json"
[[case.app]]                 # an installed app: a bundle with its Info.plist, written to disk by the generator
path = "Applications/Example Editor.app"
bundle-id = "com.example.editor"
version = "2026.3"
```

When a file declares `[world]`, sandbox mode uses `FakeClock`, `FakeRunningApps`, `FakeOpenFiles`, `FakeCommandRunner`
and `FakeAccess` with these answers instead of the real ones; a later change of the world ("a week passes", an app
quits) is a list of steps the autotest plays between two scans. Deferred (decision 5) to
[`../../deferred/sandbox-world.md`](../../deferred/sandbox-world.md), due when the first module that reads a non-disk
fact is planned.

### 11. Interfaces with other plans

| Plan | 007 needs from it                                                                 | 007 offers it                                                           |
| ---- | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| 005  | the data folder and the settings file; `~` in the scan roots                      | sandbox mode puts them in `<sandbox>/.fresher-data`                     |
| 008  | the marker, `Reach`, `FakeTrash`, the protection list, the core e2e's patterns    | `Reach::SandboxFolder`; the sandbox for its deferred run by hand        |
| 009  | `KnownFolders`, `FakeKnownFolders::home_layout`, the system fakes                 | the sandbox's known folders; the deferred world fills its fakes       |
| 011  | —                                                                                 | nothing in release; the banner's text is not a diagnostic               |
| 013  | —                                                                                 | the development identifier; the release check for `FRESHER_SANDBOX`    |
| 014  | —                                                                                 | the case format, if its sample data wants it (its own release-side code) |
| Module plans | —                                                                         | the case file, the autotest, `EXPECTED.md` (section 8)                  |

**Built only after plans 005 and 009 are merged into `main`**: sandbox mode places 005's data folder and 009's known
folders; the branch starts from `main` after both.

## Open questions

None left: the user answered all three on 2026-11-16 (decisions 4–6).

## Decisions

The user's, of 2026-11-16 (translated):

1. **Cases are declarative TOML files plus one generator**, not a script per feature. The generator and the autotests
   read the same files. A script is allowed only as an exception for what TOML cannot describe (for example the huge
   trees of the benches).
2. **In sandbox mode the Trash is a folder inside the sandbox**; nothing reaches the real Trash. A separate explicit
   development flag (`--real-trash`) may use the real Trash to check the system's behaviour.
3. **The plan is expanded now and shown to the user; it is built after approval.**

The user's answers of 2026-11-16, approving the plan (translated):

4. **The generator tops up and never removes** (option a of the first open question): on a sandbox it made before,
   it writes what is missing, resets its own files and lists strangers; a clean start is the developer's own
   `rm -rf`. The workspace keeps one deletion point.
5. **The world outside the disk comes later** (option a of the second): the former plan 06 moves to
   [`../../deferred/sandbox-world.md`](../../deferred/sandbox-world.md), due when the first module that reads apps,
   processes, tools or the clock is planned; its design stays in section 10.
6. **`cfg(debug_assertions)` keeps dev code out of release builds, as today** (option a of the third), with a
   release-only test and a binary check in plan [013](../013-release/readme.md); no `dev-tools` cargo feature.
7. **The plan is approved, with every agent's call below accepted** — explicitly call 2 (the sandbox's Trash is
   `home/.Trash`, the sandbox's `KnownFolder::Trash`) and call 6 (sandbox mode entered through environment variables
   set by the npm script). Building waits until plans 005 and 009 are merged into `main`.

Carried from plan 008 (the user's decision of 2026-11-16): 008 took from this plan only the guards it could not be
tested without (the marker, `Reach`, the fakes of the cleaner); this plan is no longer a precondition of 008.

## Agent's calls (accepted by the user on 2026-11-16)

1. **One case file per feature**, the feature a module id or a core feature; expectations per case, not per file.
2. **The sandbox's Trash is `home/.Trash`** — the sandbox's `KnownFolder::Trash`, where macOS keeps the user's Trash —
   rather than `<sandbox>/.Trash`, so a module that measures the Trash sees what the sandbox's cleanups put there.
3. **The default sandbox location is `sandbox/` at the repository's root**, gitignored. When the repository lies in a
   folder macOS guards (Desktop, Documents), the system may ask once for access; `--into` puts it elsewhere.
4. **`Reach::SandboxFolder`** for sandbox mode, rather than markers in the fake home (a marker never counts in a
   home or a hard-blocked folder) or a weaker protection list.
5. **Both protection lists apply in sandbox mode**: the real Mac's and the sandbox's.
6. **Sandbox mode by environment variables**, set by `npm run dev:sandbox`; the app reads them in its setup, in debug
   builds only. Arguments through `tauri dev` need two `--` separators and are easy to get wrong.
7. **Only known folders, data folder, Trash, reach and protection change in sandbox mode**; volumes, open files,
   running apps, commands, clock and access stay real (read-only) until a case file declares a world (section 10, deferred).
8. **The autotests go through the app's own sandbox composition** (`app/tauri/tests/sandboxes.rs`, a debug-only
   `sandbox_engine`) rather than a second composition in `crates/e2e`, so the tests and the window run the same code.
9. **`age` and `fill = "data"`** added to the tree vocabulary for every fixture: ages relative to a "now" keep a stale
   case stale; real bytes (capped) let a run measure the freed space.
10. **Golden report per case file**: `fixtures/golden/sandbox-<feature>.json`.
11. **The first case files**, both on the `trial` module: `trial.toml` (a cache to the Trash, a folder emptied and
    kept, a look-alike, a folder without a marker next to it) and `cleanup-guards.toml` (trial folders inside the
    sandbox's `.Trash` and inside its `Documents`, behind a link out of the sandbox, and a hard link from a negative's
    file into a positive) — the disk parts of plan 008's run by hand. A change between the scan and the cleanup (an
    item replaced) is not a static case: it stays in the core e2e.
12. **The rule for later plans** (section 8) goes into `docs/rules.md` rule 8 and `docs/plans/README.md`.
13. **No new third-party dependency.**
14. **No version bump**: the version stays 0.1.0 (the user's decision of 2026-11-16, [`../README.md`](../README.md), "Versions").

## Conclusions

Not written yet.
