# Writing a module

Every feature of the app is a module: one crate under `modules/<id>/`, named `fresher-module-<id>`, that depends on
`fresher-core` and, when it has rules, on `fresher-rules` — never on another module. A module never walks the disk and
never deletes: it says what it wants to see during the one shared walk, turns what the walk matched into findings, and
describes each removal as data. This page is the pattern every module plan follows; the `demo` module
(`modules/demo/`) is the smallest working example, `jetbrains` (`modules/jetbrains/`, plan
[016](../plans/016-jetbrains/readme.md)) the first real one, and `temp` (`modules/temp/`, plan
[017](../plans/017-temp-files/readme.md)) the example of a module that judges each item by its age and refuses it by
its kind and owner, and `dev-caches` (`modules/dev-caches/`, plan
[018](../plans/018-developer-caches/readme.md)) the example of a module with a data table of places and signs, and `local-ai-models` (`modules/local-ai-models/`, plan
[023](../plans/023-local-ai-models/readme.md)) the example of a module with one finding per model, named from its folder, and `unused-toolchains` (`modules/unused-toolchains`, plan
[024](../plans/024-unused-toolchains/readme.md)) the example of a module that reads small files and judges each version by what points at it, and `logs` (`modules/logs/`, plan
[033](../plans/033-logs/readme.md)) the example of a module that claims every file of a place by the place's contract and decides each one by its age. Set up by plan
[002/06](../plans/002-base-architecture/06-module-kit/readme.md).

## The crate

```
modules/<id>/
├── Cargo.toml                 fresher-module-<id>; [lints] workspace = true; fresher-testing as a dev-dependency
├── rules.toml                 the rules, if the module has any (embedded with include_str!)
├── locales/en.json            every text the module shows, keys `<id>.…` only
├── src/lib.rs                 the descriptor and the `Module` implementation
└── tests/                     contract.rs and the fixture tests, through fresher-testing
fixtures/homes/<id>-*.toml     the fixture homes of its tests (negative ones first)
fixtures/golden/<id>.json      its golden report
```

`Cargo.toml` takes `version`, `edition`, `rust-version`, `publish` and `license` from the workspace, like every crate.
Ids, text keys and descriptors are checked constructors in the core, so the module's `new()` returns a `Result` with a
small `thiserror` error of its own: no panicking path.

## The `Module` trait in short

| Method            | What the module does there                                                                          |
| ----------------- | --------------------------------------------------------------------------------------------------- |
| `descriptor()`    | its id, group, categories (each with its danger level), settings and dictionary (`locales/en.json`) |
| `interests()`     | what the walk should report to it: a folder name, a path under a known folder, a marker file or an extension |
| `on_match(…)`     | one match of one of its interests (`matched.interest` is the index in its own list) → findings     |
| `finish(…)`       | the end of the walk: work that needs every match (the newest version), or that reads no disk (a tool) |

- A module names its group: `ModuleDescriptor::new(id, group, categories, settings, locales)`, a `ModuleGroup` of the
  core (`System` for what macOS and the user's apps keep for themselves, such as `temp`; `Apps` for what apps leave behind;
  `Files`, shown as "Your files", for the user's own files, such as old installers; `Development`; `Debug` for
  the dev-only `demo` and `trial`, which the registry refuses in a release build), shown in that order. The screens
  show modules under their group's name (`core.group.<group>.name`, `.description`).
- A module may offer choices of its categories: `ModuleDescriptor::with_choices(Vec<ModuleChoice>)`, each a kebab-case
  name and the categories it selects (texts `<module>.choice.<name>.label` and `.description`), carried to the UI as
  `ModuleInfo.choices`. A finding of an app that may no longer be installed names it with
  `NewFinding::for_app_named(id, name)`, and the storage map shows that name.
- A module's problems are declared: `ModuleDescriptor::with_problems(keys)` lists every key it may report. A failed
  part returns `ModuleError::Failed { reason }` with a declared `Text` (a technical detail goes under `DETAIL_ARG`,
  never named in the sentence). An item the module recognized and does not offer is `out.left_out(LeftOutItem { … })`:
  a protected or too-new item, an app that is open. It is not for an unreadable folder, a missing permission or a path
  no rule knows (the walk and the access status report those, and a rule that cannot tell what a path is does not
  claim it). The `Vec<NewFinding>` sink used by tests drops left-outs.
