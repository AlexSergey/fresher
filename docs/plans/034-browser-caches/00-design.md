# 034 — Browser and web-engine caches: design

**Status:** ✅ approved (the user's delegation of 2026-11-22) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-22 from the draft [`readme.md`](./readme.md) and from a read of the code on `main` (97d9b314),
read-only. The facts the draft left "to check" were checked on the developer's Mac the same day, read-only (section
"Facts checked"). Approved by the user's delegation of 2026-11-22: build autonomously, take the recommended decisions
and the agent's calls; the user checks everything afterwards.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Browser caches are what a person expects first from a cleaner, and apps built on a web engine keep the same kind of
cache in their support folder. Plan [030](../030-installed-app-caches/readme.md) left both out because the folders are
named after the app, not its bundle id ([deferred](../../deferred/installed-app-caches-more-places.md)).

## Goals

1. The cache folders of each installed browser listed in a table are offered per profile, one finding per cache
   folder, in a new category **Browser caches** (`app-caches.browsers`, Low).
2. The cache folders inside the support folder of each web-engine app listed in the table are offered, one finding per
   cache folder, in a new category **Caches of web-based apps** (`app-caches.web-engine`, Low).
3. Only caches: a folder is claimed only when its exact name is a cache name of its table row; nothing the user would
   call their data (history, cookies, passwords, sessions, site storage) can be reached.
4. A folder is claimed only when its row's bundle id is an installed app; it is offered only while that app and its
   helpers are closed; it is emptied into the Trash and stays (`Removal::EmptyFolder`), as plan 030's caches are.
5. No core change, no new dependency, no new UI component: the module `app-caches` gains one table, two categories, one
   text and a second way to find a folder's owner.

## Non-goals

- History, cookies, saved passwords, sessions, extensions, site storage: privacy cleaning is not this product.
- Safari and other Apple apps (protected containers): they stay deferred.
- A generic search for cache-looking folders inside any app's folder: a rule that cannot tell does not claim.
- The browsers' graphics caches inside their support folders (`Application Support/Google/Chrome/<profile>/GPUCache`
  and the like): the folders are small (under 1 MB each on the developer's Mac), and macOS refuses to list the
  browsers' support folders to an app without Full Disk Access (section "Facts checked"). Deferred (agent's call 6).
