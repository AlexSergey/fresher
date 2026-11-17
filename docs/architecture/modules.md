# Writing a module

Every feature of the app is a module: one crate under `modules/<id>/`, named `fresher-module-<id>`, that depends on
`fresher-core` and, when it has rules, on `fresher-rules` — never on another module. A module never walks the disk and
never deletes: it says what it wants to see during the one shared walk, turns what the walk matched into findings, and
describes each removal as data. This page is the pattern every module plan follows; the `demo` module
(`modules/demo/`) is the smallest working example. Set up by plan
[002/06](../plans/002-base-architecture/06-module-kit/readme.md).

## The crate

```
modules/<id>/
├── Cargo.toml                 fresher-module-<id>; [lints] workspace = true; fresher-testing as a dev-dependency
├── rules.toml                 the rules, if the module has any (embedded with include_str!)
├── locales/en.json            every text the module shows, keys `<id>.…` only
├── src/lib.rs                 the descriptor and the `Module` implementation
├── src/settings.rs            the module's settings and reading them, if it has any
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
| `descriptor()`    | its id, categories (each with its danger level), settings and dictionary (`locales/en.json`)        |
| `interests()`     | what the walk should report to it: a folder name, a path under a known folder, a marker file or an extension |
| `on_match(…)`     | one match of one of its interests (`matched.interest` is the index in its own list) → findings     |
| `finish(…)`       | the end of the walk: work that needs every match (the newest version), or that reads no disk (a tool) |

- A category id is `<id>.<name>`; its five texts are found by convention: `<id>.<name>.name`, `.what`, `.made-by`,
  `.if-removed`, `.comes-back`.
- A setting key is `<id>.<name>`, with `<id>.<name>.label` and `.description`. The settings schema adds the switch
  `<id>.enabled` for every module; the module ships its label and description too.
- `NewFinding::new(&category, …)` starts at the category's danger level; `raise_danger` only goes up. A module that
  knows which app a finding belongs to says so with `for_app(bundle_id)`: the storage map counts the finding in that
  app's line (an id no installed app has is shown as an app no longer installed).
- The ports a module may use come in `ModuleContext`: the clock, the command runner (an external tool, with a fixed
  argument list; pass `cx.cancel()`, so a cancelled scan stops the tool), the system's facts (`cx.system()`, below),
  its settings, the cancel token. A long `finish` checks `cx.cancel()` and returns `ModuleError::Cancelled`.
- **A module never writes an OS path** (the user's decision of 2026-11-15): a place is a known folder and a relative
  path, `Pattern::Under(KnownFolder::AppCaches, "JetBrains/*")` (in rules, `under = { folder = "app-caches", path =
  "JetBrains/*" }`); `KnownFolder::Home` is the home folder itself. The walk resolves the folder through the
  `KnownFolders` port; a folder this system does not have matches nothing.

## The system's facts (`cx.system()`)

Plan [009](../plans/009-system-knowledge/readme.md). One `SystemFacts` per scan, shared by every module; each fact is
gathered on the first ask and kept for the scan, a rescan gathers it anew, nothing is written.

| Ask                          | Answers                                                                                           |
| ---------------------------- | ------------------------------------------------------------------------------------------------- |
| `locate(KnownFolder)`        | where this system keeps the folder                                                                |
| `installed_apps()`           | every installed app: bundle id, team id, name, version, build, path, its helpers' bundle ids      |
| `running_apps()`             | the running apps (`RunningSet`); `owner_of(path)` names the running app a path is named after     |
| `projects()`                 | the projects the walk found (root, kinds, artifact folders, last activity); `None` during the walk |

- An app is identified by its bundle id and team id only (`BundleId` compares ignoring case), never by a loose name.
  `BundleId::names(name)` tells whether a folder's or a file's name is named after the id: the whole name, or the id
  followed by a dot (`com.example.App.savedState`), never `com.example.Appx`.
- At the commit the engine lists the installed apps too (once per scan, even when no module asked), to attribute each
  app's data folders and its bundle in the storage map.
- The projects are known in `finish`, not in `on_match`. A project's last activity is its folder's own modification
  time for now ([deferred](../deferred/project-last-activity.md)).
- An item a running app owns is skipped by the cleaner anyway; a module may raise a finding's danger or say "quit the
  app first" with `running_apps()`.
- In tests, `run_module`'s `Ports` carry `apps: FakeInstalledApps`, `running: FakeRunningApps` and the known folders
  (by default laid out under the fixture root); `installed_app("com.example.Editor", "Editor", "2026.3")` makes one
  up.
- A module that returns an error, or panics, ends `ModuleFinished { Failed }` alone: the engine catches the panic
  (`catch_unwind`; the release profile unwinds), the other modules finish and the scan ends `Partial`.

## What a module sees of the walk

- Only the matches of its own interests, one `Match` each: `interest` (the index in its own `interests()`), `path`,
  `kind`, `modified`, `siblings` (which of the interest's sibling names exist next to the entry) and `marker` (for a
  `CACHEDIR.TAG` marker, whether the tag holds its signature: `Valid` or `Invalid`; `None` for every other interest).
  The module never reads the file itself.
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
- Fixture tests: `run_module(&module, &tree, &Ports::at(seconds))` on a `FixtureTree` → the findings, negative cases
  first; the golden report is `run.golden()` compared with `fixtures/golden/<id>.json`.

## The checklist

A module ships only when every box is ticked:

- [ ] Every category is documented in [`../categories/`](../categories/README.md) as `<id>-<name>.md`, with the same
      danger level as its descriptor, and listed in the catalogue table.
- [ ] Every category has its five texts; every setting, `<id>.enabled` included, its label and description.
- [ ] Every text is plain English, short and without jargon; every choice says what the user keeps or loses (rule 7
      of [`../rules.md`](../rules.md)).
- [ ] Negative fixtures come first: a look-alike name, the current version of a versioned item, a symlink out of the
      root; each is not claimed.
- [ ] The golden report `fixtures/golden/<id>.json` is written and reviewed.
- [ ] The contract test passes (`assert_module_contract`, `assert_texts_used`, no `check_removals_matched` problem).
- [ ] No dependency on another module (the dependency guard checks it).
- [ ] No danger level is lowered: a rule or a finding only raises its category's.
- [ ] Every removal is data (`MoveToTrash`, `EmptyFolder`, `ToolCommand`); the module deletes nothing and walks nothing.
- [ ] A path removal names the finding's own subject (`MoveToTrash` or `EmptyFolder` of the `Subject::Path`); a tool
      removal runs the subject's own tool and never forces (no `-f`, `--force`, `--force=…`, nor `f` among short
      flags such as `-rf`). The plan builder leaves any other out (`NotTheSubject`, `ForcedTool`;
      `crates/engine/src/cleanup/plan.rs`), so such a finding can never be removed.
