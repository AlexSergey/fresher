# 016 — JetBrains IDE leftovers: design

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, core, sandbox, testing, app, app/ui, docs

Written on 2026-11-17 and approved the same day by delegation: the user asked the agent to work on its own and accept
the recommended decisions. Every open question is answered with the recommended option and recorded under
"Decisions"; the agent's calls wait for the user's check — the layout of the IDEs' folders and their bundle ids are
written from knowledge and are checked by hand on a real Mac (04/02).

## Intent

The user's request of 2026-11-17, translated: "Let's do the JetBrains module after 011. Put into this module's plan
right away the ways to test it in the sandbox and coverage by e2e scenarios. Possibly a separate plan for ignoring a
module (JetBrains, and a general group 'Development' into which JetBrains should be placed)."

This is the case that started the project ([`goal.md`](../../goal.md)): an IDE updates itself, and the settings,
caches, logs and plugins of every previous version stay on disk, taking gigabytes, and nothing points at them. It is
the first real feature module. The separate plan is [015](../015-module-groups/readme.md) (groups and ignoring a
module), built before this one.

## Goals

1. Find the data folders of **older versions** of JetBrains IDEs — settings with their plugins, caches, logs — and
   offer them, each with its danger level, its plain-English texts and its app; and the data of an IDE that is **no
   longer installed**, in its own category, once unchanged for the days set in Settings (decision 10).
2. **Never offer the version in use, a newer one, or anything of an IDE that is open**; say why with a left-out line
   (plan 011), never silently.
3. Ship the module's **tests at every level the user asked for**: unit tests of the version and product parsing,
   fixture tests, the sandbox case file with its world (installed and running apps), the core e2e and the UI e2e
   scenarios, and a run by hand in the sandbox and on the real Mac.
4. Build the part of the deferred [sandbox world](../../deferred/sandbox-world.md) this module needs: installed apps
   and running apps per case file.

## Non-goals

- **The leftovers of other removed apps**: the [uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md)
  idea, with its own module. A removed JetBrains IDE is this module's (decision 10, which replaces decision 3).
- **The old layout before the 2020 versions** (`~/Library/Preferences/WebStorm2019.3`, `~/Library/Caches/WebStorm2019.3`
  with no vendor folder): left alone (decision 5); written down as an idea follow-up in the finish.
- **Other vendors' IDEs on the same platform** (Android Studio under `Google/`), **JetBrains Toolbox's own builds kept
  for rollback**, **remote development clients** (`JetBrainsClient<version>`, `Gateway<version>`): not recognized, so
  not claimed (rule 3: they stay in your folders).
- **Project-level folders** (`.idea/`), and the **caches of the version in use** (an IDE's own "Invalidate caches"
  does that better).
- The rest of the sandbox world: open files, tools' answers, permissions, a case's own clock and the steps between
  scans stay [deferred](../../deferred/sandbox-world.md); the sandbox's clock starts at the generator's moment (02/04).

## What already exists

| Need                                 | What exists                                                                                                 | Plan |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------- | ---- |
| A module crate and its checks        | `modules/<id>/`, `ModuleDescriptor`, `assert_module_contract`, `run_module`, golden reports ([`architecture/modules.md`](../../architecture/modules.md)) | 002 |
| A place under a known folder         | `Pattern::Under(KnownFolder::AppSupport \| AppCaches \| Logs, "JetBrains/*")`; `Claim::Whole` sums a folder  | 002, 004 |
| Installed apps                       | `cx.system().installed_apps()`: bundle id, team id, name, version, path ([`modules.md`](../../architecture/modules.md), "The system's facts"); the Toolbox folders and `installed_apps_complete()` are added by 02/04 | 009 |
| Running apps                         | `cx.system().running_apps()`: name and bundle ids                                                           | 009 |
| A finding's app                      | `NewFinding::for_app(bundle_id)`: the storage map counts it in the app's line                               | 012 |
| Saying why something is not offered  | `out.left_out(LeftOutItem { category, reason, sizes })`, `ModuleDescriptor::with_problems(keys)`, the `detail` rule | 011 |
| A group and ignoring a module        | `ModuleGroup::Development`, `IgnoreTarget::{Module, Group}`                                                 | 015 |
| Case files and their autotest        | `fixtures/sandboxes/<feature>.toml`, `[case.expect]`, `[case.map]`, `npm run sandbox`, `dev:sandbox`, `sandbox:test` ([`testing.md`](../../testing.md), "Sandboxes") | 007, 012 |
| Apps in a sandbox                    | the sandbox has `Applications/` and `System/Applications/`; `MacInstalledApps` reads them through the known folders; running apps are real in sandbox mode, empty in the autotest | 007 |
| The core e2e and the UI e2e          | `crates/e2e` (`cleanup.rs`, `ignore.rs`), Playwright on a mocked core (`e2e/core-mock.ts`, `flows.spec.ts`)  | 003, 008 |

## Design

### 1. Where the IDEs keep their data (to check by hand, agent's call 1)

Since the 2020.1 versions, on macOS, each IDE keeps one folder per version, named `<Product><Year>.<N>`, in three
places:

| Place                                              | Known folder | What it holds                                                                                 | Category              |
| -------------------------------------------------- | ------------ | --------------------------------------------------------------------------------------------- | --------------------- |
| `~/Library/Application Support/JetBrains/<P><V>`   | `AppSupport` | settings (`options/`, keymaps, code styles, live templates…) and `plugins/` installed for it  | `jetbrains.settings`  |
| `~/Library/Caches/JetBrains/<P><V>`                | `AppCaches`  | indexes, the compile cache, local history, downloaded sources                                  | `jetbrains.caches`    |
| `~/Library/Logs/JetBrains/<P><V>`                  | `Logs`       | the IDE's logs                                                                                | `jetbrains.logs`      |

The same vendor folders hold names that are not versions (`Toolbox`, `consentOptions`, `PrivacyPolicy`, `bl`, `crl`,
`JetBrainsClient2025.1`, …): none is recognized, so none is claimed.

### 2. Products, as data (rule 5)

`modules/jetbrains/products.toml` lists each product: the name its folders start with, the name shown, and its bundle
ids (the release and the early access build). Embedded with `include_str!`, parsed and checked at `new()`.