- A category that cannot see everything without a permission, or cannot run without a tool, declares it:
  `CategoryDescriptor::with_needs(vec![Need::sees_more(Capability::OtherAppsData)])` (`Need::required` when it cannot
  run at all). A module whose category needs a tool gives its check with `ModuleDescriptor::with_tool_check(ToolCommand)`;
  `with_tool_check` refuses a check no category needs, and the registry refuses a needed tool with no check
  (`DescriptorError::MissingToolCheck`). The engine resolves the rest
  ([`permissions.md`](./permissions.md)).
- A category id is `<id>.<name>`; its five texts are found by convention: `<id>.<name>.name`, `.what`, `.made-by`,
  `.if-removed`, `.comes-back`.
- A setting key is `<id>.<name>`, with `<id>.<name>.label` and `.description`. The settings schema adds the switch
  `<id>.enabled` for every module; the module ships its label and description too. A module with an "unchanged
  for N days" setting declares a `DaySetting` constant (name, range, default) and uses its `descriptor(id)` and
  `days(cx.settings())`.
- `NewFinding::new(&category, …)` starts at the category's danger level; `raise_danger` only goes up. A module that
  knows which app a finding belongs to says so with `for_app(bundle_id)`: the storage map counts the finding in that
  app's line (an id no installed app has is shown as an app no longer installed).
