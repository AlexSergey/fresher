# 031 — Xcode and simulators: design

**Status:** ✅ approved (the user's delegation of 2026-11-22) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-22 from the one-file draft of this plan and a read of the code on `main` (97d9b314), read-only.
Approved by the user's delegation of 2026-11-22: build autonomously and take the recommended decisions; the user checks
everything later. Where a fact found on the developer's Mac forced a different choice than the draft, the design says
so and the choice is an agent's call (call 1 changes the draft the most). Facts marked "to verify" are checked by hand
in 03/02.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Every cleaner aimed at developers handles Xcode's data; Fresher offers only DerivedData (plan
[018](../018-developer-caches/readme.md), module `dev-caches`). Archives, device support files, the previews' cache and
the simulators' caches are often the largest part, and [`goal.md`](../../goal.md), section 4, names them.

## Goals

1. A module `xcode` (`modules/xcode`, crate `fresher-module-xcode`, group **Development**) that offers Xcode's data in
   four categories, each with its own danger level; never one "Xcode junk" bucket.
2. Each item names what it is (an OS version and build, an archive's name and date) and the size it frees; every
   finding is counted in Xcode's line of the storage map (`NewFinding::for_app(com.apple.dt.Xcode)`).
3. Nothing is offered while Xcode or the Simulator runs: each candidate is left out `app-open` with a sentence.
4. A folder the module does not recognise (its name's shape, its sign inside) is not claimed (rule 1).
5. No change to the core, the scanner, the cleaner, the engine or the sandbox crate: the module is a table of places,
   a pure decision and texts, in the shape of `app-caches` and `dev-caches`.

## Non-goals

- **Simulator devices and runtimes** (the draft's `xcode.unavailable-devices`): deferred, with the facts found, to
  [`../../deferred/xcode-simulator-devices.md`](../../deferred/xcode-simulator-devices.md) (agent's call 1). With them
  goes the sandbox's tool world: plan [032](../032-docker/readme.md) builds it (its plan 01, "only if plan 031 has not
  built it").
- Xcode itself, its SDKs, the command line tools, the simulator runtimes and the system's
  `/Library/Developer/CoreSimulator` (system-managed disk images).
- DerivedData: it stays `dev-caches.xcode-derived-data` (the draft's non-goal).
- Archives kept elsewhere than `~/Library/Developer/Xcode/Archives` (Xcode lets the user choose another folder): not
  read.
- `~/Library/Caches/com.apple.dt.Xcode` and the other `com.apple.` caches: Apple's (`app-caches` excludes them too).
- Xcode's own settings (`~/Library/Developer/Xcode/UserData` outside `Previews`): never claimed.
- A size floor, an age threshold or a setting of the module's own (the core's `<module>.enabled` switch exists).

## Facts found on the developer's Mac (2026-11-22, read-only)

| Fact                                                                                                         | Found                                                                                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Xcode installed                                                                                              | `/Applications/Xcode.app`, 27.2 (27B52); `xcode-select -p` is its `Contents/Developer`                                                                                                                                     |
| `~/Library/Developer/`                                                                                       | `CoreSimulator`, `DVTDownloads`, `Xcode`, `XCTestDevices`                                                                                                                                                                  |
| `~/Library/Developer/Xcode/`                                                                                 | `SDKToSimulatorIndexMapping.plist`, `UserData/` (`IDEPreferencesController.xcuserstate`, `KeyBindings`, `XcodeCloud`); no `DerivedData`, `Archives`, `iOS DeviceSupport` or `UserData/Previews` on this Mac (to verify on a Mac that has them) |
| `~/Library/Developer/CoreSimulator/Caches`                                                                   | 3.4 GB: `dyld/<macOS build>/<runtime id>.<runtime build>/dyld_sim_shared_cache_arm64*`; the build folder `24D70` is an older macOS build than the running one (26B131): the simulator rebuilds this cache per host build |
| `~/Library/Developer/CoreSimulator/Devices`                                                                  | 2.1 GB: one folder per device (`<device id>/device.plist`, `data/`) and `device_set.plist`; every device names the runtime `iOS-18-0`, which is not installed (`/Library/Developer/CoreSimulator/Volumes` is empty)            |
| `xcrun simctl list -j devices`                                                                               | **did not answer in 120 s.** `simctl` in Xcode 27.2 is a shell script that runs `xcodebuild -runFirstLaunch` (an installer of system components) when the CoreSimulator framework is older than the one it expects (1048 here, 1174.21 expected), and then the framework's own `simctl`. The run started `xcodebuild -runFirstLaunch`; it was stopped by hand at once and the framework's version was unchanged (nothing installed) |

The last row decides agent's call 1: listing devices through `xcrun simctl` can start a system installer, and the
answer of an older framework may misreport a device's availability. The paths of the categories this plan does build
are marked "to verify" where this Mac does not have them; a wrong path or sign claims nothing (the safe direction).

## What already exists

| Need                                        | What exists                                                                                                    | Plan     |
| ------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | -------- |
| A place under the user's Library            | `Pattern::Under(KnownFolder::Library, "Developer/…")`; two `*` in one path (`local-ai-models`' `.lmstudio/models/*/*`) | 002, 023 |
| Only some names at the last `*`             | `Interest::only_suffixes(vec![".xcarchive"])`                                                                  | 020      |
| A sign inside the matched folder            | `Interest::holding(names)`, `Match.holds`                                                                      | 018      |
| A folder summed with everything inside      | `Claim::Whole`, `Match.sizes`, `Match.inside` (`special`, `foreign`, `complete`)                               | 004, 017 |
| What the walk saw of a place                | `cx.place_seen(interest)` (only for a path with no `*` before its last segment)                                 | 019      |
| Running apps                                | `cx.system().running_apps()`, `RunningApp.bundle_ids`                                                          | 009      |
| A finding in its app's line of the map      | `NewFinding::for_app(id)`                                                                                      | 012      |
| Left-outs                                   | `out.left_out(LeftOutItem { … })`, `with_problems(keys)`                                                       | 011      |
| Moving a folder to the Trash; emptying one  | `Removal::Trash`, `Removal::EmptyFolder` (the cleaner's checks, the journal, put back)                          | 008, 030 |
| Nothing pre-selected; only Safe in bulk     | `app/ui/src/state/defaults.ts` (`PRESELECTED` is nothing; `selectByLevel` refuses all but `safe`)               | 027      |
| A sandbox world with an open app            | `[[case.running]]` (`bundle-ids`), `[[case.symlink]]`, `kind = "socket"`                                        | 016, 017 |

**Nothing in the core is missing.** The draft's "archives never pre-selected" holds already: nothing is pre-selected,
and only Safe categories can be selected in one go.

## Design

### 1. Rules or code

Code, as `dev-caches` and `app-caches`: a rule cannot keep a folder while an app runs, report a left-out, or tell the
newest versions of a platform from the older ones in its reason ([`../../architecture/modules.md`](../../architecture/modules.md),
"Rules or code"; agent's call 3). The places live in a table (rule 5).

### 2. The places (`places.toml`)

One row per place, read with `deny_unknown_fields`, kebab-case keys:

| Row id                    | Category                 | `path` (under `library`)                     | `shape`          | `platform` | `sign`              | `removal`      |
| ------------------------- | ------------------------ | -------------------------------------------- | ---------------- | ---------- | ------------------- | -------------- |
| `ios-device-support`      | `xcode.device-support`   | `Developer/Xcode/iOS DeviceSupport/*`        | `device-support` | `iOS`      | `Symbols`           | `trash`        |
| `watchos-device-support`  | `xcode.device-support`   | `Developer/Xcode/watchOS DeviceSupport/*`    | `device-support` | `watchOS`  | `Symbols`           | `trash`        |
| `tvos-device-support`     | `xcode.device-support`   | `Developer/Xcode/tvOS DeviceSupport/*`       | `device-support` | `tvOS`     | `Symbols`           | `trash`        |
| `visionos-device-support` | `xcode.device-support`   | `Developer/Xcode/visionOS DeviceSupport/*`   | `device-support` | `visionOS` | `Symbols`           | `trash`        |
| `previews`                | `xcode.previews`         | `Developer/Xcode/UserData/Previews`          | `fixed`          | —          | `Simulator Devices` | `empty-folder` |
| `simulator-caches`        | `xcode.simulator-caches` | `Developer/CoreSimulator/Caches`             | `fixed`          | —          | `dyld`              | `empty-folder` |
| `archives`                | `xcode.archives`         | `Developer/Xcode/Archives/*/*`               | `archive`        | —          | `Info.plist`        | `trash`        |

- `folder` is `library` on every row today and stays a column (rule 5).
- Interests, in the table's order: one `Under(folder, path)` `Claim::Whole` `.holding(sign)` per row; the `archive`
  row adds `.only_suffixes(vec![".xcarchive"])`.
- Refused, one error variant each: not valid TOML or an unknown key (`Format`); no row (`Empty`); an id not kebab case
  (`BadId`) or twice (`DuplicateId`); a category not the module's (`UnknownCategory`) or a module category no row uses
  (`CategoryWithoutRow`); a walked place as `folder` (`WalkedPlace`); a `path` that is not plain names and `*`, or a
  `*` that is part of a name (`BadPath`); a `device-support` or `archive` row whose path does not end in `*`, or a
  `fixed` row whose path holds a `*` (`ShapeAndPath`); a `platform` on a row that is not `device-support`, or none on
  one that is (`Platform`); an empty `sign` (`NoSign`); two rows of one `(folder, path)` (`DuplicatePlace`).
- The paths and signs of the device support folders, the previews and the archives are "to verify" (section "Facts").

### 3. The names (`name.rs`, pure functions)

- **Device support** (`shape = "device-support"`): the folder's name is `[<model> ]<version> (<build>)`: `<version>`
  is two or three dot-separated numbers (`17.2`, `17.2.1`), `<build>` one or more ASCII letters and digits (`21C62`),
  `<model>` an optional first word of letters, digits and commas (`iPhone15,2`, `Watch6,1`) followed by one space.
  `DeviceSupportName { model: Option<String>, version: Version, build: String }`; `Version` compares by its numbers
  (`17.10` above `17.9`). Anything else (`Notes`, `17 (21A1)`, `17.2(21C62)`, `17.2 ()`, a name not in UTF-8) is no
  name: not claimed.
- **Archive** (`shape = "archive"`): the parent folder's name is a date `YYYY-MM-DD` (four, two and two digits; month
  01-12, day 01-31) and the folder's own name ends with `.xcarchive` and has something before it. `ArchiveName {
  name: String (without the suffix), date: String (the parent's name) }`. Otherwise not claimed.
- **Fixed** rows match one folder; the name is not judged.

### 4. The decision (`decide.rs`, a pure function)

Matches are gathered in the scan (keyed by its cancel token, as `app-caches`' `Gathered`) and decided in `finish`, in
path order, with the running apps asked at most once, only when a candidate reaches row 8. For one match, the first
row that applies:

| #   | When                                                                                      | Verdict              |
| --- | ----------------------------------------------------------------------------------------- | -------------------- |
| 1   | not a folder (a file, a link, a socket)                                                   | not claimed          |
| 2   | the name does not read as its row's shape (section 3)                                     | not claimed          |
| 3   | the row's sign is not inside (`holds` misses a name of `sign`)                            | not claimed          |
| 4   | not read in full (`inside` missing or not `complete`)                                     | not claimed          |
| 5   | nothing in it (logical size 0)                                                            | not claimed          |
| 6   | a socket, a pipe or a device inside                                                       | left out `special`   |
| 7   | not the user's, a system flag, restricted, the user's lock, or such an entry inside       | left out `not-yours` |
| 8   | the running apps cannot be listed                                                         | left out `no-system` |
| 9   | Xcode (`com.apple.dt.Xcode`) or the Simulator (`com.apple.iphonesimulator`) runs           | left out `app-open`  |
| 10  | otherwise                                                                                 | offered              |

- The two bundle ids are a constant of the module (`XCODE_APPS`), one list for every row (agent's call 5).
- Nothing is judged by time, so the verdict does not depend on the order of matches, and a restored scan decides the
  same from the same open apps (the open apps are asked again at launch).
- **The newest versions of a platform** (device support only): after the verdicts, among the offered device support
  folders of one `platform` plus the left-out ones of it, the two highest distinct versions are "recent". A recent
  folder is still offered, at the category's level, with the reason `xcode.reason.device-support-recent` instead of
  `xcode.reason.device-support`. The level never changes (the draft).
- At the act, the cleaner's own checks run (check 10: an open file of the item, a running app owning it by bundle id).

### 5. Findings

| Category                 | Danger | Removal       | Identity                         | Title                                                         | Reason                                                     |
| ------------------------ | ------ | ------------- | -------------------------------- | ------------------------------------------------------------- | ---------------------------------------------------------- |
| `xcode.device-support`   | Low    | `Trash`       | `<row id>/<folder name>`         | `xcode.title.device-support` (`platform`, `version`, `build`) | `xcode.reason.device-support` or `-recent` (`platform`, `version`, `size`) |
| `xcode.previews`         | Safe   | `EmptyFolder` | `previews`                       | `xcode.title.previews`                                        | `xcode.reason.previews` (`size`)                           |
| `xcode.simulator-caches` | Safe   | `EmptyFolder` | `simulator-caches`               | `xcode.title.simulator-caches`                                | `xcode.reason.simulator-caches` (`size`)                   |
| `xcode.archives`         | High   | `Trash`       | `archives/<date>/<folder name>`  | `xcode.title.archive` (`name`)                                | `xcode.reason.archive` (`date`, `size`)                    |

- Subject `Subject::Path` of the folder; the removal names the same path; sizes the match's (`logical`, `allocated`,
  `reclaimable`); `size` in a text is the logical size (`TextArg::Size`), the others `TextArg::Text`.
- Every finding `for_app(com.apple.dt.Xcode)`: the map counts it in Xcode's line (an Xcode no longer installed is
  shown as an app no longer installed, which is true).
- Left-outs: `LeftOutItem { category, reason: xcode.left-out.<reason>, sizes }` for rows 6 to 9.

| Category                 | Why this level                                                                                                   |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `xcode.device-support`   | Low: Xcode copies the symbols again the next time a device on that version is connected, which takes minutes     |
| `xcode.previews`         | Safe: Xcode rebuilds the previews when a preview is opened                                                       |
| `xcode.simulator-caches` | Safe: the simulator rebuilds its shared cache the next time a simulator starts (verified: one folder per macOS build) |
| `xcode.archives`         | High: an archive holds the build and the debug symbols of a version that may be shipped; without them, crash reports of that version cannot be read, and nothing makes them again |

### 6. Texts (`locales/en.json`)

Plain English (rule 7), polished when built:

- `xcode.device-support.name` "Device support files"; `.what` "Debug information Xcode copies from an iPhone, iPad,
  Apple Watch, Apple TV or Vision Pro the first time you connect it, one folder per system version."; `.made-by`
  "Xcode."; `.if-removed` "The folder goes to the Trash. You can still build and run apps; debugging on a device of that
  version waits until Xcode copies the files again."; `.comes-back` "Yes, when you connect a device on that version
  again. Copying takes a few minutes."
- `xcode.previews.name` "Xcode previews"; `.what` "What Xcode builds to show your SwiftUI previews."; `.made-by`
  "Xcode."; `.if-removed` "What is inside the folder goes to the Trash and the folder stays."; `.comes-back` "Yes, the
  next time you open a preview. The first one takes longer."
- `xcode.simulator-caches.name` "Simulator caches"; `.what` "Files the iOS simulators keep to start faster, one set for
  each version of macOS you ran them on."; `.made-by` "The Simulator, part of Xcode."; `.if-removed` "What is inside the
  folder goes to the Trash and the folder stays. Your simulators and their apps stay."; `.comes-back` "Yes, the next
  time a simulator starts. That start takes longer."
- `xcode.archives.name` "Xcode archives"; `.what` "The builds you archived in Xcode to publish an app, each with the
  debug symbols of that version."; `.made-by` "Xcode, when you choose Product, then Archive."; `.if-removed` "The
  archive goes to the Trash. Without it you cannot read the crash reports of that version or publish that build
  again."; `.comes-back` "No. Keep the archives of every version people still use."
- `xcode.enabled.label` "Xcode's files"; `.description` "Finds device support files, previews, simulator caches and
  archives that Xcode keeps, while Xcode and the Simulator are closed. They go to the Trash."
- `xcode.title.device-support` "{platform} {version} ({build})"; `xcode.title.previews` "Xcode previews";
  `xcode.title.simulator-caches` "Simulator caches"; `xcode.title.archive` "{name}".
- `xcode.reason.device-support` "Debug files for {platform} {version} take {size}. Xcode copies them again if you
  connect a device on this version."; `xcode.reason.device-support-recent` "Debug files for {platform} {version} take
  {size}. This is one of the newest versions here, so a device you use may need them; Xcode copies them again when you
  connect it."; `xcode.reason.previews` "Xcode's previews take {size}. Xcode builds them again when you open a
  preview."; `xcode.reason.simulator-caches` "The simulators' caches take {size}. They are made again the next time a
  simulator starts."; `xcode.reason.archive` "Archived on {date}, {size}. It holds the debug symbols of that build:
  keep it if that version is still in use."
- Left-outs: `xcode.left-out.special` "It holds a connection a running program may be using, so it is kept.";
  `.not-yours` (the `app-caches` sentence); `.no-system` "Fresher could not list your open apps, so it keeps this. Scan
  again."; `.app-open` "Xcode or the Simulator is open and may be using this. Quit them and scan again."

### 7. Who else claims `~/Library/Developer`

`dev-caches` claims `Developer/Xcode/DerivedData/*`; no other module has an interest under `Library/Developer`
(checked with a search of every module's tables and code). The places of this module are apart from it; the core e2e
registers both and asserts no path is offered by both (section 8).

### 8. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Unit**: `places.rs` (every refusal, then the shipped table pinned), `name.rs` (device support and archive names,
  each refusal first, then the forms; `Version` order), `decide.rs` (one test per row, keep before offer; the recent
  versions; property: a candidate stopped before row 8 never asks the running apps), `tests.rs` (the module on
  hand-built matches; the properties that the verdict never depends on the order of matches and that a restored
  context decides the same).
- **Fixtures** (`fixtures/homes/xcode-*.toml`, `run_module`, golden `fixtures/golden/xcode.json`): `-not-claimed`,
  `-kept`, `-offered` (step 01/03).
- **Contract**: `assert_module_contract` with `Catalogue::Required`: four category files with the same dangers, every
  text and every declared left-out used by a fixture run, every removal a `Trash` or `EmptyFolder` of the matched path.
- **Sandbox**: `fixtures/sandboxes/xcode.toml` and `xcode-open.toml` with their goldens; `EXPECTED.md` walked by hand.
- **Core e2e**: `crates/e2e/tests/xcode.rs` beside `dev-caches`.
- **UI e2e**: the Development card and Details, the review naming High for an archive, a cleanup with an archive in
  the Trash and the simulator caches emptied, the switch in Settings, one visual baseline.

## Rebase onto plans 033, 034 and 037

Those plans merge into `main` before this one is built; the branch is rebased onto them. To keep it mechanical:

- This plan's own files are new paths (`modules/xcode/`, `fixtures/*/xcode*`, `crates/e2e/tests/xcode*`,
  `app/ui/src/test/xcode-scan.ts`, `docs/categories/xcode-*.md`, `docs/deferred/xcode-simulator-devices.md`).
- Shared lists get one appended line each: `modules()` and the three expected-id lists (the module after the last
  feature module), `case_files!` (sorted), `crates/e2e/Cargo.toml`, `app/tauri/Cargo.toml`, the catalogue's and the
  deferred README's tables, `docs/testing.md`'s and `docs/manual-testing.md`'s lists. A conflict there is resolved by
  keeping both lines.
- The sandbox goldens (`fixtures/golden/sandbox-xcode*.json`) are never merged by hand: after the rebase they are
  regenerated with `UPDATE_GOLDEN=1 npm run sandbox:test` (plan 037 adds a `trash` section to every sandbox golden)
  and the diff is reviewed. `Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are regenerated, never
  merged.
- `docs/deferred/user-review-2026-11-22.md` may already exist from those plans: this plan appends its section.

## Risks

- **A path or a sign is wrong on a newer Xcode** (this Mac has no device support, previews or archives): the folder
  is not claimed and its space stays in the storage map. 03/02 checks them on a Mac that has them.
- **An archive of a version still in use is removed**: High, never selected in bulk, the reason says what it holds,
  and it goes to the Trash. Fresher cannot know which versions are shipped.
- **Xcode opened between the scan and the cleanup**: the cleaner's check 10 sees an open file of the item or a running
  app owning it; otherwise an archive or a device support folder goes while Xcode runs. Xcode copes with a missing
  device support folder (it copies again); an archive it is not showing is only a folder.
- **The simulators' service (`CoreSimulatorService`) runs without the Simulator app** (it is a background service, not
  an app in the running list): a booted simulator maps its shared cache, and the cleaner's open-files check skips an
  entry held open; a cache emptied under a simulator that is not running is rebuilt on its next start.
- **Deferring simulator devices leaves the largest Xcode item out** (2.1 GB of unavailable devices on this Mac): the
  deferred item says what doing it needs; the space stays in the storage map.

## Open questions

None.

## Decisions

- The user's delegation of 2026-11-22 (translated): "Restore the last session's context and start implementing; work
  autonomously with subagents, take the recommended decisions; implement all the features; write into docs/deferred
  what I will need to look at — I will check everything tomorrow." This design is approved by it.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Each with what it costs if wrong.

1. **Simulator devices are deferred, and with them the sandbox's tool world** (changes the draft's calls 2 and 4).
   On this Mac `xcrun simctl list -j devices` started `xcodebuild -runFirstLaunch`, an installer of system components,
   because the CoreSimulator framework was older than the one Xcode expects; the app would run that installer on a
   scan, against "the app never runs as administrator" and rule 1. A framework older than Xcode's may also misreport
   which devices are available, so a working device could be offered for deletion. And `xcrun` on a Mac without the
   developer tools opens the system's install dialog. Doing it safely needs a way to ask simctl that can never start an
   installer or a dialog, which no command found does. Cost if wrong: unavailable devices (often gigabytes) wait for a
   later plan; their space stays in the storage map. [`xcode-simulator-devices`](../../deferred/xcode-simulator-devices.md).
2. **The archives stay at High and are never pre-selected**, with no code of this plan: nothing is pre-selected and
   only Safe can be selected in bulk. Cost if wrong: none.
3. **Code, not rules** (section 1). Cost if wrong: none; a later rule engine feature could take the table.
4. **Device support folders of the two newest versions of a platform are offered at Low with their own reason**, not
   kept (the draft). Cost if wrong: a device support folder in use is copied again on the next connect (minutes).
5. **One list of apps for every row**: Xcode or the Simulator open keeps every candidate, even an archive while only
   the Simulator runs. Cost if wrong: an item waits until both are closed.
6. **The previews' and the simulators' caches are emptied, not moved** (`EmptyFolder`, the draft): Xcode and the
   simulators expect the folders. Cost if wrong: one Trash entry per top-level item.
7. **The sign of each place** (`Symbols`, `Simulator Devices`, `dyld`, `Info.plist`) and the device support name
   shape with an optional model prefix: verified for the simulators' caches only, the rest to verify. Cost if wrong: a
   folder not claimed (the safe direction).
8. **Every finding in Xcode's line of the map** (`for_app(com.apple.dt.Xcode)`), the Simulator's caches included (the
   Simulator ships inside Xcode). Cost if wrong: a line of the map.
9. **No scan bench**: the walk reads the same folders; the new interests add branches to the matcher at four places.
   No UI hot path changes, so no performance run. Cost if wrong: a slower scan noticed in plan 041's run.
10. **The module's place in the registration is after the last feature module**, so the rebase onto 033 and 034 adds
    a line beside theirs. Cost if wrong: none (the order decides only a claim two modules make, and there is none).
11. **No new dependency** (`serde`, `thiserror`, `toml`; `proptest`, `serde_json` as dev-dependencies) **and no new UI
    component**. Cost if wrong: none.

## Conclusions

Not written yet.