```toml
[[product]]
folder = "WebStorm"
name = "WebStorm"
bundle-ids = ["com.jetbrains.WebStorm", "com.jetbrains.WebStorm-EAP"]
```

The first table (agent's call 2, each id checked by hand on an installed IDE where possible):

| Folder        | Shown as                      | Bundle ids                                          |
| ------------- | ----------------------------- | --------------------------------------------------- |
| `IntelliJIdea`| IntelliJ IDEA                 | `com.jetbrains.intellij`, `com.jetbrains.intellij-EAP` |
| `IdeaIC`      | IntelliJ IDEA Community Edition | `com.jetbrains.intellij.ce`, `com.jetbrains.intellij.ce-EAP` |
| `WebStorm`    | WebStorm                      | `com.jetbrains.WebStorm`, `com.jetbrains.WebStorm-EAP` |
| `PyCharm`     | PyCharm                       | `com.jetbrains.pycharm`, `com.jetbrains.pycharm-EAP` |
| `PyCharmCE`   | PyCharm Community Edition     | `com.jetbrains.pycharm.ce`, `com.jetbrains.pycharm.ce-EAP` |
| `PhpStorm`    | PhpStorm                      | `com.jetbrains.PhpStorm`, `com.jetbrains.PhpStorm-EAP` |
| `GoLand`      | GoLand                        | `com.jetbrains.goland`, `com.jetbrains.goland-EAP`  |
| `CLion`       | CLion                         | `com.jetbrains.CLion`, `com.jetbrains.CLion-EAP`    |
| `RubyMine`    | RubyMine                      | `com.jetbrains.rubymine`, `com.jetbrains.rubymine-EAP` |
| `Rider`       | Rider                         | `com.jetbrains.rider`, `com.jetbrains.rider-EAP`    |
| `DataGrip`    | DataGrip                      | `com.jetbrains.datagrip`, `com.jetbrains.datagrip-EAP` |
| `DataSpell`   | DataSpell                     | `com.jetbrains.dataspell`, `com.jetbrains.dataspell-EAP` |
| `RustRover`   | RustRover                     | `com.jetbrains.rustrover`, `com.jetbrains.rustrover-EAP` |
| `AppCode`     | AppCode                       | `com.jetbrains.AppCode`, `com.jetbrains.AppCode-EAP` |
| `Aqua`        | Aqua                          | `com.jetbrains.aqua`, `com.jetbrains.aqua-EAP`      |
| `Writerside`  | Writerside                    | `com.jetbrains.writerside`, `com.jetbrains.writerside-EAP` |

Every product has its early access id with `-EAP` (added on 2026-11-17, in 02/02). The table also lists the vendor's
apps that are not IDEs and keep no version folders, `other-apps = ["com.jetbrains.toolbox"]`: one of them running
keeps nothing (row 3 of section 4).

Checked at `new()`: folder names unique, plain letters only, none empty; every bundle id valid (`BundleId::new`) and in
one product (or the other apps) only; at least one id per product. A broken table is a `JetBrainsError` (no panic).

### 3. Reading a folder's name

`parse(name) -> Option<(Product, Version)>`, pure, in `src/name.rs`:

- The name is **exactly** a product's `folder`, then `<Year>.<N>`: four digits starting `20`, a dot, one or two digits
  with no leading zero. Nothing before, nothing after.
- The product is the **exact** prefix: `PyCharmCE2024.1` is `PyCharmCE`, never `PyCharm` with `CE2024.1`; the prefix is
  compared case-sensitively (the IDEs write these names; `webstorm2024.1` is not theirs).
- Refused, so never claimed: `WebStorm2024` (no minor), `WebStorm2024.3.1`, `WebStorm2024.3.bak`, `WebStorm2024.3 copy`,
  `WebStorm-2024.3`, `WebStorm.2024.3`, `WebStorm2024.03`, `WebStorm1999.1`, `FooStorm2024.1`, `JetBrainsClient2025.1`.
- `Version { year, minor }`, ordered by year then minor (`2025.10` after `2025.9`).
- An installed app's version is read the same way from the start of its version string (`2025.3.1` → 2025.3,
  `2026.1 EAP` → 2026.1); a string that does not start that way is an unknown version.

### 4. The decision

The module's interests, in order: `Under(AppSupport, "JetBrains/*")`, `Under(AppCaches, "JetBrains/*")`,
`Under(Logs, "JetBrains/*")`, each `Claim::Whole`. `on_match` takes only a match whose entry is a **folder** (a link
or a file is never claimed) and whose name parses, and decides it at once (code, not rules: the answer depends on the
installed and running apps, which a rule cannot say; agent's call 3) — but for row 4, whose answer depends on every
folder of the product: those folders are kept for the scan's `finish` (agent's call 12, changed on 2026-11-17 in
02/04). So the walk's order never matters, and a scan cancelled before its `finish` leaves nothing to the next one.

For each product with folders, the first answer that applies, for **every** folder of the product:

| # | Situation                                                                    | Result                                             |
| - | ---------------------------------------------------------------------------- | -------------------------------------------------- |
| 1 | the installed or the running apps cannot be listed; the installed list is partial | left out: `jetbrains.left-out.no-system`; `jetbrains.left-out.apps-unread` |
| 2 | an app with one of the product's bundle ids is **running**                   | left out: `jetbrains.left-out.running`             |
| 3 | an app of the vendor (`com.jetbrains.…`) that the table does not list is **running** | left out: `jetbrains.left-out.vendor-app-open` |
| 4 | no installed app has one of the product's bundle ids                          | left out in `jetbrains.uninstalled`: `jetbrains.left-out.unknown-ide` when an unknown app of the vendor is installed; else decided in `finish`, for every folder of the product: **offered** in `jetbrains.uninstalled` when the scan is walked (not restored) and read every folder, none changed in the setting's days and Spotlight knows no app of the product; else left out there (`used-recently`, `installed-elsewhere`, or `spotlight-unread` when Spotlight cannot be asked or its index is off) |
| 5 | an installed app of the product has a version that cannot be read            | left out: `jetbrains.left-out.unknown-version`     |
| 6 | the folder's version is one an installed app of the product has              | left out: `jetbrains.left-out.current`             |
| 7 | the folder's version is newer than the newest installed one                  | left out: `jetbrains.left-out.newer`               |
| 8 | the folder's version is older than the newest installed one                   | **offered** in the place's category                |