- The ports a module may use come in `ModuleContext`: the clock, the command runner (an external tool, with a fixed
  argument list; pass `cx.cancel()`, so a cancelled scan stops the tool), the system's facts (`cx.system()`, below),
  its settings, the cancel token. A long `finish` checks `cx.cancel()` and returns `ModuleError::Cancelled`.
  `cx.is_restored()` says the scan is restored from the last one's cache, not walked: a module that judges by the
  entries' times must not then (plan 016). In `finish`, `cx.is_walk_complete()` says the walk read every folder
  (`false` during the walk, and after a walk that could not read a folder or stopped early): a module that decides
  from every match together must not offer when a folder it did not see may change the answer (plan 016).
  `cx.place_seen(interest)` says, in `finish`, what the walk saw of the folder that interest of the module matches in
  (its place: `Library/Caches` for `Library/Caches/*`; only an `Under` pattern with no `*` before its last segment
  has one): `Read` (listed in full; what is inside a match is the match's), `Absent` (its parent read, no such folder),
  `Unread` (it, a folder on the way, or an entry of its listing not read; it or a folder on the way claimed whole counts),
  `NotScanned` (avoided, another volume, another user's, outside every root) or `Unknown` (during the walk, in a
  restored scan, an interest without one place). `PlaceSeen::is_complete()` is `Read` or `Absent`. A module that
  decides from every match of its own places checks them, not `is_walk_complete()`, which any unreadable folder of
  the scan makes `false` (on every Mac without Full Disk Access); one that decides from the whole walk keeps
  `is_walk_complete()` (plan 019).
- **A module never writes an OS path** (the user's decision of 2026-11-15): a place is a known folder and a relative
  path, `Pattern::Under(KnownFolder::AppCaches, "JetBrains/*")` (in rules, `under = { folder = "app-caches", path =
  "JetBrains/*" }`); `KnownFolder::Home` is the home folder itself. The walk resolves the folder through the
  `KnownFolders` port; a folder this system does not have matches nothing.
- `Interest::only_bundle_ids()` (plan [019](../plans/019-uninstalled-app-leftovers/readme.md)) makes an anchored
  interest's last `*` match only a name shaped as a bundle id (`is_bundle_id_shaped`: what `BundleId::new` accepts,
  with at least one dot): `com.example.App` and `com.example.App.plist`, never `JetBrains`, `Homebrew` or
  `.DS_Store`. A whole claim of `Caches/*` would otherwise swallow the places other modules claim; a literal beside it
  (`Caches/JetBrains/*`) still leads inside. Only for a `Pattern::Under` whose last segment is `*`
  (`InterestError::NoStarToShape` otherwise); the shape is part of the interest and of the matcher's fingerprint.
- `Interest::only_suffixes(suffixes)` (plan [020](../plans/020-downloads/readme.md)) makes the last `*` match only a
  name that ends with one of the suffixes and is longer than it, ASCII case ignored: with `.download` and `.app`,
  `movie.download` and `Install macOS Sonoma.APP`, never `report.pdf`, `Projects` or `.download` itself. It is for a
  whole claim of a few kinds of names in a folder of the user's (`Downloads/*`): claimed whole with a plain `*`, every
  other folder there would be opaque to the storage map and to every other module. Each suffix is a dot and a name
  whose parts between dots are not empty (`.tar.gz`), listed once (`InterestError::BadSuffix` otherwise), kept
  lowercased (`Interest::name_suffixes()`); an interest takes one shape, bundle ids or suffixes
  (`InterestError::TwoShapes`). The suffixes are part of the interest and of the fingerprint. A plain `*` of the same
  place takes the name too, and so does a shaped one when the name is also shaped as a bundle id
  (`trip.zip.download`): a folder that ends a `Whole` and an `Item` interest is summed whole and reported to both, the
  `Item` hit with `Sizes::ZERO`. The cost is one suffix check for each name of such a folder that no literal took.
- Two known folders lie outside the home: `UserTemp` (the user's own temporary folder,
  `/private/var/folders/<xx>/<hash>/T`) and `SharedTemp` (`/private/tmp`). While a registered module's interest is
  anchored in one, and no scan root holds it, the walk reads it as a place: a root of its own after the scan roots,
  walked in full on every scan, where only the interests anchored in it match. A restore emits nothing inside a place
  ([`README.md`](./README.md), "Places outside the scan roots").

## The system's facts (`cx.system()`)

Plan [009](../plans/009-system-knowledge/readme.md). One `SystemFacts` per scan, shared by every module; each fact is
gathered on the first ask and kept for the scan, a rescan gathers it anew, nothing is written.

| Ask                          | Answers                                                                                           |
| ---------------------------- | ------------------------------------------------------------------------------------------------- |
| `locate(KnownFolder)`        | where this system keeps the folder                                                                |
| `installed_apps()`           | every installed app: bundle id, team id, name, version, build, path, its helpers' bundle ids, `changed`: when the bundle was last changed on disk (its status change time, whole seconds; `None` when unreadable) |
| `find_apps(ids, cancel)`    | the bundles with one of the ids anywhere Spotlight knows them (a sandbox: its own apps); asked anew each call, after checking once per scan that Spotlight sees every mounted volume; an error when it cannot |
| `installed_apps_complete()`  | the same, but an error when the listing could not read an apps folder, a folder in one or a bundle (`InstalledList::unread`) |
| `running_apps()`             | the running apps (`RunningSet`); `owner_of(path)` names the running app a path is named after     |
| `receipts()`                 | the installer receipts the system keeps (package file name, when installed); an error when the folder cannot be read; asked once per scan |
| `system_version()`           | the version of the running macOS (`SystemVersion`, major and minor); an error when it cannot be read; asked once per scan |
| `projects()`                 | the projects the walk found: root, kinds, artifact folders and Rust caches with their sizes and last change, and the last activity; `None` during the walk |
| `small_file(folder, name)`   | a text file of at most 4 KiB, read whole at every call through the walk's reader (a regular file only, never a link); `Missing`, `NotAFile`, `TooLong`, `NotText`, `Unreadable` otherwise. Only for a file the module's own matches showed: a name its anchored interest holds, the marker of its marker match, or a file it matched |

- An app is identified by its bundle id and team id only (`BundleId` compares ignoring case), never by a loose name.
  `BundleId::names(name)` tells whether a folder's or a file's name is named after the id: the whole name, or the id
  followed by a dot (`com.example.App.savedState`), never `com.example.Appx`.
- A module that keeps something because its app is installed asks `installed_apps_complete()`: an app the listing
  could not read must not count as not installed (plan 016).
- At the commit the engine lists the installed apps too (once per scan, even when no module asked), to attribute each
  app's data folders and its bundle in the storage map.
- The projects are known in `finish`, not in `on_match`. A project's last activity is the newest folder time outside
  its artifact folders and `.git` (and the artifact folders and `.git` of the projects inside it), or the newest folder
  time of `.git/refs/heads`: `Known` when every folder counted was read, `AtLeast` when one was not (unreadable, on
  another volume, collapsed, or claimed whole by a module's interest), `Unknown` when the walk saw no time. Status
  changes and files' times are never used: a folder's time moves on every new file, rename and atomic save, and a
  migration moves the status change of every file. A project whose artifact was removed looks active from that day.
  Details: [plan 021](../plans/021-stale-projects/00-design.md). `run_module` finds the projects as the engine does.
- An item a running app owns is skipped by the cleaner anyway; a module may raise a finding's danger or say "quit the
  app first" with `running_apps()`.
- In tests, `run_module`'s `Ports` carry `apps: FakeInstalledApps`, `running: FakeRunningApps` and the known folders
  (by default laid out under the fixture root); `installed_app("com.example.Editor", "Editor", "2026.3")` makes one
  up.
- In tests, `small_file` reads the fixture tree on the disk: `fake_system` and `run_module` give `DiskFiles` (the
  standard library: a regular file only, never through a link; `Ports::files` replaces it), and the hand-built ports
  of the other tests give `NoFiles` (every file `Unreadable`). The app reads through the scanner's own reader (in
  sandbox mode the sandbox's stated lens over it), with the same no-follow call the walk uses for `CACHEDIR.TAG`.
- A module that returns an error, or panics, ends `ModuleFinished { Failed }` alone: the engine catches the panic
  (`catch_unwind`; the release profile unwinds), the other modules finish and the scan ends `Partial`.

## What a module sees of the walk

- Only the matches of its own interests, one `Match` each: `interest` (the index in its own `interests()`), `path`,
  `kind` (a folder, a file, a link, or `Other`: a socket, a named pipe or a device, reported as itself, never read,
  never a marker; a rule never claims one and the cleaner refuses one), `modified`, `changed` (the status change, which
  no tool sets back), `accessed` (`None` for a folder, whose access time moves when the walk lists it), `siblings`
  (which of the interest's sibling names exist next to the entry), `holds` (below) and `marker` (for a `CACHEDIR.TAG`
  marker, or a held `CACHEDIR.TAG`, whether the tag holds its signature: `Valid` or `Invalid`; `None` for every other
  interest). The module never reads the file itself.
- **What a matched folder holds**: an anchored interest (`Pattern::Under`) may name, with
  `Interest::holding(names)` (at most 64 single names), entries its matched folder must be checked for: a tool's sign
  inside its own folder (`index-v5`, `CACHEDIR.TAG`). `holds` lists which of them the folder's own listing holds, in
  the interest's order: an entry that could not be read is absent, any kind counts, but a `CACHEDIR.TAG` only as a
  regular file. A held `CACHEDIR.TAG` also sets `marker`: `Valid`, or `Invalid` when it is absent, not a regular file,
  unreadable or without the signature. For a file, a link, an `Other` entry, a folder not entered or not read
  (another volume, another user's, unreadable) and an interest that names none, `holds` is empty and `marker` `None`.
  A rescan reads it again with the folder's listing; a folder it copies keeps what the cache recorded.
- For a `Claim::Whole` folder, `inside` (`None` for anything else), gathered while the walk sums it:
  `newest_change` (the newest modification or status change of the folder and everything inside it),
  `newest_file_access` (the newest access of a file inside: lazy, only ever a reason to keep), `special` (a socket, a
  pipe or a device inside), `foreign` (an entry inside not the user's, or locked or flagged by the system),
  `lock_named` (an entry inside named as a program's lock: `*.lock`, `*.pid`, `LCK..*`, `.#*`) and
  `complete` (every folder inside was read; one unreadable, avoided or on another volume was not).
- Three sizes in `sizes`: `logical` (what Finder shows), `allocated` (what the entry takes on disk) and `reclaimable`
  (the part of `allocated` shared with no clone or other hard link: what removing it frees), with their
  completeness. For a `Claim::Whole` folder they cover everything inside it.
- **Matches come in no fixed order**: the walk is parallel, and a rescan or a restore at launch hands the same matches
  in another order. `finish` must not depend on it (sort, or keep the newest by value).
- The walk matches the interests of every registered module, enabled or not, so turning a module off never changes
  what the walk reads or the cache; the engine drops a disabled module's matches before they reach it.
- A rescan runs every module over all the matches again, not over the changes; a restore at launch runs the modules
  with interests over the cached matches (a module with none waits for the user's scan).

## Rules or code

- **Rules** (`rules.toml`, matched by `fresher-rules`) when a kind of leftover is recognized by where it is and what it
  is called: a cache folder, a folder with a marker file, a version-named folder older than the newest one. The format
  is in `crates/rules/src/format.rs`; every rule states what it must never match, a danger level no lower than its
  category's and its removal. The module's interests are `RuleSet::interests()`, in that order, and `on_match` pushes
  what `RuleSet::evaluate(matched, cx)` returns; `RuleSet::stale` tells old versions from the current one in `finish`.
  A rule's texts are `<id>.rule.<rule id>.title` (with `{name}`) and `.reason`.
- **Code** when the answer is not in a name and a path: an external tool's list (Docker images), file contents, a
  question to another port. Code that can be a rule is a rule.

**`jetbrains` is code, not rules**: whether an older IDE version's folder is offered depends on which apps are
installed and which are running, across three places (settings, caches, logs), and on the other folders of the same
product; a rule sees a name and a path only. Its products are data (`products.toml`), the decision a pure function
(`decide.rs`, a table of eight rows checked in order), and every folder it recognizes and does not offer is a
`left_out` with its reason (the version in use, a newer one, an IDE that is open, an app of unknown version, a list
of apps that could not be read, a removed IDE's data that changed lately): the user sees why, never a silent skip.

**`dev-caches` is code with a data table**: each tool's cache is a folder at a default place (`tools.toml`: the place,
the sign the tool writes there, the category, whether it comes back by a download or a build), claimed whole, and the
sign is checked by what the folder holds (`Interest::holding`: npm's `index-v5` and `content-v2`, a valid
`CACHEDIR.TAG`). Cargo's and Homebrew's signs lie in a folder beside the cache, so the sign is its own item match, a
holder that the cache's match finds in `finish`; the answers wait for `finish` for that reason, and for the running
apps (derived data is left out while Xcode is open). Like `jetbrains`, the decision is a pure function (`decide.rs`, a
table of nine rows) and every cache it recognizes and does not offer is a `left_out` with its reason (not the user's,
a special entry inside, its app is open, the running apps could not be read).

**`app-leftovers` is code with data tables, and the example of a module that decides per app**: its places
(`places.toml`: eight places in `~/Library`, nine rows with a suffix, a kind, a category and whether the row proves an
id; app containers are not read) match only names shaped as a bundle id (`Interest::only_bundle_ids()`), and a name without the row's suffix is not
an id. The leftovers of one id are gathered in `on_match`; `finish` decides each id from all of them, in the rows of a
pure function (`decide.rs`): a proof (a present saved state of the same id; an absent place never proves), no installed or running app with the id or a related id, every place seen in full (`cx.place_seen`), nothing
changed within the days setting, and Spotlight last (`find_apps` with the id and its dot-prefixes, one call). Any
doubt keeps all of the id's leftovers, each reported as a `left_out` with its reason. The exclusion table
(`exclude.toml`) keeps `com.apple` and system components. Group Containers and launch agents are not places of the
module. A leftover that is a socket or not the user's is left out alone (`decide_item`), the rest of the id is offered.

**`app-caches` is code with data tables, and the example of a module that shares an interest and claims the other
half**: its one place (`places.toml`) is the same `Caches/*` shaped to bundle ids (`Interest::only_bundle_ids()`) that
`app-leftovers` has, so the walk reports each id-named folder to both modules and each decides. `app-caches` claims
exactly what `app-leftovers`' row 4 leaves unclaimed: the folder of an installed app, found by `BundleId::names` over
`installed_apps()` and the apps' helpers (the id itself, a helper's, or an id under one; the longest naming id wins,
never a vendor prefix or a sibling id); a removed app's is `app-leftovers`' and nobody's else, and no folder is ever
offered by both (the core e2e asserts it). The folders are gathered in `on_match` and decided in `finish` by a pure
function (`decide.rs`, eleven rows, the same questions as `dev-caches`): the installed apps are asked once, and the
running apps only when a candidate reaches the row that needs them (a folder whose app or helper runs is left out
`app-open`). The exclusion table (`exclude.toml`: `com.apple.` and Fresher's own id) keeps what is never claimed. It is
the first real module whose removal is `Removal::EmptyFolder`: the folder stays and its entries go to the Trash one by
one (the cleaner's `Emptied`), and one finding is one folder, named after its app and counted in the app's line of the
storage map (`NewFinding::for_app`). The web-engine caches in `Application Support`, browsers' and game launchers'
caches are named after the app's name, not its bundle id, so they wait for a table that ties a name to an id
([`deferred`](../deferred/installed-app-caches-more-places.md)).

**`downloads` is code with data tables, and the example of a module whose categories need a guarded folder**: every
category declares `Need::required(Capability::Folder(GuardedFolder::Downloads))`, so the module is unavailable, and
nothing in `~/Downloads` is read, until the user allowed the folder; the engine then runs it in the scan that first
tries the folder (`permissions.md`). It reads one folder's first level with two interests of the same place: an `Item`
star for every file (`Claim::Item`, so the user's folders in Downloads stay open to the walk and to the storage map),
and a `Whole` star that matches only the names with a suffix (`Interest::only_suffixes`: Safari's `.download` bundle
and `.app`, held by `Info.plist` or `Contents`), summed whole. The suffix tables (browsers' unfinished suffixes with
the app that writes each, the macOS installers' names), the decision (a pure function, `decide/`) and the texts are
data and code of the crate; an installer is claimed only when a receipt or its name proves it is an installed app's at
that version or older (`cx.system().receipts()`, `installed_apps()` with `changed`, `system_version()`), and the rest
of what it recognizes is a `left_out` with its reason. A disk image is never mounted, an archive never expanded.

**`stale-projects` is code with a data table, and the example of a module that decides from `cx.system().projects()` in
`finish`**: its interests are `Item` matches of the artifact folders' names only to learn which folders exist (they are
claimed one by one, never whole, so no folder named `build`, `target` or `incremental` becomes opaque to the storage
map), and nothing is claimed whole. The sizes, the complete or partial state and the caches come from the projects
(`Project`'s artifacts and caches, `Artifact`, sized from the walked tree after the walk by `DiskWalk::subtrees`), not from a
claim. The table (`artifacts.toml`: a folder, the project's kind, a category, the command that brings it back) is
checked against the core's artifacts when the module is built. `finish` groups a monorepo's projects under the
outermost one (the group's last activity is the newest of its members'), then decides each group and each Rust cache
in a pure function (`decide.rs`): the days setting (`DaySetting`, 365 by default), the last activity (`Known`, `AtLeast`
or `Unknown`: only a `Known` one older than the days offers), whether every size was read in full, and whether the scan
is a restored one (nothing is offered, `scan-again`). Source, the project folder, `.git` and framework output are not in
the table and are never offered; a cache is offered by its own age (7 days), not its project's. Every project it
recognizes and does not offer is a `left_out` with its reason.

**`local-ai-models` is code with a data table, and the example of a module that offers one finding per model**: each
store is a row of `stores.toml` (the place under the home with `*` for each name, the shape of a model's folder name,
the sign its folder holds, the app whose store it is, the category), one whole claim per store, so a model's folder is
summed with everything inside it. The model's name is read from its folder's path by components, never from a file
(`hub-repo`: `<kind>--<owner>--<name>`, titled `owner/name`; `publisher-model`: two folder names), and a folder that
does not fit the shape or does not hold the store's sign is not claimed. A Hugging Face repository's kind picks its
category (`models`, `datasets`); a kind the row does not list (a Space) is never claimed, and the hub's locks are
summed, never claimed. The decision is a pure function (`decide.rs`, a table of nine rows) taken in `finish`, because it
needs the running apps: a model of LM Studio is left out while LM Studio is open (`app-open`), or when the apps could
not be listed (`no-system`). Two more rows refuse a model as `dev-caches` does: a special entry inside it, and a folder
that is not the user's. No age is judged: every model is offered with the date it was last opened or changed in its
reason. A finding's identity is `<row id>/<the folder names>` and its removal is the model's folder moved to the Trash,
never the tool's own command (Hugging Face's cache has no index beside its folders, and a tool never runs in a sandbox).

**Ollama is a row of the same table with a shape of its own (`ollama-store`), and the example of a module whose finding
is several paths**: one model is not one folder. The store `~/.ollama/models` holds `manifests/<host>/<namespace>/
<model>/<tag>` (a small JSON file per tag naming the model's layers by digest and size) and `blobs/sha256-<hex>` (the
layers, shared between models). The row gives six interests, each an item claim so the storage map keeps the real
folders: the store (holding `manifests` and `blobs`), the manifests' three folder levels (only to see whose they are)
and the manifests and the layers. The store's rows decide first, once a scan, before any manifest is read: no store or
no `manifests` and `blobs` is not claimed; an entry that is not the user's (a lock or a restricted one counts as not
the user's) or a partial download in `blobs/` leaves the store out whole (`not-yours`, `ollama-download`); then, and
only then, `finish` reads every manifest the scan matched through `cx.system().small_file` (at most 4 KiB, never a
link) and any manifest that cannot be read, or any entry at a host's, namespace's or model's depth that is not a folder
of a plain name (a link, a file, a hidden folder; a hidden file such as `.DS_Store` is ignored), leaves the store out
as `ollama-unreadable`, since which layers are shared cannot be told. A model is the manifests that name exactly the
same files (the tags of one model are one finding, tags in name order, the subject the first tag's manifest); its parts
are its manifests and the layers no manifest outside it names, and its size is what removing it frees. A layer another
model names never leaves with a model, even when that model is incomplete. A model with a missing or wrong-sized layer
is left out (`ollama-incomplete`); one with no known time or fewer than two parts (a tag whose every layer is shared)
or more than 64 is not claimed. The finding's identity is `ollama/<host>/<namespace>/<model>/<tag>` of its first tag, its
removal is `Removal::MoveTogether` of the manifests then the layers, and its reason says when it was last opened or
changed and, when layers are shared, how large the model is in all and with which model (the first by name, and how
many more) the rest stays. The cleaner checks every part before the first moves, so a model the server has loaded (its
layers are mapped) moves nothing ([`removal.md`](./removal.md)).

**`unused-toolchains` is code with a data table, and the example of a module that reads small files**: each manager is a
row of `toolchains.toml` (rustup, nvm: the versions' place under the home, the shape of a version's folder name, the
manager's own pointer places, the pin files and, for the shared `.tool-versions`, the keys it reads, the category and
the module's switch, if any), checked when the module is built. A version folder is a whole claim (summed, so the
storage map shows it), the manager's pointer files are item claims (rustup's `settings.toml` through a folder holding
it, nvm's alias files), and every pin file name is a marker claim found anywhere in the scanned folders (an item claim
makes nothing opaque). `finish` reads the files its matches showed, through `cx.system().small_file` (the pointer and
pin files, each at most 4 KiB, never a link, never kept or logged), and skips a pin inside a project's artifact folder
(`cx.system().projects()`: `node_modules/<dep>/.nvmrc` is a dependency's own file). The files' formats are read by pure
parsers (`pointers/`): every pointer is a reference that resolves to the version folders it keeps, to nothing, or to
"cannot tell", which blocks the manager's offers. The decision is a pure function (`decide.rs`, a table of twelve rows
checked in order): a row whose switch is off is decided with no pointers and no file of it is read; the manager's
default, a rustup moving channel, the newest installed Node version (a linked version folder counts as installed, as
nvm counts it) and any version a known reference keeps are never reported, and a version that would be offered but for
a blocking condition (a special entry, not the user's, a walk that did not read every folder, a default or a pin that
could not be read) is a `left_out` with its reason. Every category needs Desktop, Documents and Downloads
(`Need::required`), where projects live. The Node switch is off by default, and a case file's `[settings]` turns it
on in the sandbox. The removal is the version's folder moved to the Trash, never the manager's own command (nvm is a
shell function, and neither command can run in a sandbox). The other managers are
[deferred](../deferred/other-toolchain-versions.md), each for the fact it needs.

**`logs` is code, and the example of a module that claims a whole place by its contract**: the user's
`~/Library/Logs` is the place made for logs, so every regular file in it is claimed whatever its name, the second
exception of "Unknown is not junk" ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)). It
declares six `Claim::Item` interests, `Pattern::Under(KnownFolder::Logs, ...)` from one name to six names deep, so
no folder is made opaque and the walk keeps its shape. `on_match` makes one decision per file (the shape of `temp`:
`decide.rs`, a pure function of rows checked in order): a folder, a link, a state or lock file and what another module
claims whole (`JetBrains`, a folder shaped as a bundle id) are not claimed; a special entry, a file not the user's and a
locked file are left out with a reason, and so is a file changed within the setting's days. `Logs` is shared with
`jetbrains` and `app-leftovers`, whose whole claims hide their folders from this module; the split is refused by name
in `logs` as well, so it holds even if their claims change. The module asks for no installed or running apps: the
cleaner's open-file check protects a log in use.

A module may mix both: its own interests come after the rules' ones, so `matched.interest` still indexes the rules
first.

## Registration

One line in the app's composition root, `app/tauri/src/compose.rs`:

```rust
registry.register(Box::new(ExampleModule::new()?))?;
```

The registry refuses a module whose dictionary or settings are not valid, or whose id is taken.

## Tests

Through `fresher-testing` ([`../testing.md`](../testing.md)):

- `tests/contract.rs`: `assert_module_contract(&module, &Catalogue::Required(docs_dir))` (`Catalogue::DevOnly` for the
  dev-only `demo` and `trial` modules only), `assert_texts_used(&module, &runs)` over the runs of its fixture homes,
  and `check_removals_matched(&runs)` empty: every path a removal names is a path the module was matched on, so the
  engine can tell whether the app may remove it.
- The contract also fails a module whose run reports a failure or a left-out with a key it did not declare
  (`UndeclaredProblem`), a left-out in a category that is not its own (`ForeignLeftOutCategory`), or a template that
  names `{detail}` (`DetailInSentence`); a declared key without a text is `MissingText`.
- Fixture tests: `run_module(&module, &tree, &Ports::at(seconds))` on a `FixtureTree` → the findings, negative cases
  first; the golden report is `run.golden()` compared with `fixtures/golden/<id>.json`.

## The checklist

A module ships only when every box is ticked:

- [ ] The module names its group (`ModuleGroup`); a `Debug` module never ships (the registry refuses it in a
      release build, `RegistryError::DebugModule`).
- [ ] Every category is documented in [`../categories/`](../categories/README.md) as `<id>-<name>.md`, with the same
      danger level as its descriptor, and listed in the catalogue table.
- [ ] A module with an "unchanged for N days" setting uses `DaySetting`, not its own copy.
- [ ] Every category has its five texts; every setting, `<id>.enabled` included, its label and description.
- [ ] A module with choices gives each its two texts (`<id>.choice.<name>.label` and `.description`), and each choice
      names categories of the module only.
- [ ] Every text is plain English, short and without jargon; every choice says what the user keeps or loses (rule 7
      of [`../rules.md`](../rules.md)).
- [ ] Every failure and left-out text is declared with `with_problems`, plain, and says what to do next.
- [ ] An item the module recognized and does not offer is reported with `left_out`, never dropped.
- [ ] Every category that cannot see everything without a permission declares it (`with_needs`), and a needed tool has
      its `with_tool_check`.
- [ ] Negative fixtures come first: a look-alike name, the current version of a versioned item, a symlink out of the
      root; each is not claimed.
- [ ] The golden report `fixtures/golden/<id>.json` is written and reviewed.
- [ ] The contract test passes (`assert_module_contract`, `assert_texts_used`, no `check_removals_matched` problem).
- [ ] No dependency on another module (the dependency guard checks it).
- [ ] No danger level is lowered: a rule or a finding only raises its category's.
- [ ] Every removal is data (`MoveToTrash`, `EmptyFolder`, `MoveTogether`, `ToolCommand`); the module deletes nothing
      and walks nothing.
- [ ] A path removal names the finding's own subject (`MoveToTrash` or `EmptyFolder` of the `Subject::Path`); a
      `MoveTogether` of two to 64 paths (`Parts`, `MAX_PARTS`) has the subject first and every part a path the module
      was matched on, none twice, none inside another; a tool removal runs the subject's own tool and never forces
      (no `-f`, `--force`, `--force=…`, nor `f` among short flags such as `-rf`). The plan builder leaves any other
      out (`NotTheSubject`, `ForcedTool`; `crates/engine/src/cleanup/plan.rs`), so such a finding can never be
      removed.