- Rows for apps not installed on the developer's Mac: each row is checked on a real install before it is added; the
  well-known candidates are listed for the user to check (agent's call 4).
- Game launchers, the Metal shader cache, sandboxed apps' caches: they stay deferred.

## Facts checked (2026-11-22, the developer's Mac, read-only)

| Fact                                                     | What was found                                                                                               |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Google Chrome's bundle id                                | `com.google.Chrome` (`/Applications/Google Chrome.app`, `Info.plist`)                                        |
| Chromium's bundle id                                     | `org.chromium.Chromium`                                                                                      |
| Firefox's bundle id                                      | `org.mozilla.firefox`                                                                                        |
| Redis Insight's bundle id (an app built on a web engine) | `org.RedisLabs.RedisInsight-V2`                                                                              |
| Chrome's caches                                          | `~/Library/Caches/Google/Chrome/<profile>/{Cache,Code Cache}`; profiles `Default`, `Profile 1`, `System Profile`; beside them `image_cache` and `Storage` (not caches we can vouch for). `Cache` 1.2 GB and `Code Cache` 611 MB in one profile |
| Chromium's caches                                        | `~/Library/Caches/Chromium/<profile>/{Cache,Code Cache}`, the same layout without the vendor folder          |
| Firefox's caches                                         | `~/Library/Caches/Firefox/Profiles/<profile>/{cache2,startupCache}` beside `safebrowsing`, `thumbnails`, `settings` and small JSON files; profile names such as `abcd1234.default-release` |
| The browsers' support folders                            | `~/Library/Application Support/Google/Chrome` and `.../Firefox`: "Operation not permitted" to a shell without Full Disk Access; `.../Chromium/Default` holds `History`, `Cookies`, `Login Data` beside a 548 KB `GPUCache` |
| A web-engine app's layout                                | `~/Library/Application Support/RedisInsight/{Cache,Code Cache,GPUCache,DawnGraphiteCache,DawnWebGPUCache}` beside `Cookies`, `Local Storage`, `Session Storage`, `Preferences`, `config.json`. The same layout in the support folder of a removed code editor (with `CachedData`, `User`, `Backups` beside it) |
| Not on this Mac                                          | Microsoft Edge, Brave, Vivaldi, Arc, Opera; Visual Studio Code, Slack, Discord and other chat apps. Microsoft Teams (`com.microsoft.teams2`) is installed but keeps its data in its container |
| Other modules' claims on these places                    | none: `app-leftovers` claims only bundle-id-shaped names under `Caches` and `Application Support`; `dev-caches` claims `Yarn`, `Homebrew/downloads`, `CocoaPods`, `pip`, `go-build`; `jetbrains` claims `JetBrains/*`. Nothing claims `Google`, `Chromium`, `Firefox` or `RedisInsight` |

## What already exists

| Need                                                | What exists                                                                                         | Plan |
| --------------------------------------------------- | --------------------------------------------------------------------------------------------------- | ---- |
| A path with a `*` in the middle                     | `Pattern::Under(folder, "Google/Chrome/*/Cache")`: the scanner's trie takes `*` at any segment       | 004  |
| A folder summed with everything inside              | `Claim::Whole`, `Match.sizes`, `Match.inside`                                                       | 004  |
| The module, its decision rows, its report           | `modules/app-caches`: `places.toml`, `exclude.toml`, `owner.rs`, `decide.rs`, `report.rs`, `gathered.rs` | 030  |
| Installed and running apps                          | `installed_apps()`, `InstalledApp::owns`, `running_apps()`, `RunningSet::owner_of`                  | 009  |
| Emptying a folder, keeping it                       | `Removal::EmptyFolder`, the cleaner's `remove/empty.rs`, put back                                   | 008  |
| A finding in its app's line of the storage map      | `NewFinding::for_app(id)`                                                                           | 012  |

**Nothing in the core is missing.**

## Design

### 1. The table (`apps.toml`)

Rule data (rule 5): one row per app, the place of its caches and the exact names that are caches.

```toml
[[app]]
id = "chrome"                     # the row's id, kebab case: the start of its findings' identities
category = "app-caches.browsers"  # one of the table's two categories
bundle-id = "com.google.Chrome"   # the app that must be installed, and closed, for its caches to be offered
folder = "app-caches"             # the known folder the place is under
path = "Google/Chrome/*"          # plain names; one `*`, never the first name, stands for a profile
caches = ["Cache", "Code Cache"]  # the exact names inside the place that are caches; nothing else is touched
```

The shipped rows, each checked on a real install (section "Facts checked"):

| Row             | Category                 | Bundle id                       | Folder        | `path`               | `caches`                                                              |
| --------------- | ------------------------ | ------------------------------- | ------------- | -------------------- | --------------------------------------------------------------------- |
| `chrome`        | `app-caches.browsers`    | `com.google.Chrome`             | `app-caches`  | `Google/Chrome/*`    | `Cache`, `Code Cache`                                                 |
| `chromium`      | `app-caches.browsers`    | `org.chromium.Chromium`         | `app-caches`  | `Chromium/*`         | `Cache`, `Code Cache`                                                 |
| `firefox`       | `app-caches.browsers`    | `org.mozilla.firefox`           | `app-caches`  | `Firefox/Profiles/*` | `cache2`, `startupCache`                                              |
| `redis-insight` | `app-caches.web-engine`  | `org.RedisLabs.RedisInsight-V2` | `app-support` | `RedisInsight`       | `Cache`, `Code Cache`, `GPUCache`, `DawnGraphiteCache`, `DawnWebGPUCache` |

The table is read with `deny_unknown_fields` and refused, one `AppsError` variant each, for:

| Variant              | When                                                                                                         |
| -------------------- | ------------------------------------------------------------------------------------------------------------ |
| `Format`             | not valid TOML, an unknown field, a field missing                                                            |
| `Empty`              | no row                                                                                                       |
| `BadId`              | a row id not one kebab-case segment                                                                          |
| `DuplicateId`        | a row id twice                                                                                               |
| `BadBundleId`        | a `bundle-id` that `BundleId::new` refuses or with no dot                                                    |
| `UnknownCategory`    | a category not one of the table's two                                                                       |
| `CategoryWithoutRow` | one of the table's two categories with no row                                                                |
| `WalkedPlace`        | a folder the scan walks as its own (`user-temp`, `shared-temp`)                                              |
| `BadPath`            | a `path` with an empty name, `.`, `..`, a NUL, a name holding `*` that is not `*` itself, or more than one `*` |
| `StarFirst`          | a `path` whose first name is `*` (a generic finder under a known folder)                                     |
| `IdShapedName`       | a `path` whose first name is shaped as a bundle id (`is_bundle_id_shaped`): those names are `library-caches`' and `app-leftovers`' |
| `NoCaches`           | an empty `caches`                                                                                            |
| `BadCache`           | a cache name empty, `.`, `..`, holding `/`, `*` or a NUL, or listed twice in its row                         |
| `OverlappingPlaces`  | two rows under one folder whose paths can name the same folder (the same number of names, each pair equal or one of them `*`) |

Interests, after the one of `places.toml` and in the table's order: for each row, for each cache name, one
`Pattern::Under(folder, "<path>/<cache>")`, `Claim::Whole`. A match names its row and cache by its interest's index.

### 2. The owner of a table row

For a match of a table row, the owner is the installed app whose bundle id equals the row's `bundle-id` (ids compare
ignoring case, as `BundleId`'s equality does); two copies of the app are one owner, the first by path. No installed
app has it: not claimed. Never by the folder's name. The finding's app is the row's bundle id (`for_app`), so it is
counted in the browser's line of the storage map.

### 3. The decision (`decide.rs`, the rows of plan 030 with a second naming)

`decide` takes a `Naming`: `FolderName` (plan 030's `library-caches` row) or `Table(&BundleId)` (a row of
`apps.toml`). Only rows 2 and 4 differ:

| #   | When                                                                                       | `FolderName` (plan 030)                         | `Table(id)` (this plan)                    |
| --- | ------------------------------------------------------------------------------------------ | ----------------------------------------------- | ------------------------------------------ |
| 1   | not a folder (a file, a link, a socket)                                                    | not claimed                                     | not claimed                                |
| 2   | the id                                                                                     | the name gives no id, or `exclude` keeps it: not claimed | `exclude` keeps the row's id: not claimed |
| 3   | the installed apps cannot be listed                                                        | not claimed                                     | not claimed                                |
| 4   | the owner                                                                                  | no installed app names the folder: not claimed  | no installed app has the row's id: not claimed |
| 5   | not read in full                                                                           | not claimed                                     | not claimed                                |
| 6   | nothing in it (logical size 0)                                                             | not claimed                                     | not claimed                                |
| 7   | a socket, a pipe or a device inside                                                        | left out `special`                              | left out `special`                         |
| 8   | not the user's, a system flag, restricted, the user's lock, or such an entry inside        | left out `not-yours`                            | left out `not-yours`                       |
| 9   | the running apps cannot be listed                                                          | left out `no-system`                            | left out `no-system`                       |
| 10  | a running app has the owner's id or a helper's, or `owner_of(path)` names one              | left out `app-open`                             | left out `app-open`                        |
| 11  | otherwise                                                                                  | offered                                         | offered                                    |

The four left-out reasons and their texts are plan 030's, unchanged ("This app, or a part of it, is open and may be
using its cache. Quit it and scan again." reads true for a browser). A browser keeps its cache files open while it
runs, and the cleaner's open-file check stays the last guard at the act.

### 4. Findings

| Part     | Profile row (`path` has a `*`)                                         | Row without a profile                          |
| -------- | ---------------------------------------------------------------------- | ---------------------------------------------- |
| category | the row's, Low                                                         | the row's, Low                                 |
| identity | `<row id>/<profile>/<cache>` (`chrome/Profile 1/Code Cache`)           | `<row id>/<cache>` (`redis-insight/GPUCache`)  |
| subject  | `Subject::Path` of the cache folder; removal `Removal::EmptyFolder` of the same path | the same                         |
| sizes    | the match's                                                            | the match's                                    |
| title    | `app-caches.title.profile` "{app}, {profile}: {folder}" (`Google Chrome, Profile 1: Code Cache`) | `app-caches.title.part` "{app}: {folder}" (plan 030's) |
| reason   | `app-caches.reason` with `app` and `size` (plan 030's)                 | the same                                       |
| app      | `for_app(row's bundle id)`                                             | the same                                       |

`profile` is the name the `*` matched (the cache folder's parent); `app` is the installed app's shown name; `folder` is
the cache name. Left-outs: `LeftOutItem { category: the row's, reason, sizes }`.

### 5. Categories and texts

| Category                | Danger | Why                                                                                     |
| ----------------------- | ------ | --------------------------------------------------------------------------------------- |
| `app-caches.browsers`   | Low    | the browser fills it again as the user browses; pictures and videos download again      |
| `app-caches.web-engine` | Low    | the app makes it again; its next start is slower                                        |

`locales/en.json` gains (plain English, rule 7; polished when built):

- `app-caches.browsers.name` "Browser caches"; `.what` "Copies of web pages, scripts and pictures your browsers keep so
  sites open faster, kept for each browser profile."; `.made-by` "The browsers themselves; each item names its
  browser, its profile and the cache folder."; `.if-removed` "What is inside the folder goes to the Trash and the
  folder stays. Your history, bookmarks, passwords, cookies and open tabs are kept elsewhere and stay."; `.comes-back`
  "Yes, the browser fills it again as you browse. Sites may open a little slower the first time, and pictures and
  videos download again."
- `app-caches.web-engine.name` "Caches of web-based apps"; `.what` "Files that apps built with web technology, such as
  chat apps and code editors, keep in their own support folder to draw their windows and run faster."; `.made-by`
  "The apps themselves; each item names its app and the cache folder."; `.if-removed` "What is inside the folder goes
  to the Trash and the folder stays. Your messages, files, settings and sign-ins are kept elsewhere in the app's
  folder and stay."; `.comes-back` "Yes, the app makes it again as it needs it. Its next start may be slower."
- `app-caches.title.profile` "{app}, {profile}: {folder}".

The switch (`app-caches.enabled.*`), the reason and the left-out texts are unchanged.

### 6. The module's shape after the change

- `categories.rs`: `PLACE_CATEGORIES` (`app-caches.library-caches`, Low) and `APP_CATEGORIES` (`app-caches.browsers`,
  `app-caches.web-engine`, Low), and `CATEGORIES`, the three in that order (the descriptor's list).
- `apps.rs` (new): `Apps::shipped()`, `Apps::parse(text, categories)`, `AppsError` (re-exported from `lib.rs` as
  `PlacesError` and `ExcludeError` are), `Apps::interests()`, `Apps::rows()`; `AppRow { id, category, bundle_id,
  folder, path, profile: bool, caches }`.
- `lib.rs`: the interests are the places' then the table's; a `Vec<Source>` beside them says what each interest is
  (`Source::Place { row }` or `Source::App { row, cache }`); `AppCachesError::Apps`.
- `gathered.rs`: `Candidate.row: usize` becomes `Candidate.source: Source`.
- `decide.rs`: the `Naming` parameter; `owner.rs`: `installed(id, apps) -> Option<Owner>`.
- `report.rs`: `tell` takes what the source says (the category, the identity, the title) for either kind of row.

### 7. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Unit**: `apps.rs` (every refusal, then the shipped table pinned: four rows, eleven interests, both categories
  used, no row's id kept by `exclude.toml`), `owner.rs` (`installed`: no app, a sibling id, case, two copies),
  `decide.rs` (rows 2 and 4 for `Table`, and one table candidate through rows 7 to 11), `tests.rs` (the module on
  hand-built matches of both kinds; the order and restored-context properties cover table candidates).
- **Fixtures** (`fixtures/homes/app-caches-tables-*.toml`, golden `fixtures/golden/app-caches-tables.json`):
  `-not-claimed` (a browser not installed, a name not in the row, a case variant `cache`, a link named `Cache`, a file
  named `Code Cache`, an empty cache folder, a cache folder directly under the browser's folder with no profile, a
  cache-named folder of an app not in the table, the profile's and the app's data beside the caches), `-kept` (a socket
  inside, a locked entry, the browser open, only a browser's helper open, the running apps failing), `-offered` (two
  profiles of one browser, a second browser, the web-engine app's five caches).
- **Contract**: the two new category files with the same danger; every text and left-out used by a fixture run; every
  removal an `EmptyFolder` of a matched path.
- **Sandbox**: `fixtures/sandboxes/app-caches-browsers.toml` and `app-caches-browsers-open.toml` with their goldens;
  `EXPECTED.md` walked by hand.
- **Core e2e**: `crates/e2e/tests/app_caches_browsers.rs`: the module beside `app-leftovers`, `dev-caches` and
  `jetbrains`; no folder offered twice; a profile's data and the app's data never in a finding; an open browser's
  caches left out until a rescan after it quits; an entry held open skips its cache; a cleanup empties each cache and
  put back restores it; the storage map counts the caches in the browser's line.
- **UI e2e**: the two categories' cards and Details, a finding's title with its profile, the review naming Low, a
  cleanup that empties, an open browser's cache that cannot be selected.

## Risks

- **A cache name a browser uses for data in a later version**: the table names exact folders that hold caches today;
  the categories' docs and the rows' comments say what each is; a row is changed when a browser changes. The Trash
  keeps everything recoverable.
- **A browser opened between the scan and the cleanup**: row 10 saw it closed; the cleaner's open-file check skips the
  cache when the browser holds a file of it open (Chromium-based browsers hold their cache index open while running).
- **A profile with a strange name**: the `*` matches any UTF-8 name, so a profile folder named like the browser's own
  files (`System Profile`, `Guest Profile`) is a profile too: its `Cache` is a cache. A link standing for a profile is
  never followed, so nothing under it matches.
- **Few rows**: on a Mac with only Edge or Brave, the categories offer nothing until a row is added; the space stays in
  the storage map as the user's folders. The candidates are listed for the user (agent's call 4).
- **The shared sandbox world** (`npm run sandbox -- all`): a browser declared running in one case file and closed in
  another would contradict; the case files use Chrome only as the open browser (as `downloads-browser-open.toml` does)
  and Chromium for the closed one (02/01).

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated (translated): "Restore the last session's context and start implementing; work
  autonomously with subagents, take the recommended decisions; implement all the features; write into docs/deferred
  what I will need to look at — I will check everything tomorrow." This design is approved by it, with the draft's
  agent's calls and the calls below.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Each with what it costs if wrong. Calls 1 to 4 are the draft's.

1. **Caches only, by exact name from a table**; never history, cookies or sessions. Cost if wrong: none for safety; a
   cache the table does not name stays.
2. **Known apps only**; no generic finder. Cost if wrong: an unlisted app's cache waits for its row.
3. **Safari stays deferred.** Cost if wrong: Safari's cache waits.
4. **Only rows checked on a real install ship**: Chrome, Chromium, Firefox and one web-engine app (Redis Insight), the
   ones installed on the developer's Mac. Edge, Brave, Vivaldi, Arc, Opera, Visual Studio Code, Slack, Discord and
   other chat apps are listed in [`docs/deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md)
   with their expected places, to be added once checked. Cost if wrong: the web-engine category is small on most Macs
   until rows are added.
5. **One finding per cache folder, not per profile** (the draft said "a finding a profile"): `Removal::EmptyFolder`
   names one path, and an item of several folders that are emptied would be a core change. Each finding names its
   browser, its profile and the cache folder, and all are counted in the browser's line. Cost if wrong: a profile shows
   two lines (`Cache`, `Code Cache`) instead of one.
6. **The browsers' support-folder graphics caches are not rows**: small, and the folders are protected by macOS without
   Full Disk Access. Cost if wrong: under 1 MB per profile stays.
7. **Firefox's `startupCache` is a cache, `safebrowsing` and `thumbnails` are not rows**: `startupCache` is rebuilt at
   start; the safe-browsing lists matter for safety until they download again, and thumbnails are small. Cost if wrong:
   a few megabytes stay.
8. **The title shows the folder's own name** (`Code Cache`, `cache2`) rather than a translated kind of cache: one text,
   no table of kinds. Cost if wrong: a less friendly title; the category's texts explain.
9. **The table's first name may not be shaped as a bundle id, and two rows may not overlap**: keeps one claim per
   folder inside the module and away from `app-leftovers`. Cost if wrong: none.
10. **The same four left-out reasons and texts as plan 030**; no new reason. Cost if wrong: a sentence.
11. **The table is a second file (`apps.toml`), not more rows of `places.toml`**: a row of `places.toml` finds its owner
    by the folder's name, a row of `apps.toml` by the table; two shapes, two tables. Cost if wrong: none.
12. **The switch's description is unchanged** (browsers are installed apps). Cost if wrong: a sentence.

## Conclusions

Written in [the plan's readme](./readme.md#conclusions): what was built, the numbers, the changes from this design and the calls taken while building.