- **An IDE that is no longer installed** (row 4, the user's decision 10 of 2026-11-17, built in 02/04): rows 1 to 3
  already hold — the installed list is complete (an apps folder, a folder in one or a bundle the reader could not read
  makes it partial: row 1; the reader also looks in `~/Applications/JetBrains Toolbox/` and the older Toolbox's
  `~/Library/Application Support/JetBrains/Toolbox/apps/`, agent's call 6), nothing of the product runs, and no
  unknown app of the vendor runs. Every folder of the product in the three places is then offered in
  `jetbrains.uninstalled` when the newest of their own modification times is at least the setting's days before now
  (`jetbrains.uninstalled-after-days`, 30 by default, 7 to 3650); a folder whose time is unknown or in the future
  counts as changed now. Otherwise every folder of the product is left out there with `used-recently`. "Now" is the
  clock port's: the system's in the app, the sandbox's moment in sandbox mode (section 7), a fake in tests.
  The controller's rulings of 2026-11-17 (review of 02/04), each closing a path by which a live IDE's data could be
  offered: an app in the apps folders that is a **link** is read where it points (an unreadable target makes the
  list partial), and a folder link or another volume's folder among them makes it partial; an **installed** unknown
  app of the vendor (not only a running one) keeps the product's folders (`unknown-ide`); a **restored** scan
  (`ModuleContext::is_restored`) never offers, since its times are the last walk's (`used-recently`); and before
  offering, the module asks **Spotlight** (`SystemFacts::find_apps`: `mdfind` by each bundle id of the product,
  through the command runner in `fresher-system`; in a sandbox, the sandbox's own apps): an existing hit keeps
  everything (`installed-elsewhere`), a failure keeps everything (`spotlight-unread`; first `no-system`, changed by
  the final review of 2026-11-17). The final review's rulings of 2026-11-17 add: Spotlight is first asked for the
  Finder, and no answer counts as a failure (an index that is off answers nothing for anything); its paths are read
  NUL-separated (`mdfind -0`); a walk that could not read every folder (`ModuleContext::is_walk_complete`, known in
  `finish`) never offers (`used-recently`), since an unseen folder of the product may have changed; a partial
  installed list gives `apps-unread`, not `no-system`, which stays for a list that failed. An IDE on a volume that
  Spotlight does not index is not found (docs/known-issues, "Spotlight does not see an IDE on a volume it does not
  index").
- **Apps deeper than the reader looks** (the final review of 2026-11-17): the reader looks in each apps folder and one
  folder below (two levels), so an app two folders below `/Applications` (`/Applications/Tools/JetBrains/X.app`) is
  not seen. A newer copy there leaves the product counted as not installed or at the version the reader does see; an
  older copy there makes its folders look older than the version in use, and they are offered as older versions
  (Medium for the settings), which the user confirms, and which go to the Trash.
- **An unknown app of the vendor** (row 3, the controller's ruling of 2026-11-17): a running app whose bundle id starts
  `com.jetbrains.` (case ignored) and is neither a product's nor one of the table's other apps may be an IDE the table
  does not know yet, reading or about to import any product's folders; every product's folders are kept while it runs,
  with its own reason, `vendor-app-open` (first the running reason; changed by the final review of 2026-11-17). The Toolbox app, which runs all day for many users and keeps no
  version folders, is listed as an other app so it does not block every offer.
- **Two installed versions side by side** (a release and an early access build) are both in use (row 6); only what is
  older than the newest of them and is not one of them is offered.
- **The newest version present** is used only through row 7's comparison: a folder newer than every installed app is
  kept (a newer IDE on a drive the app cannot see, or a removed early access build). Without an installed app, the
  newest present is not trusted to name the version in use (decision 3).
- **A running IDE** keeps every folder of its product, the older ones too: a just-updated IDE reads the older folder
  to import the settings at its first start (decision 4).
- **The finding's app** is the newest installed app of the product; of two apps of the same version, the first the
  installed apps list gives.
- The **team id** is not checked: a fake app with a real IDE's bundle id can only change which versions are kept or
  offered among the IDE's own folders, never claim another app's data (agent's call 4).

A finding:

| Part      | Value                                                                                                          |
| --------- | -------------------------------------------------------------------------------------------------------------- |
| category  | `jetbrains.settings` (Medium), `jetbrains.caches` (Safe) or `jetbrains.logs` (Safe), by place                    |
| identity  | `<place>/<folder name>` (`caches/WebStorm2024.3`)                                                              |
| subject   | `Subject::Path` of the folder; removal `Removal::MoveToTrash` of the same path                                  |
| sizes     | the match's (the whole folder)                                                                                 |
| title     | `jetbrains.finding.title`: "{product} {version}"                                                               |
| reason    | `jetbrains.finding.reason`: "Left by {product} {version}. You use {current} now."                                |
| app       | `for_app`: the bundle id of the newest installed app of the product, so the storage map shows the IDE's line   |

A finding of an IDE that is no longer installed differs in:

| Part      | Value                                                                                                          |
| --------- | -------------------------------------------------------------------------------------------------------------- |
| category  | `jetbrains.uninstalled` (Medium), whatever the place                                                            |
| identity  | `<place>/<folder name>` (`settings/DataGrip2024.3`), unique in the category since a folder name is in one place once |
| title     | by place: `jetbrains.finding.uninstalled-settings` "{product} {version} settings and plugins", `-caches` "{product} {version} caches", `-logs` "{product} {version} logs" |
| reason    | `jetbrains.finding.uninstalled-reason`: "Left by {product} {version}. {product} is no longer installed, and none of its data has changed in the last {days} days." |
| app       | `for_app`: the product's release bundle id (the first of the table's); the storage map names a removed app by its id (agent's call 16) |

A left-out holds the category, the reason and the folder's sizes (plan 011: no path, no args).

### 5. Categories and texts

| Category              | Name (shown)                                     | Danger     | Why this level                                                                                                     |
| --------------------- | ------------------------------------------------ | ---------- | ------------------------------------------------------------------------------------------------------------------ |
| `jetbrains.caches`    | Caches of older JetBrains IDE versions           | **Safe**   | only that older version reads them; it rebuilds them if it is ever opened again                                    |
| `jetbrains.logs`      | Logs of older JetBrains IDE versions             | **Safe**   | nothing reads them; they are never needed again                                                                    |
| `jetbrains.uninstalled` | Data of JetBrains IDEs that are no longer installed | **Medium** | settings and plugins are among them and cannot be made again; nothing installed uses any of it, and the setting's days without a change guard an IDE the app could not see |
| `jetbrains.settings`  | Settings and plugins of older JetBrains IDE versions | **Medium** | the newer version copied the settings when it first started, but if it did not (the copy was declined, or the newer version was never opened), custom keymaps, code styles and templates are lost; the plugins would download again (Low) — one folder holds both, so the higher level wins (decision 2) |

The five texts of each (what it is, made by, if removed, comes back) follow rule 7, for example for the settings:
"What it is: the settings and plugins an older version of a JetBrains IDE used. If removed: they go to the Trash; your
current version keeps its own settings. Comes back: no; if you install that older version again, it starts with
default settings." For the data of an IDE that is no longer installed: "If removed: they go to the Trash. If you
install that IDE again, it starts from scratch: default settings, no plugins, and its caches are built again. Comes
back: no; only that IDE would make them again, if you installed it and opened it." Each category gets its catalogue
file, `docs/categories/jetbrains-<name>.md`, and a row in the catalogue, in the step that adds it (02/01; 02/04 for
`jetbrains.uninstalled`).

The module's switch: `jetbrains.enabled.label` "JetBrains IDEs", `.description` "Finds what JetBrains IDEs left
behind: the settings, caches and logs of older versions, and the data of an IDE that is no longer installed." Its one
setting, `jetbrains.uninstalled-after-days` (a number of days, 7 to 3650, 30 by default, applied at the next scan):
`.label` "Offer the data of a removed IDE after", `.description` "The data of a JetBrains IDE that is no longer
installed is offered only once none of it has changed for this many days, at least 7." The left-out reasons (declared with `with_problems`), in plain
English with the next step where there is one:

| Key                                  | Text                                                                                                |
| ------------------------------------ | --------------------------------------------------------------------------------------------------- |
| `jetbrains.left-out.current`         | This is the version you use.                                                                        |
| `jetbrains.left-out.newer`           | It belongs to a newer version than the one installed, so it is kept.                                |
| `jetbrains.left-out.running`         | The IDE is open. Quit it and scan again to clean its older versions.                                |
| `jetbrains.left-out.unknown-version` | Fresher could not tell which version of this IDE you use, so it keeps every version of it.          |
| `jetbrains.left-out.no-system`       | Fresher could not list your apps, so it keeps every IDE version. Scan again; if this goes on, restart your Mac. |
| `jetbrains.left-out.used-recently`   | Fresher could not find this IDE installed, but its data changed within the days set in Settings, so it is kept; a result restored from the last scan, or a scan that could not read every folder, keeps it too. A new scan after those days offers it. |
| `jetbrains.left-out.unknown-ide`     | Another JetBrains app that Fresher does not know is installed and may use this data, so it is kept. |
| `jetbrains.left-out.installed-elsewhere` | This IDE is installed outside your apps folders, so Fresher keeps its data.                     |
| `jetbrains.left-out.vendor-app-open` | Another JetBrains app is open and may use this data. Quit it and scan again to clean older versions. |
| `jetbrains.left-out.apps-unread`     | Fresher could not read every app in your apps folders, so it keeps every IDE version.               |
| `jetbrains.left-out.spotlight-unread` | Fresher could not ask Spotlight whether this IDE is installed elsewhere on your Mac, so it keeps its data. If Spotlight is turned off, turn it on and scan again. |

`jetbrains.left-out.not-installed` was removed on 2026-11-17 (02/04): no row gives it any more. The last three keys
were added by the final review of 2026-11-17, so each case has its own words; `no-system` is left only for a list of
apps that failed.

### 6. The module's group

`ModuleGroup::Development` (plan 015). The module is registered in every build, release included, in
`app/tauri/src/compose.rs`'s `modules()`, before the dev-only ones.

### 7. The sandbox world this module needs

Taken from the deferred item (its parts 1 and the running part of 2; decision 6). The rest stays deferred.

**Installed apps on disk** — a case may declare apps; the generator writes each as a bundle with
`Contents/Info.plist` (an XML property list: `CFBundleIdentifier`, `CFBundleName`, `CFBundleShortVersionString`,
`CFBundleVersion`, `CFBundlePackageType` `APPL`), so plan 009's real `MacInstalledApps` reads the sandbox's apps
folders as it reads a Mac's. The team id stays `None` (an unsigned bundle).

```toml
[[case.app]]
path = "Applications/WebStorm.app"        # in Applications/, System/Applications/ or home/Applications/
bundle-id = "com.jetbrains.WebStorm"
name = "WebStorm"
version = "2025.3.1"                      # optional: a bundle with no version string
build = "WS-253.1"                        # optional
running = true                            # optional, false by default: the app is open
```

**Running apps** — an app declared `running = true`, and an app that runs from no apps folder:

```toml
[[case.running]]
name = "CLion"
bundle-ids = ["com.jetbrains.CLion"]
```

- The generator writes the declared running apps into `world.toml` at the sandbox's root (its own file, reset like
  `EXPECTED.md`); the autotest and sandbox mode read it into `FakeRunningApps`. Since 02/04 it always writes it, with
  `now`, the generator's moment in seconds, which every `age` of the case files counts back from: sandbox mode's clock
  starts there and runs at the system's pace (`StartedClock`), the autotest's stands there, so a folder 40 days old
  stays 40 days old however long ago the sandbox was generated. A `[[case.dir]]` may carry `age` too (set after its
  contents, so the folder keeps it). **Sandbox mode always takes the
  running apps from `world.toml`** (none when the file is missing) and never the real Mac's, so a real open IDE never
  changes what a sandbox shows (decision 7); the banner adds "Open apps are simulated."
- Checks, when read: a bundle id that is not reverse-DNS, an app path not ending in `.app` or outside an apps folder,
  two apps at one path, a running entry with no bundle id are refused. `MacInstalledApps` over a generated sandbox lists
  exactly the declared apps.
- The sandbox's protection list already protects its `Applications/` and `System/Applications/` (007); the generator
  refuses a case file that writes `home/Applications/` files other than apps.
- `EXPECTED.md` lists a case's apps ("Installed: WebStorm 2025.3.1 (open)").

**The left-out expectation** (plan 011's agent's call 6: the first module that reports a left-out adds it):

```toml
[[case.left-out]]
category = "jetbrains.caches"
reason = "jetbrains.left-out.running"
items = 2
```

On any case. The autotest sums the expectations per (category, reason) over the case file and compares them **exactly**
with the module's `ModuleFinished.left_out` counts (an unexpected left-out fails too); the reason must be one the module
declared. `EXPECTED.md` adds "Under '<category name>', <items> left out: <reason's text>".

### 8. Tests

The user asked for these explicitly; each is a step's check.

**Unit** (`modules/jetbrains/src/{name,products,decide}.rs`, negatives first):

- name parsing: the refused names of section 3, a name with a product's prefix but another product's suffix, the
  empty name, a name of non-UTF-8 bytes, a year of three or five digits, a minor of `0` or `00`; then each product's
  name parsed, `PyCharmCE` versus `PyCharm`, `IdeaIC` versus `IntelliJIdea`.
- version order: `2025.10` after `2025.9`, year before minor; a property test (proptest) that the order is the order of
  `(year, minor)` and that parsing what `Display` writes gives the same version.
- installed version strings: `2025.3.1`, `2025.3`, `2026.1 EAP`, `253.1234` (unknown), empty (unknown).
- the products table: a duplicate folder, a bundle id in two products, an invalid id, an empty id list are refused;
  the shipped table parses.
- the decision table of section 4, row by row, on plain values (no disk): each row's negative before the offered case;
  the precedence (running wins over not installed and current); an unknown app of the vendor running keeps the folders,
  the Toolbox app and another product's app do not.
- the data of an IDE that is no longer installed (02/04): installed through the Toolbox, a partial list, the IDE or an
  unknown vendor app running, a folder changed 29 days ago, an unknown time, a longer setting, a cancelled scan; then
  the three folders offered, a shorter setting; a property test of the age (old exactly when every folder is old
  enough, whatever the order).

**Fixture tests** (`modules/jetbrains/tests/fixtures.rs`, through `run_module` with `FakeInstalledApps` and
`FakeRunningApps`): `fixtures/homes/jetbrains-*.toml`, negative homes first — `jetbrains-current.toml` (only the
installed version), `jetbrains-similar-names.toml` (every refused name of section 3, a version folder outside
`JetBrains/`, a file named like a folder, a link to a folder outside the root), `jetbrains-running.toml`,
`jetbrains-uninstalled-fresh.toml` (a removed IDE, one folder changed 5 days before `FIXTURE_NOW`); then
`jetbrains-stale.toml` (three versions in three places, two installed side by side, an EAP bundle id) and
`jetbrains-uninstalled.toml` (a removed IDE's three folders, `[[dir]]` `age = "40d"`). Golden report `fixtures/golden/jetbrains.json`. A test that hands the same matches in two
orders gives the same findings. `tests/contract.rs`: `assert_module_contract` with `Catalogue::Required`,
`assert_texts_used` over every fixture run, `check_removals_matched` empty.

**Sandbox** (`fixtures/sandboxes/jetbrains.toml`, `feature = "jetbrains"`, `module = "jetbrains"`; golden
`fixtures/golden/sandbox-jetbrains.json`). Every case lives under `home/Library/{Application Support,Caches,Logs}/JetBrains/`
or the apps folders. Negatives first:

| Case (negative)                                                         | Apps and files                                                                                       | Expected                                                          |
| ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| the version you use is never offered                                    | WebStorm 2025.3.1 installed; `WebStorm2025.3` in the three places                                     | left out `current` ×1 in each category; `[case.map]` your folders |
| two versions installed side by side are both kept                       | IntelliJ IDEA 2025.2 and an early access build 2025.3 (`com.jetbrains.intellij-EAP`); their folders  | left out `current` ×2 (settings, caches)                          |
| a newer version than the installed one is kept                          | `WebStorm2026.1` caches (WebStorm 2025.3 installed)                                                  | left out `newer` ×1                                               |
| an open IDE keeps every version                                         | GoLand 2025.2 installed and open; `GoLand2024.3` and `GoLand2025.2` settings and caches               | left out `running` ×2 settings, ×2 caches                         |
| an open IDE outside the apps folders keeps every version                | CLion running, not installed; `CLion2024.1` caches                                                   | left out `running` ×1                                             |
| a removed IDE whose data changed recently keeps every version           | `PhpStorm2024.2` settings 40 days old, `PhpStorm2025.1` settings 5 days old (`[[case.dir]]` ages), no app | left out `used-recently` ×2 under `jetbrains.uninstalled`  |
| a removed IDE installed by the Toolbox app is not removed               | DataGrip 2025.3 in `home/Applications/JetBrains Toolbox/DataGrip.app`; `DataGrip2025.3` settings 40 days old | left out `current` ×1                                     |
| a removed IDE keeps its data while an unknown app of the vendor runs    | `[[case.running]]` `com.jetbrains.gateway`; `RustRover2024.2` caches 40 days old, no app              | left out `running` ×1                                             |
| the Community edition is not the full edition                           | PyCharm 2025.2 installed; `PyCharmCE2024.1` caches, fresh                                            | left out `used-recently` ×1 under `jetbrains.uninstalled` (never PyCharm's) |
| an app with no version string keeps every version                       | RubyMine installed with no `version`; `RubyMine2024.1` caches                                        | left out `unknown-version` ×1                                     |
| names that only look like versions are never claimed                    | `WebStorm2024`, `WebStorm2024.3.1`, `WebStorm2024.3.bak`, `webstorm2024.2`, `WebStorm-2024.1` caches | nothing offered, no left-out; your folders                        |
| unknown products and the vendor's other folders are never claimed       | `FooStorm2024.1`, `JetBrainsClient2025.1`, `Toolbox/`, `consentOptions/` in Application Support      | nothing offered, no left-out                                      |
| a version folder outside the vendor folder is never claimed             | `home/Library/Caches/WebStorm2023.1/`, `home/Library/Caches/JetBrainsX/WebStorm2023.1/`              | nothing offered                                                   |
| a file named like a version folder is never claimed                     | a file `Logs/JetBrains/WebStorm2023.2`                                                               | nothing offered                                                   |
| a link named like a version folder is never followed                    | `Caches/JetBrains/WebStorm2023.3` → a folder outside the sandbox                                     | nothing offered; the target untouched                             |

Then the positives (WebStorm 2025.3.1 installed by the first negative case's app):

| Case (positive)                                            | Files                                                                    | Expect                                                    |
| ---------------------------------------------------------- | ------------------------------------------------------------------------ | --------------------------------------------------------- |
| caches of an older version go to the Trash                 | `Caches/JetBrains/WebStorm2024.3/` (index files, `fill = "data"`)        | `jetbrains.caches`, safe, `in-trash`; map explained       |
| logs of an older version go to the Trash                   | `Logs/JetBrains/WebStorm2024.3/idea.log`                                 | `jetbrains.logs`, safe, `in-trash`                        |
| settings and plugins of an older version go to the Trash   | `Application Support/JetBrains/WebStorm2024.3/{options,plugins}/…`       | `jetbrains.settings`, medium, `in-trash`; map explained   |
| an older version than both side-by-side installs is offered | `Caches/JetBrains/IntelliJIdea2024.3/`                                  | `jetbrains.caches`, safe, `in-trash`                      |
| an IDE installed in the user's own apps folder counts      | Rider 2025.3 in `home/Applications/Rider.app`; `Caches/JetBrains/Rider2025.1/` | `jetbrains.caches`, safe, `in-trash`                |
| the data of a removed IDE goes to the Trash                | no DataGrip app; `DataGrip2024.3` in the three places, each folder 40 days old (`[[case.dir]]` `age = "40d"`) | `jetbrains.uninstalled`, medium, `in-trash` (one item per folder) |

The golden holds the findings (with their app), the left-out counts, the map, the cleanup's outcome and the tree after.

**Core e2e** (`crates/e2e/tests/jetbrains.rs`, the engine with the real scanner, the `jetbrains` module, the real
settings file and the real `DiskCleaner` in a fixture sandbox, `FakeInstalledApps`, `FakeRunningApps` and `FakeTrash`;
fixture home `fixtures/homes/jetbrains-flow.toml`; golden `fixtures/golden/jetbrains-flow.json`):

1. **Scan → review → confirm → Trash → put back**: the stale WebStorm folders are offered in three categories under
   the group Development; select all three; the plan names Medium as the highest level; confirm; the folders are in the
   Trash, the journal has the cleanup; put back: the folders are where they were, byte for byte.
2. **An open IDE**: GoLand running → its folders are left out with `running` in `ModuleFinished`; GoLand quits (the
   fake's list changes) and a rescan offers its older versions.
3. **An update between scans**: WebStorm 2025.3 → 2026.1 installed; the rescan offers the 2025.3 folders and keeps
   2026.1.
4. **Ignore** (plan 015): ignoring the module `jetbrains`, then the group `development`, leaves every answer and the
   storage map's explained part; restore brings it back without a rescan.
5. **The storage map**: the offered folders are explained in WebStorm's app line. The folders of the version in use
   are not in that line's data: the engine counts an app's data only in folders named after its bundle id, so
   `WebStorm2025.3` stays among the user's folders, never offered (corrected 2026-11-17, by 03/02; deferred:
   [a module names its app's data folders](../../deferred/module-names-app-data.md)).
6. **A failure of the system's facts**: the installed apps fake fails → every folder left out `no-system`, nothing
   offered, the scan ends complete.

**UI e2e** (Playwright, mocked core; `app/ui/src/test/jetbrains-scan.ts`, a scan as the core streams it with the
module's real texts read from `modules/jetbrains/locales/en.json`, shared by the unit tests and the e2e mock):

1. A scan shows **Development → JetBrains IDEs** in the sidebar with its size; its page shows the three categories
   with Safe, Safe and Medium and their texts.
2. The left-out lines under each category: "3 items left out (1.4 GB): This is the version you use." and the running
   line with its next step.
3. **Select → Review → confirm to the Trash → progress → result → Cleanups → put back**, on the JetBrains findings;
   the confirmation names Medium and the settings' "what you lose" sentence.
4. Selecting only caches and logs: the confirmation names Safe.
5. **Ignore this module** from its page → gone from the overview's totals and the sidebar dimmed; Settings → Ignored
   modules shows it on; turning it off brings it back.
6. **Ignore the group Development** from Settings → the JetBrains module is hidden; restoring the group brings it back.
7. The overview's by-meaning view shows the WebStorm app line holding the JetBrains findings.
Visual and accessibility: the module page with its left-out lines and the confirmation with Medium in `visual.spec.ts`
and `axe.spec.ts`.

**By hand** (04/02): `npm run dev:sandbox -- jetbrains` and `EXPECTED.md` walked; `npm run dev` on the real Mac (a
development build's cleaner cannot reach the real home, so this run only scans): the offered and left-out folders
compared with `ls ~/Library/{Application\ Support,Caches,Logs}/JetBrains`, the installed IDEs' versions and bundle ids
(`mdls -name kMDItemCFBundleIdentifier -name kMDItemVersion`), an open IDE left out; every fact of sections 1 and 2
that the Mac shows otherwise is corrected in the plan and the table.

### 9. Where the code lives

| Crate or folder                    | What it gains                                                                                         |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `modules/jetbrains/` (new)         | `fresher-module-jetbrains`: `lib.rs` (descriptor, `Module`), `products.rs` and `products.toml`, `name.rs`, `decide.rs`, `locales/en.json`, `tests/` |
| `crates/sandbox`                   | `[[case.app]]`, `[[case.running]]`, `[[case.left-out]]`; the bundle writer; `world.toml`; `EXPECTED.md` lines |
| `crates/testing`                   | `TreeDescription`'s `[[app]]` entry (the bundle and its plist), shared by fixture homes and case files |
| `app/tauri`                        | the module registered; sandbox mode's running apps from `world.toml`; the harness's left-out check     |
| `app/ui`                           | nothing of its own: the module uses the generic page; a test scan and the e2e scenarios               |
| `docs/categories/`                 | three category files and their rows                                                                   |

No new third-party dependency: the plist is written as text by the generator, and read by plan 009's reader.

## Interfaces with other plans

- **011** (in `main` before this is built): `with_problems`, `left_out`, `ModuleFinished.left_out`, the `detail` rule.
- **015** (built before this): `ModuleGroup::Development`; the ignore kinds module and group.
- **012**: `for_app`; the map's checks in the case file.
- **009**: `installed_apps()`, `running_apps()`, `FakeInstalledApps`, `FakeRunningApps`, `MacInstalledApps`.
- **007**: the case format, the generator, sandbox mode and the harness this plan extends.

## Open questions

None left: every question was delegated and decided below.

## Decisions

The user's, of 2026-11-17 (translated): the JetBrains module after plan 011; the plan carries its sandbox tests and its
e2e scenarios from the start; JetBrains in a group Development; ignoring a module and a group in a separate plan (015).

Taken by delegation on 2026-11-17 (the user's instruction: accept the recommended decisions):

1. **The plan is approved** as designed here, with the agent's calls below still to check.
2. **Danger levels: caches Safe, logs Safe, settings and plugins Medium** (section 5). Options for the settings: (a)
   Medium for the one folder — recommended: the catalogue's own example, and losing un-copied settings is a real loss;
   (b) Low, as plugins alone would be — too low for settings; (c) settings Medium and plugins Low as two findings —
   two claims in one folder, for a gain of a level on the smaller part.
3. **An IDE that is not installed keeps every version** (row 4; replaced by the user's decision 10 on 2026-11-17).
   Options: (a) keep and report it — recommended:
   rule 1, and an IDE installed where the app cannot see it (a drive, an older installer's folder) would otherwise lose
   the settings in use; (b) offer all but the newest present — wrong whenever the newest present is not the one in use;
   (c) offer all — the uninstalled app leftovers' job, with its own design.
4. **An open IDE keeps every version of its product** (row 2), not only its own: a just-updated IDE imports from the
   older folder at its first start.
5. **The layout before 2020 is left alone** (non-goals): its folders carry no vendor folder, so a name alone would
   have to be trusted. Options: (a) leave it — recommended; (b) the same rules on `~/Library/Preferences/<P><V>` — a
   wider net on names, for versions six years old.
6. **The sandbox world's installed and running apps are built here** (plan 01), with the left-out expectation; the
   rest of the world stays deferred, and the deferred item says so.
7. **Sandbox mode takes the running apps from the sandbox, never the real Mac**: a real open IDE must not change what a
   sandbox shows, so a run by hand matches the autotest.
8. **No version bump**: the version stays 0.1.0 ([`../README.md`](../README.md), "Versions").
9. **Built after plans 011 and 015 are merged into `main`**, on this branch made again from `main` then.

The user's, of 2026-11-17 (translated), taken after 02/02 was built, from the user's own case (DataGrip was removed and
its settings stayed):

10. **The leftovers of a JetBrains IDE that is no longer installed are offered**, as a separate category
    (`jetbrains.uninstalled`, Medium), once none of the IDE's folders changed for 30 days; the days can be changed in
    Settings. It replaces decision 3. Built in step 02/04 with the controller's rulings: the reader also looks in the
    Toolbox app's folders, and a partial installed list counts as row 1 (cost: nothing offered on a Mac where an
    apps folder or a bundle cannot be read).

## Agent's calls (to check)

1. **The layout of section 1** (three places, `<Product><Year>.<N>`, plugins inside the settings folder since 2020.1)
   is from knowledge; checked on the real Mac in 04/02.
2. **The products table** (section 2): the folder names and bundle ids, the early access ids with `-EAP`, and that the
   Community editions keep their own folders. Checked on the real Mac for the IDEs installed there; the others stay as
   written, with the table's comment saying which were seen.
3. **Code, not rules**: the module does not use `fresher-rules`; its products are data (`products.toml`). Costs, if
   wrong: a rule format for "a version older than the installed app's" later.
4. **The team id is not checked**; only bundle ids. Costs, if wrong: a signature check in the decision, and signed
   bundles in the sandbox, which it cannot make.
5. **The installed version is the start of the version string**; an app whose string does not start with
   `<Year>.<N>` is an unknown version (row 5). Costs, if wrong: a build-number reading (`253.1234` → 2025.3) later.
6. **Toolbox installs** (changed on 2026-11-17 in 02/04): an IDE that the Toolbox app installs into `~/Applications`
   or `~/Applications/JetBrains Toolbox/` is found as an app of the user's apps folder (one folder below); an older
   Toolbox that keeps the IDEs inside its own folder, `~/Library/Application Support/JetBrains/Toolbox/apps/<IDE>/
   ch-<n>/<build>/<IDE>.app`, is found by the reader looking there up to four folders below. That layout is from
   knowledge; checked on the real Mac where Toolbox is installed. Costs, if wrong: a deeper or other folder, one
   constant; until then such an IDE counts as not installed, and only the setting's days guard its data.
7. **The finding's app** is the bundle id of the newest installed app of the product. Costs, if wrong: another id, one
   line.
8. **`world.toml`** at the sandbox's root holds the running apps; its format grows into the deferred world. Costs, if
   wrong: a rename.
9. **Left-out expectations are compared exactly**, summed per category and reason over the case file. Costs, if wrong:
   a looser "at least".
10. **Sizes on the confirmation**: the plan adds nothing; the module's findings are sized by the walk like any other.
11. **The module's page is the generic one** (`customViews` stays empty). Costs, if wrong: a custom view with a
    per-IDE grouping later.
12. **Each folder is decided in `on_match`, but row 4's are gathered for `finish`** (2026-11-17, in 02/02; changed
    in 02/04): rows 1 to 3 and 5 to 8 depend on no other folder; row 4 depends on every folder of the product. A module
    instance lives across scans while `finish` is not called after a cancelled walk, so the gathered folders are kept
    with the scan's `CancelToken` (`CancelToken::is_same`, added to the core): a folder of another scan is dropped,
    never decided. Costs, if wrong: a per-scan hook in the module contract.
13. **The early access ids** (`<release id>-EAP` for every product) and **the Toolbox app's id**
    (`com.jetbrains.toolbox`) are from knowledge; every bundle id of the table is checked by hand with `mdls` on the
    real Mac in 04/02. Costs, if wrong: an early access build counted as not installed (its folders kept), or a running
    Toolbox keeping every folder; both only keep more.

14. **A folder's age is its own modification time** (02/04), as the rules' age is: an IDE changes it when it starts or
    quits (the lock and port files it writes at the top of its settings and caches folders, its logs rotated at each
    start), not when it writes only inside the folder's subfolders. The newest of the product's folders in the three
    places counts. Checked on the real Mac in 04/02 (`stat -f %Sm` of an IDE's folders against its last start).
    Costs, if wrong: the newest time inside each folder, from the walk; the age from the newest of the folder and its
    direct children is [deferred](../../deferred/folder-age-from-children.md) (2026-11-17).
15. **A partial installed list** (02/04; links changed by the controller's ruling of 2026-11-17): an apps folder, a
    folder in one, a folder link or another volume's folder among them, or a bundle whose `Info.plist` cannot be read
    (or another volume's app) makes the list partial, and the module keeps every folder as row 1. An **app that is a
    link** (`/Applications/Safari.app` is one on every Mac) is read where it points, a read-only lookup, nothing walked
    through it; a link that leads to no readable bundle makes the list partial. Costs, if wrong: nothing offered on a
    Mac with such a link.
16. **The finding's app of a removed IDE is its release bundle id** (02/04): the storage map shows a removed app by its
    id (`com.jetbrains.datagrip`), not the IDE's name; the name was
    deferred (2026-11-17), done by plan 019/01/04.
17. **An unknown or future time is recent** (02/04), as the rules' age: the product's folders are kept with
    `used-recently`; `not-installed` is gone. Costs, if wrong: a reason of its own for an unreadable time.
18. **The sandbox's clock** (02/04): sandbox mode's clock starts at `world.toml`'s `now` and runs from there
    (`StartedClock`), so progress and durations still move; the autotest's `FakeClock` stands at it; a sandbox
    generated before has no `now` and runs on the system's clock. Costs, if wrong: a frozen clock in sandbox mode.
19. **Spotlight as the second source of row 4** (the controller's ruling of 2026-11-17): asked only for a product about
    to be offered, by each of its bundle ids (`kMDItemCFBundleIdentifier == '<id>'c`, case ignored), through
    `InstalledApps::find`; a hit whose path no longer exists is not an app. A Mac with Spotlight's index turned off
    answers nothing: the final review of 2026-11-17 asks for the Finder first and keeps everything when it is not
    found (`spotlight-unread`); a single volume left out of the index is still not seen (a known issue). Costs, if
    wrong: another index, behind the same port.

## Conclusions

Written when the build ended, before the run by hand ([04/02](./04-finish/02-run-and-conclusions.md)); the
conclusions of the plan are in [`readme.md`](./readme.md). The agent's calls above are 1 to 19. The rulings taken
while building, each with what it costs if wrong:

20. **Every product has an `-EAP` bundle id, and the Toolbox app is the one vendor app that never keeps a folder**
    (02/01, 02/02): a running app whose id starts with `com.jetbrains.` and is not in the table keeps every product's
    folders (left out: an IDE is open), except the Toolbox app (`other-apps`, an exact id). Cost if wrong: nothing is
    offered while an unknown JetBrains app runs; a running Toolbox that is not exempt would keep everything.
21. **A partial installed list is row 1** (02/04): an apps folder or a bundle that cannot be read means the module offers
    nothing. Cost if wrong: nothing is offered on such a Mac.
22. **Links in the apps folders are read where they point; a folder link or another volume's folder makes the list
    partial** (02/04 fix round). Cost if wrong: nothing offered on a Mac with such a link, or an IDE behind a link
    counted as not installed (only the days guard it).
23. **Spotlight is the second source of row 4**, asked only for a product about to be offered; an error keeps the
    folders (call 19).
24. **A restored scan never offers a removed IDE's data** (its times are from the cache). Cost if wrong: such data is
    offered only after the user's own scan.
25. **An installed unknown JetBrains app keeps every removed IDE's data** (`unknown-ide`). Cost if wrong: fewer offers.
26. **The days setting has a floor of 7** (the 30-day default is the user's decision). Cost if wrong: fewer offers
    for a user who wants a shorter time.
27. **The restored-scan case is in the core e2e, not in the sandbox checklist** (03/01). Cost: the case is not visible
    in `EXPECTED.md`.
28. **The version in use is not in the IDE's line of the storage map** (03/02): the engine attributes app data to
    folders named by the bundle id; the folders of the version in use stay in the user's folders. Cost if wrong: a
    rule in the engine for names like `<Product><Year>.<N>`.

**The user's decision of 2026-11-17** (decision 10, above): the data of a removed JetBrains IDE is offered in its own
category after 30 days, changeable in Settings, replacing decision 3.

**The final review's fix wave** (2026-11-17), each with what it costs if wrong:

29. **A module page's subtitle is the module's description**; what ignoring does is the ignore button's tooltip, linked
    as its description, and stays in Settings. Cost if wrong: a line of the page.
30. **Each reason has its own words**: `apps-unread` for a partial list, `spotlight-unread` for Spotlight, and
    `vendor-app-open` for an unknown app of the vendor running; `no-system` only for a list that failed. Cost if wrong:
    texts.
31. **Spotlight is asked for the Finder first**: no answer is an index that is off, and everything is kept. Its paths
    are read NUL-separated (`mdfind -0`). A volume left out of the index is a known issue. Cost if wrong: one more
    `mdfind` per product about to be offered.
32. **A walk that could not read every folder never offers a removed IDE's data** (`ModuleContext::is_walk_complete`,
    set by the engine for `finish`; `false` during the walk). Cost if wrong: fewer offers on a Mac with an unreadable
    folder anywhere in the scan.
33. **Apps two folders below an apps folder are not seen** (section 4). Cost if wrong: an older copy there makes its
    folders look older; they are offered, Medium for the settings, and confirmed by the user.
34. **The sandbox autotests run one at a time** (a nextest group), filled files are synced before their blocks are
    read, a scan waits 180 s and a timeout names the last event. A case file may not write inside a declared app, an
    app's texts may not hold a control character, and `world.toml` must be a file of at most 1 MiB.
35. **A sandbox scan reads as made days ago** when the sandbox was generated days before (its clock starts at the
    generation): a known issue.

**What is still to do** (parked by the reviews, small): summing duplicate left-out pairs in the case reader; the
scenario flip check of the sandbox case file.
