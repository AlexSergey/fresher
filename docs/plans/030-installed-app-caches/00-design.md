# 030 — Installed app caches: design

**Status:** ✅ approved (the user's choice of 2026-11-21) · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-21 from the idea [Caches of installed apps](../../ideas/installed-app-caches.md) and from a read of
the code on plan 024's design commit (39ec314f), read-only. Approved by the user's choice of 2026-11-21: after plans
024 and 029, build the caches of installed apps autonomously, take the recommended decision on every question and
record each as an agent's call; the user checks everything later. Where the code or the rules forced a different
choice than the idea, the design says so and the choice is an agent's call. Facts marked "to verify" are checked by
hand in 03/02.

## Intent

Apps the user still uses keep caches nobody empties: chat and music apps, editors, browsers, updaters. Each is often
hundreds of megabytes; together, gigabytes. Plan 019's `app-leftovers` offers the caches of apps that are gone; plan
018's `dev-caches` offers package managers' and build caches; the caches of installed, everyday apps are offered by
no module. The idea asks for them, offered only while their app is closed, emptied rather than removed.

## Goals

1. A module `app-caches` (`modules/app-caches`, crate `fresher-module-app-caches`, group System) that offers, one
   finding per folder, the folders `~/Library/Caches/<bundle id>` that **an installed app provably owns**: its own
   bundle id or one of its helpers', or an id under one of them (`com.example.Editor.ShipIt`).
2. Each finding names the app (its shown name) and the size it frees; it is counted in the app's line of the storage
   map (`NewFinding::for_app`).
3. A cache is offered only while its app and its helpers are closed; nothing is claimed whose owner cannot be told
   by bundle id, nothing of Apple's (`com.apple.`) and nothing of Fresher's own.
4. Removal **empties** the folder into the Trash and keeps the folder (`Removal::EmptyFolder`, plan 008), through the
   cleaner's existing checks; no new removal kind, no core change.
5. A path is claimed by one module only: `app-caches` claims exactly the folders `app-leftovers` leaves unclaimed
   because an installed app owns them (section 3).

## Non-goals

Each is postponed, with what it needs, in
[`../../deferred/installed-app-caches-more-places.md`](../../deferred/installed-app-caches-more-places.md):

- **The generic finder of web-engine caches** (`~/Library/Application Support/<name>/{Cache,Code Cache,GPUCache,…}`):
  those folders are named after the app's name (`Slack`, `discord`, `Code`), not its bundle id, and the rules identify
  an app "by its bundle id and team id only, never by a loose name" ([`../../architecture/modules.md`](../../architecture/modules.md),
  "The system's facts"). It needs a table that ties each folder name to a bundle id (rules as data), checked app by app
  (agent's call 1).
- **Browsers' caches**: Chrome, Firefox, Edge, Brave, Vivaldi, Arc keep them under names that are not ids
  (`Caches/Google/Chrome/<profile>/Cache`, `Caches/Firefox/Profiles/<profile>/cache2`); the same table is needed, with
  the profile layout of each. Safari's is Apple's. A browser whose cache folder is its id (`Caches/com.operasoftware.Opera`)
  is offered by this plan like any app.
- **Game launchers' download and web caches** (Steam, Epic Games, Battle.net): inside their support folders, the same
  table.
- **The Metal shader cache**: it lives in the user's cache folder under `/private/var/folders/<xx>/<hash>/C/`, which the
  protection list refuses (only `T` is carved out, [`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)),
  and it is Apple's (`com.apple.metal`). It needs a deletion-safety decision of its own.
- **Sandboxed apps' caches** (`~/Library/Containers/<id>/Data/Library/Caches`): the containers are not read
  ([`../../deferred/app-leftovers-containers.md`](../../deferred/app-leftovers-containers.md)).
- **Apple's own apps' caches** (`Caches/com.apple.*`): many are written by the system's services, not by an app the
  user opens; never claimed here.
- An age threshold or a setting: every cache of a closed installed app is offered, whatever its age (agent's call 7).
- A store moved by an environment variable: there is none for `~/Library/Caches`; the known folder is resolved by the
  `KnownFolders` port.

## What already exists

| Need                                        | What exists                                                                                                     | Plan     |
| ------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| The place                                   | `Pattern::Under(KnownFolder::AppCaches, "*")`                                                                    | 002, 009 |
| A `*` that matches only bundle-id names     | `Interest::only_bundle_ids()`; `Caches/JetBrains`, `Caches/Homebrew`, `Caches/Yarn` stay open to their modules    | 019      |
| A folder summed with everything inside      | `Claim::Whole`, `Match.sizes`, `Match.inside` (`special`, `foreign`, `complete`)                                 | 004, 017 |
| Two modules matching one folder             | the walk reports a folder to every interest that ends at it, each module with its own index (the engine's test `a_disabled_modules_match_reaches_no_module` matches two modules' identical interests) | 002 |
| One owner per path among findings           | `claims::resolve` (higher danger, then the module registered first): a guard, not the design (section 3)         | 012      |
| Installed apps and their helpers            | `cx.system().installed_apps()`, `InstalledApp { bundle_id, name, helpers, … }`, `InstalledApp::owns`              | 009, 016 |
| A name named after an id                    | `BundleId::names(name)`: the whole name, or the id followed by a dot, ignoring case                             | 009      |
| Running apps                                | `cx.system().running_apps()`, `RunningSet::owner_of(path)`, `RunningApp.bundle_ids` (a helper's and its app's)   | 009      |
| A finding in its app's line of the map      | `NewFinding::for_app(id)`                                                                                         | 012      |
| Left-outs                                   | `out.left_out(LeftOutItem { … })`, `with_problems(keys)`                                                          | 011      |
| Emptying a folder, keeping it               | `Removal::EmptyFolder(path)` → `Action::Empty`; the cleaner's `remove/empty.rs` (each entry checked and moved on its own), `ItemOutcome::Emptied { moved, skipped }`, the journal and put back, `core.cleanup.outcome.emptied` (shown in the cleanup's detail row; the result summary counts `moved + emptied` as moved to the Trash), the sandbox's `after-cleanup = "emptied"` | 008 |
| An exclusion table                          | `modules/app-leftovers/exclude.toml` (`com.apple.` required): the shape to follow; a module never depends on another, so the new module has its own | 019 |
| A sandbox world with installed and open apps | `[[case.app]]`, `[[case.running]]` in a case file                                                                 | 016      |

**Nothing in the core is missing.** `Removal::EmptyFolder` has been built end to end since plan 008 but only the
dev-only `trial` and `demo` modules and the rules use it: `app-caches` is its first real module, so the core e2e and
the UI cases prove the emptying, the journal and put back on it (section 7).

## Design

### 1. The places (`places.toml`) and the exclusions (`exclude.toml`)

One row, as data (rule 5), so a later place is a row:

| Row id           | Category                     | Known folder | `path` | Name     |
| ---------------- | ---------------------------- | ------------ | ------ | -------- |
| `library-caches` | `app-caches.library-caches`  | `app-caches` | `*`    | `<id>`   |

- The table is read with `deny_unknown_fields` and refused, one error each, for: no row; an id twice or not kebab
  case; a category that is not the module's, or a module category no row uses; a known folder that is a walked place
  (`user-temp`, `shared-temp`); a `path` that is not plain names followed by `*`; two rows of one `(folder, path)`.
- Interests, in the table's order: one `Under(folder, path)` `Claim::Whole` `.only_bundle_ids()` per row. It is the
  same interest as `app-leftovers`' `caches` row: the walk reports each id-named folder of `Caches` to both modules,
  and each decides; the trie's id branch at `Caches` exists already, so the walk does no new work there.

`exclude.toml`: ids never claimed, each with why:

```toml
[[exclude]]
prefix = "com.apple."                 # every id that starts with it
why = "Apple's own apps and the system's services"

[[exclude]]
id = "io.github.alexsergey.fresher"   # the id and every id under it (`.dev`), by `BundleId::names`
why = "Fresher's own files"
```

Refused when not valid TOML, empty, a `prefix` that is not an id shape ending with `.`, an `id` that is not a bundle
id, an entry twice, or without `com.apple.` (so it cannot be dropped by mistake). An entry only ever keeps; entries for
apps whose `Caches` folder holds what does not come back are added from 03/02's run (to verify).

### 2. The owner (`owner.rs`, a pure function)

For a folder name `n` and the installed apps (`installed_apps()`, partial or not):

- `n` gives an id by `BundleId::new` with at least one dot (`com.example.App`); otherwise no owner.
- An app **names** `n` when its bundle id or one of its helpers' names it (`BundleId::names(n)`: `n` is the id, or the
  id followed by a dot and more).
- The owner is the app whose naming id is the longest (the most specific: `com.example.Editor.Pro` over
  `com.example.Editor` for `Caches/com.example.Editor.Pro`); apps with the same naming id (two copies of one app)
  are one owner, the first by path. No app names `n`: no owner.
- The finding's **app** is the owner's bundle id (its helper's folder counts in the app's line); **own** when `n` is
  the owner's bundle id itself (ignoring case), else a **part** (a helper's folder, or an id under the app's).
- Never by name, never the other way round: a folder `com.example` under which apps' ids lie names no app (a vendor's
  folder for several apps may be anyone's).

### 3. Who claims a `Caches/<id>` folder: `app-caches` or `app-leftovers`

| The folder's id                                                       | `app-leftovers` (plan 019, rows 1, 2, 4)  | `app-caches` (this plan)            |
| --------------------------------------------------------------------- | ----------------------------------------- | ----------------------------------- |
| excluded (`com.apple.`)                                                | not claimed                               | not claimed                         |
| an installed app's or helper's id, or an id under one                  | not claimed (row 4)                       | **decides** (section 4)              |
| the prefix of an installed app's id (`com.example`)                    | not claimed (row 4, "the other way round") | not claimed (no owner)              |
| no installed app relates to it                                         | **decides** (rows 2 to 15)                | not claimed (no owner)              |

`app-caches` claims only what `app-leftovers`' row 4 does not claim, from the same `installed_apps()` of the same scan
(one `SystemFacts` per scan), so no folder is ever claimed by both. The engine's `claims::resolve` would still count it
once if they ever did; the e2e asserts they never do (section 7). Fresher's own id is excluded by `app-caches`;
`app-leftovers` never offers it either (Fresher is installed, row 4, or open while it scans, row 7).

The other modules: `dev-caches` (`Caches/Yarn`, `Homebrew/downloads`, `CocoaPods`, `pip`, `go-build`) and
`jetbrains` (`Caches/JetBrains/*`) claim names that are not ids, which the shaped `*` never matches; `temp` works in
the temporary places; `downloads` in `Downloads`; `local-ai-models` and `stale-projects` outside `Library`. No overlap.

### 4. The decision (`decide.rs`, a pure function)

Matches are gathered in the scan (keyed by its cancel token, as `dev-caches`' `Gathered`) and decided in `finish`,
in path order, with the installed apps asked once and the running apps asked only when a candidate reaches row 9.
For one match, the first row that applies:

| #   | When                                                                                              | Verdict              |
| --- | ------------------------------------------------------------------------------------------------- | -------------------- |
| 1   | not a folder (a file, a link, a socket)                                                           | not claimed          |
| 2   | the name gives no id, or the id is excluded                                                       | not claimed          |
| 3   | the installed apps cannot be listed                                                               | not claimed          |
| 4   | no installed app names it (section 2)                                                             | not claimed          |
| 5   | not read in full (`inside` missing or not `complete`)                                             | not claimed          |
| 6   | nothing in it (logical size 0)                                                                    | not claimed          |
| 7   | a socket, a pipe or a device inside                                                               | left out `special`   |
| 8   | not the user's, a system flag, restricted, the user's lock, or such an entry inside               | left out `not-yours` |
| 9   | the running apps cannot be listed                                                                 | left out `no-system` |
| 10  | the owner runs: a running app has the owner's bundle id or a helper's, or `owner_of(path)` names one | left out `app-open`  |
| 11  | otherwise                                                                                         | offered              |

- Row 3 says nothing: `app-leftovers` already leaves out every id-named leftover `no-system` when the installed apps
  cannot be listed, so the user is told once (agent's call 9).
- Rows 5 to 8 mirror `dev-caches`' and `local-ai-models`' (one shape for the same questions). Nothing is judged by
  time, so the verdict does not depend on the order of matches and a restored scan decides the same from the same open
  apps; the open apps are asked again at launch, so a restored scan leaves out the folders of the apps open now
  (`app-open`) even when they were offered when the scan was made.
- `decide` takes the installed apps and a closure for the running apps, with two lifetimes (`'a` for the apps the
  verdict borrows, `'r` for the running set); `finish` memoises the running apps in a `OnceCell`, so they are asked at
  most once per scan and only at row 9.
- At the act, an app opened after the scan is caught only by the cleaner's check 10 when it holds a file of the folder
  open; otherwise its cache is emptied under it, which the macOS contract for `Caches` allows (the system empties
  these folders itself when space runs low). Risks, below.

### 5. Findings

| Part     | Value                                                                                                     |
| -------- | --------------------------------------------------------------------------------------------------------- |
| category | `app-caches.library-caches`, Low                                                                           |
| identity | `<row id>/<folder name>` (`library-caches/com.example.Editor`)                                             |
| subject  | `Subject::Path` of the folder; removal `Removal::EmptyFolder` of the same path                             |
| sizes    | the match's (`logical`, `allocated`, `reclaimable`): what the Trash takes and, once emptied, frees         |
| title    | own: `app-caches.title.own` with `app` (the owner's name); part: `app-caches.title.part` with `app` and `folder` |
| reason   | `app-caches.reason` with `app` and `size` (the logical size)                                               |
| app      | `for_app(owner's bundle id)`                                                                               |

Left-outs: `LeftOutItem { category, reason: app-caches.left-out.<reason>, sizes }` for rows 7 to 10.

| Category                     | Danger | Why                                                                                                   |
| ---------------------------- | ------ | ----------------------------------------------------------------------------------------------------- |
| `app-caches.library-caches`  | Low    | the app makes it again by itself, but some apps download again what they kept there (music, maps, previews, a pending update) (agent's call 4) |

### 6. Texts (`locales/en.json`)

Plain English (rule 7); the drafts, polished when built:

- `app-caches.library-caches.name` "Caches of your apps"; `.what` "Files your installed apps keep in your Library's
  Caches folder to open and work faster, one folder per app."; `.made-by` "The apps themselves; each item names its
  app."; `.if-removed` "What is inside the folder goes to the Trash and the folder stays. Your documents and the app's
  settings are kept elsewhere and stay."; `.comes-back` "Yes, the app makes it again as it needs it. Its next start may
  be slower, and an app that kept music, videos or maps for offline use downloads them again."
- `app-caches.title.own` "{app}"; `app-caches.title.part` "{app}: {folder}".
- `app-caches.reason` "{app} keeps {size} here to work faster. It is closed now and makes the cache again when it needs
  it."
- Left-outs: `app-caches.left-out.special`, `.not-yours`, `.no-system` (the `dev-caches` sentences adapted),
  `.app-open` "This app, or a part of it, is open and may be using its cache. Quit it and scan again."
- The switch (the module declares no setting of its own; the core's generic `<module>.enabled` switch exists for every
  module and reads these two texts): `app-caches.enabled.label` "Caches of your apps", `.description` "Finds the
  caches your installed apps
  keep for themselves, while each app is closed. The folders are emptied into the Trash; the apps make them again."

### 7. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Unit**: `places.rs` and `exclude.rs` (every refusal, then the shipped tables pinned), `owner.rs` (no id, no app,
  the vendor prefix, a sibling `com.example.Editorx`, case; then own, helper, an id under the app, the longest naming
  id, two copies), `decide.rs` (one test per row, keep before offer; property: a candidate stopped before row 9 never asks the running apps; `decide/tests.rs`), `tests.rs` (the module on
  hand-built matches; the properties that the verdict never depends on the order of matches and that a restored scan
  decides the same live here, since `decide` takes one candidate and no context).
- **Fixtures** (`fixtures/homes/app-caches-*.toml`, `run_module`, golden `fixtures/golden/app-caches.json`):
  `-not-claimed` (a name that is not an id, Apple's, Fresher's own and its `.dev`, a removed app's cache, the vendor
  prefix, a sibling id, a link named like an installed id, a file named like one, an empty folder), `-kept` (a socket
  inside, an entry not the user's, the app open, a helper open, the running apps failing; the installed Agent declares
  its helper `com.example.Agent.Menu`, so "only the helper runs" keeps its folder), `-offered` (the app's own
  folder, a helper's, an id under the app's, two apps with nested ids).
- **Contract**: `assert_module_contract` with `Catalogue::Required`: one category file with the same danger, every text
  and every declared left-out used by a fixture run.
- **Sandbox**: `fixtures/sandboxes/app-caches.toml` and `app-caches-app-open.toml` with their goldens; `EXPECTED.md`
  walked by hand. The cleanup expectation is `emptied`. Their names (`OtherTool`, `com.apple.Maps`, `com.example.Lost`,
  `Scribe`, `Reader`, and the open `Mailer`, an installed `[[case.app]]` with `running = true`) are apart from every
  other case file's, so `npm run sandbox -- all` leaves out no file it did not leave out before.
- **Core e2e**: `crates/e2e/tests/app_caches.rs`, beside `app-leftovers`, `dev-caches` and `jetbrains` (the other
  modules of `Library/Caches`): an installed app's cache is offered by `app-caches` only and a removed app's by
  `app-leftovers` only; a cleanup empties the folder into the sandbox's Trash and keeps it; put back restores every
  entry byte for byte, beside a file the app wrote since; an entry held open skips the item `InUse`; the app open
  leaves it out until a rescan after it quits; a restored scan offers the same folders while the apps are closed and
  leaves out the folders of an app open at launch.
- **UI e2e**: Scan's card for the category and its Details, a finding's title and reason, the review naming Low, a
  cleanup whose detail row reads "Emptied. The folder itself stays." for an item with nothing trashed (the result
  summary counts what went to the Trash), put back, the core's switch in Settings.

## Risks

- **An app keeps something that does not come back in its `Caches` folder** (offline downloads that need a paid
  account, a draft): Low and the texts say so; the Trash keeps it recoverable; 03/02 looks at the largest caches on the
  real Mac, and an app found to keep such data gets an `exclude.toml` row (to verify).
- **An app opened between the scan and the cleanup**: row 10 saw it closed; the cleaner's check 10 skips the folder
  only if the app holds one of its files open. Apps must accept a missing cache (macOS purges `Caches` itself); the
  cost is a slower start.
- **A background agent outside any bundle** writes into an app's cache: it has no id, so row 10 cannot see it; the
  cleaner's check 10 sees its open files.
- **Many small findings**: a Mac has dozens of id-named caches of a few kilobytes; empty ones are not claimed (row 6),
  the rest are listed largest first by the UI. A size floor stays out until the real Mac shows it is needed (03/02).
- **The first real module on `EmptyFolder`**: its Trash holds one entry per top-level item of the folder, not one
  folder; put back returns each. The core e2e and a UI case prove it; the dry run on the real Mac cannot rehearse what
  runs inside the act ([`../../restrictions/deletion-safety.md`](../../restrictions/deletion-safety.md)).
- **The full app on another feature's sandbox**: `npm run dev:sandbox -- app-leftovers` registers every module, so
  its negative "an installed app's files are not claimed" now shows `app-caches` offering that app's caches. The
  checklist's negative sentence names the case file's module (02/01, agent's call 11).

## Open questions

None.

## Decisions

- The user's choice of 2026-11-21 (translated): "after 024 and 029, build the caches of installed apps, autonomously;
  take the recommended decisions; I check everything later." This design is approved by it.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code and the rules; each with what it costs if wrong.

1. **Only `~/Library/Caches/<bundle id>` in this plan; the web-engine finder, browsers, game launchers, the Metal
   shader cache, containers and Apple's apps are deferred** ([`installed-app-caches-more-places`](../../deferred/installed-app-caches-more-places.md)),
   against the idea, which leads with the generic web-engine finder. Those folders are named after the app's name, and
   the rules tie a folder to an app by bundle id only; recognizing the layout alone would claim folders whose owner
   cannot be told (rule 1). Cost if wrong: Electron apps' and browsers' caches (often the largest) wait for the next
   plan; their space stays in the storage map.
2. **Attribution by bundle id through the installed apps, including helpers and ids under the app's**
   (`BundleId::names`): an app's reverse-DNS namespace is its own; an updater's or helper's cache
   (`com.example.Editor.ShipIt`) is the app's. Never a vendor prefix. Cost if wrong: an id under an app's that another
   maker wrote (unseen in practice) is offered in the app's name at Low.
3. **The folder is emptied, not moved** (`EmptyFolder`, the idea and best practice #25): an app may expect its folder
   to exist, with its attributes. Cost if wrong: one Trash entry per top-level item instead of one folder.
4. **Low, not Safe** (the idea said Safe, and the catalogue's Safe examples say "app caches"): the module cannot tell
   per app whether a re-download follows (offline music, maps, a pending update); rule 1 says the higher level when in
   doubt. The catalogue's danger table moves "an installed app's cache" to Low in 03/01. Cost if wrong: one level too
   high; nothing is pre-selected either way.
5. **Kept while the app or a helper runs** (`app-open`), plus the cleaner's open-file check at the act. Cost if wrong:
   none for safety; a cache of an app with an always-running helper (a menu bar agent) is never offered until it quits.
6. **One finding per folder, not per app**: `EmptyFolder` names one path, and plan 029's several-path removal would
   tie this plan to it for apps with two or three folders. Each finding is counted in the app's line, and the part
   title names the folder. Cost if wrong: an app with a helper's cache shows two lines.
7. **No age threshold, no size floor, no setting**: a cache is meant to be emptied at any time; an empty folder is not
   claimed. Cost if wrong: dozens of small lines on a busy Mac (03/02 measures it).
8. **Group System** ("What macOS and your apps keep for themselves"), not Apps ("What apps leave behind"): these apps
   are still installed. Module id `app-caches`, category `app-caches.library-caches` (room for later places). Cost if
   wrong: a group or a name to change.
9. **The installed apps unreadable: nothing claimed, nothing said** (row 3): `app-leftovers` already reports every
   id-named leftover `no-system` then, and two modules leaving out the same folders would count them twice in the
   left-out lines. Cost if wrong: in that rare case the installed apps' caches are not mentioned by this module.
10. **A one-row `places.toml` and an `exclude.toml` of its own** rather than constants: rule 5, the pattern of
    `app-leftovers`; a module may not depend on another, so the exclusions are not shared. `com.apple.` and Fresher's
    own id (and `.dev`) are excluded. Cost if wrong: two small files to keep in step with `app-leftovers`' list.
11. **The sandbox's checklist names the case file's module in a negative's sentence** ("never offered by
    `app-leftovers`: …"), a change to `crates/sandbox/src/expected.rs`: the autotest registers only that module, while
    `npm run dev:sandbox` runs the full app, where `app-caches` now offers the caches of `app-leftovers.toml`'s
    installed app. Cost if wrong: a sentence; without it, that line of the app-leftovers walk reads as failed.
12. **The core e2e of `app-leftovers` is not changed**: it registers its own list of modules, without `app-caches`;
    the split is proven in `app_caches.rs`, which registers both. Cost if wrong: none.
13. **No new dependency** (`serde`, `thiserror`, `toml`; `proptest`, `serde_json` as dev-dependencies) **and no new UI
    component** (the card, Details, the emptied result exist). Cost if wrong: none.

## Conclusions

Built, **awaiting the user's walk**; the numbers, the changes from this design and the calls taken while building
are in the [global readme](./readme.md#conclusions). Of the calls above, the registration of the module moved to the
case-files step, a restored scan judges the open apps again (so the design's restored-scan line holds while the app is
closed), `decide` takes two lifetimes and asks the running apps once at row 9, and the open app of the second case
file is an installed app that is running; the others stand as written. Open questions: none; the real-Mac facts "to
verify" (the largest caches, the size floor, an always-running helper, an updater's cache) are pending the user's dry
run.
