# 018 — Developer caches: design

**Status:** ✅ approved (the user's decision of 2026-11-18) · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, scanner, testing, modules, sandbox, e2e, app, app/ui, docs

Written on 2026-11-18 from the approved one-file draft (the user approved it as written, with its fourteen agent's
calls, which became decisions) and from a read of the code on `feat/018-developer-caches` (from `main` 002ed0c). Where
the draft was wrong about the code the design is corrected and the correction is listed under "Agent's calls". Facts
about the tools marked "to verify" are checked by hand, read-only, in 04/02.

## Intent

The next module in the user's agreed order (developer caches, then uninstalled-app leftovers, downloads and
installers, permissions): the download caches of package managers, Go's build cache and Xcode's derived data — the
largest regenerable folders on a developer's Mac that nobody looks at. Each is found at its tool's known place,
explained in plain English (which tool, what comes back and what it costs) and offered to the Trash. Taken from the
"package managers" and "Xcode DerivedData" lines of the idea
[Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md).

## What the research found

Read-only, on the developer's Mac on 2026-11-18:

| Tool      | Place                                   | What is there                                                                    | Size    |
| --------- | --------------------------------------- | -------------------------------------------------------------------------------- | ------- |
| npm       | `~/.npm/_cacache`                       | `content-v2`, `index-v5`, `tmp`; beside it `_npx`, `_logs`, `_prebuilds`          | 8.9 GB  |
| Gradle    | `~/.gradle/caches`                      | a valid `CACHEDIR.TAG`, `jars-9`, `modules-2`, `journal-1`, `8.13`, many `*.lock` | 1.7 GB  |
| Cargo     | `~/.cargo/registry`                     | a valid `CACHEDIR.TAG` beside `cache`, `index`, `src`                            | 1.4 GB  |
| Homebrew  | `~/Library/Caches/Homebrew`             | `downloads`, `api`, `api-source`, `Cask`, `Backup`, `bootsnap`, top-level links into `downloads` | 0.5 GB |
| Yarn      | `~/Library/Caches/Yarn`                 | `v6`                                                                             | —       |
| pip       | `~/Library/Caches/pip`                  | `http-v2`, `selfcheck`, `wheels`                                                 | —       |
| Go        | `~/Library/Caches/go-build`             | absent on this Mac                                                               | —       |
| CocoaPods | `~/Library/Caches/CocoaPods`            | absent on this Mac                                                               | —       |
| Xcode     | `~/Library/Developer/Xcode/DerivedData` | absent on this Mac (`Xcode/` holds `UserData` only)                              | —       |

- **Go writes no `CACHEDIR.TAG`** (the draft said it does): its build cache holds a `README` written when the cache is
  made and a `trim.txt` written by its daily trim, beside 256 folders `00` to `ff` (to verify).
- **Gradle's caches always hold `*.lock` files**, and a running Gradle daemon keeps some open: the subtree's
  `inside.lock_named` is no reason to keep here (it would keep Gradle's caches for ever); the cleaner's in-use check
  (check 10) skips a cache a daemon holds open.
- **Homebrew's top-level entries are links into `downloads`**: once `downloads` is gone they point nowhere, which is
  harmless; Homebrew's own cleanup removes such links.
- **Xcode names each project's folder `<project>-<28 lowercase letters>`** and writes an `info.plist` into it; the
  shared `ModuleCache.noindex` beside them is no project (to verify, on a Mac with Xcode).

## Goals

1. One category per tool, so each has its own texts, danger level and Ignore
   ([plan 015](../015-module-groups/readme.md)).
2. Only the tool's own cache folder at its known place is ever offered; the tool's settings, credentials, installed
   programs and the user's projects never are.
3. What the module recognizes and does not offer is reported as left out with its reason, never dropped.
4. No external tool runs: the module is read-only facts and paths, the cleaner moves to the Trash.
5. The core gains what the research found missing: a matched folder reports which names it holds, so a tool's sign
   inside its own folder can be checked.

## Non-goals (each stays in its idea)

- Docker images, containers, volumes, build cache and its disk image:
  [container disk image](../../ideas/container-disk-image.md) and the Docker line of
  [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md) — always through the Docker CLI.
- Simulators, simulator runtimes, device support files, Xcode Archives (High: shipped builds' symbols), and derived
  data's shared `ModuleCache.noindex`: the Xcode line of the same idea.
- `node_modules`, `target/` and other project artifacts: [stale project cleanup](../../ideas/stale-project-cleanup.md).
- Toolchain versions (rustup, nvm, pyenv, JDKs, Playwright's browsers):
  [unused toolchain versions](../../ideas/unused-toolchain-versions.md).
- Stores and repositories that are not plain caches: pnpm's store (hard-linked into projects), Maven's
  `~/.m2/repository` (may hold the only copy of what `mvn install` put there), Cargo's git checkouts and its registry
  index, npm's `_npx`, Gradle's wrapper distributions. Each is a later addition, with its own danger.

## Anti-goals

- No cache relocated by an environment variable or a tool's config (`npm_config_cache`, `CARGO_HOME`,
  `GRADLE_USER_HOME`, `GOCACHE`, …) is guessed at: only the default places, so a moved cache is missed, never mistaken.
- No age threshold and no "unused for N days": a cache is regenerable whatever its age; the cleaner's in-use check
  protects a cache being written.

## What already exists

| Need                                    | What exists                                                                                                     | Plan     |
| --------------------------------------- | --------------------------------------------------------------------------------------------------------------- | -------- |
| A module crate and its checks           | `modules/<id>/`, `ModuleDescriptor`, `assert_module_contract`, `run_module`, goldens ([`modules.md`](../../architecture/modules.md)) | 002 |
| The places                              | `KnownFolder::{Home, Library, AppCaches}`; `Pattern::Under(folder, path)` with `*` for one name                  | 002      |
| A folder summed whole, and what it holds | `Claim::Whole`, `Match.sizes`, `Match.inside` (`complete`, `foreign`, `special`)                               | 004, 017 |
| Names next to a match                   | `Match.siblings` (presence only, from the parent's listing)                                                     | 002      |
| A valid `CACHEDIR.TAG`                  | `is_valid_cachedir_tag`, `Match.marker` — only for an unanchored `Pattern::MarkerFile`, which matches any folder of the home holding the tag | 002, 004 |
| A data table embedded in a module       | `modules/jetbrains/products.toml`, `include_str!`, parsed and checked when the module is built                  | 016      |
| Running and installed apps              | `cx.system().running_apps()` (`RunningSet::apps`, each with its bundle ids)                                     | 009      |
| Saying why something is not offered     | `out.left_out(LeftOutItem { … })`, `with_problems(keys)`                                                        | 011      |
| A finding's app in the storage map      | `NewFinding::for_app(bundle_id)`                                                                                | 012      |
| Groups and ignoring them                | `ModuleGroup::Development`, `IgnoreTarget::{Module, Group}`, a category's Ignore                                | 015      |
| Answers that wait for the whole walk    | `finish`, the jetbrains module's `Gathered` keyed by the scan's cancel token                                    | 016      |
| Stated owners, flags and kinds          | the tree vocabulary's `owner`, `flags`, `kind`; `FactsLens`                                                     | 017      |

**What is missing**: a tool's sign **inside** its own cache folder (npm's `index-v5`, Gradle's `CACHEDIR.TAG`, Go's
`README`, Xcode's `info.plist`) cannot be seen today. `siblings` look beside a match, not into it; the only content
check, `Pattern::MarkerFile`, is unanchored and would match every Rust `target/` of the home; a module never reads the
disk. And the tree vocabulary of fixtures and case files cannot write a valid `CACHEDIR.TAG` (a file is zeros or a
fixed pattern). Both are built first (plan 01, and 02/03 for the tag).

## Design

### 1. What a matched folder holds (core prerequisite)

An anchored interest may name entries its matched folder must be checked for:

```rust
impl Interest {
    /// Names the walk looks for in the matched folder's own listing (`index-v5`, `CACHEDIR.TAG`); at most 64, single
    /// names, only for a `Pattern::Under` interest.
    pub fn holding(self, names: Vec<String>) -> Result<Self, InterestError>;
    pub fn holds(&self) -> &[String];
}

pub struct Match {
    // …
    /// Which of the interest's `holds` names the matched folder itself holds, in the interest's order; empty for a
    /// file, a link, a folder not entered and an interest that names none.
    pub holds: Vec<String>,
    // `marker`: also `Some(Valid | Invalid)` for an interest whose `holds` names `CACHEDIR.TAG` (`Invalid` when it is
    // absent, not a regular file, unreadable or without the signature).
}
```

- `InterestError` gains `HoldsNotAnchored` (any pattern but `Under`) and `TooManyHolds` (more than 64); each name is
  checked as a sibling's is.
- The walk reads the matched folder's listing anyway, whether it claims the folder whole (the opaque root's job) or as
  an item (a plain folder job): the job knows the anchored interests that end at its trie state, and for those that
  hold names it builds a mask over its own listing (an entry that could not be read is absent; any kind counts, but a
  `CACHEDIR.TAG` counts only as a regular file, whose first bytes are read as a `MarkerFile` tag's are). The mask joins
  the folder's hit when the aggregator emits the match.
- The scan cache stores the mask with each hit (the tree format goes up by one, so an older cache is refused and the
  next scan is full); a rescan recomputes it for a folder it reads again and keeps it for one it copies; a restore
  hands it back. The reference walk (`NaiveWalk`) computes the same, and the scanner's property test compares them.
- Cost: one name lookup per entry of a matched folder's own listing, only for interests that hold names; the scan
  bench is run before and after (rule 4).

### 2. The places and their signs

Every place is data, `modules/dev-caches/tools.toml` (rule 5), embedded with `include_str!` and parsed and checked
when the module is built, as `products.toml` is for `jetbrains`. One row per offered folder:

| Row id               | Category                        | Known folder | Cache folder (`path`)              | Sign at (`sign-at`) | Sign (`sign`, all of them)   | Comes back | App                  |
| -------------------- | ------------------------------- | ------------ | ---------------------------------- | ------------------- | ---------------------------- | ---------- | -------------------- |
| `npm`                | `dev-caches.npm`                | `home`       | `.npm/_cacache`                    | —                   | `index-v5`, `content-v2`     | download   | —                    |
| `yarn`               | `dev-caches.yarn`               | `app-caches` | `Yarn`                             | —                   | `v6`                         | download   | —                    |
| `homebrew`           | `dev-caches.homebrew`           | `app-caches` | `Homebrew/downloads`               | `Homebrew`          | `api`                        | download   | —                    |
| `cargo-cache`        | `dev-caches.cargo`              | `home`       | `.cargo/registry/cache`            | `.cargo/registry`   | `CACHEDIR.TAG` (valid)       | download   | —                    |
| `cargo-src`          | `dev-caches.cargo`              | `home`       | `.cargo/registry/src`              | `.cargo/registry`   | `CACHEDIR.TAG` (valid)       | download   | —                    |
| `cocoapods`          | `dev-caches.cocoapods`          | `app-caches` | `CocoaPods`                        | —                   | `Pods` (to verify)           | download   | —                    |
| `gradle`             | `dev-caches.gradle`             | `home`       | `.gradle/caches`                   | —                   | `CACHEDIR.TAG` (valid)       | download   | —                    |
| `pip`                | `dev-caches.pip`                | `app-caches` | `pip`                              | —                   | `http-v2`                    | download   | —                    |
| `go-build`           | `dev-caches.go-build`           | `app-caches` | `go-build`                         | —                   | `README`, `trim.txt` (to verify) | build  | —                    |
| `xcode-derived-data` | `dev-caches.xcode-derived-data` | `library`    | `Developer/Xcode/DerivedData/*`    | —                   | `info.plist` (to verify)     | build      | `com.apple.dt.Xcode` |

A row also carries `tool`, the tool's name as its makers write it (`npm`, `Yarn`, `Homebrew`, `Cargo`, `CocoaPods`,
`Gradle`, `pip`, `Go`, `Xcode`), which the finding's reason names, and the Xcode row `name = "project-hash"`: each
match's name must be `<project>-<28 ASCII lowercase letters>` with a non-empty project (to verify).

```toml
[[cache]]
id = "cargo-src"                 # its title (`dev-caches.title.cargo-src`) and its findings' identity
category = "dev-caches.cargo"
tool = "Cargo"
folder = "home"                  # a known folder, as rules write it
path = ".cargo/registry/src"     # the folder offered; `*` matches one name
sign-at = ".cargo/registry"      # where the sign is, when not in the folder offered; an ancestor of `path`
sign = ["CACHEDIR.TAG"]          # names that folder must hold, all of them; a `CACHEDIR.TAG` must be valid
comes-back = "download"          # "download" or "build": which reason the finding gives
# app = "com.apple.dt.Xcode"     # kept while this app is open; the finding counts in its line of the storage map
# name = "project-hash"          # each match's name must have this shape; only with a `path` ending in `*`
```

The table is refused when: it is not valid TOML or has an unknown field; it has no row; a row id is not kebab case or
is listed twice; a category is not one of the module's, or a module category has no row; the known folder is a walked
place (`user-temp`, `shared-temp`); a `path` or `sign-at` is not a relative path of plain names, or `sign-at` is not a
strict ancestor of `path`; two rows have the same place; two rows with the same `sign-at` have different signs; a
`sign` is empty, holds a name that is not a single name, or more than 64; the app is not a valid bundle id; `name` is
given without a `path` ending in `*`.

The interests, built from the table in its order: per row, `Under(folder, path)` `Claim::Whole`, `.holding(sign)` when
the row has no `sign-at`; per distinct `(folder, sign-at)`, one `Under(folder, sign-at)` `Claim::Item`
`.holding(sign)`. A table built with the interests maps each interest index to its row (an offered folder) or to its
sign holder.

**Categories and their dangers are code** (`modules/dev-caches/src/categories.rs`, a `const` list of id and danger),
since a `CategoryId` is a `&'static str`, as `jetbrains` keeps its places' categories in `places.rs`; the table names a
category by id and is checked against the list. The category files of the catalogue carry the same dangers (the
contract test).

### 3. The decision

The walk hands matches in no fixed order, and the Cargo and Homebrew rows need their sign holder's match. So every
match is gathered for the scan (keyed by the scan's cancel token, as `jetbrains`'s `Gathered`: a scan cancelled before
`finish` leaves nothing to the next) and decided in `finish`, each offered-folder match on its own, the first row
that applies:

| #   | Situation                                                                                                     | Result                                       |
| --- | ------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1   | the match is not a folder (a file, a link, a socket, a pipe)                                                  | not claimed                                  |
| 2   | the row's `name` shape does not fit the folder's name                                                         | not claimed                                  |
| 3   | the sign is missing: the folder holding it (the match itself, or the `sign-at` match, which must be a folder) lacks a name of `sign`, or `sign` names `CACHEDIR.TAG` and `marker` is not `Valid` | not claimed |
| 4   | the folder was not read in full (`inside` is `None` or `!inside.complete`)                                     | not claimed (the walk reports what it could not read) |
| 5   | a socket, a pipe or a device inside (`inside.special`)                                                        | left out `dev-caches.left-out.special`       |
| 6   | not the user's, a system flag, restricted or the user's lock on the folder itself, or such an entry inside (`inside.foreign`) | left out `dev-caches.left-out.not-yours` |
| 7   | the row names an app and the running apps cannot be listed                                                    | left out `dev-caches.left-out.no-system`     |
| 8   | the row's app is open (its bundle id among the running apps' bundle ids)                                      | left out `dev-caches.left-out.app-open`      |
| 9   | otherwise                                                                                                     | **offered** in the row's category            |

- Rows 1 to 4 are "not recognized as the tool's whole cache" (rule 1): no left-out, nothing said.
- `inside.lock_named` and `inside.newest_change` are not read (research; anti-goals).
- A restored scan decides the same: nothing here depends on times, and the signs and insides come from the cache; the
  guard checks the item again at the act.
- The cleaner's in-use check (check 10) skips a cache a tool holds open at the act (`InUse`, named); the module does
  not look for open files at the scan.

A finding:

| Part     | Value                                                                                                                |
| -------- | -------------------------------------------------------------------------------------------------------------------- |
| category | the row's, at its danger                                                                                             |
| identity | `<row id>/<folder name>` (`npm/_cacache`, `cargo-src/src`, `xcode-derived-data/Demo-abcdefghijklmnopqrstuvwxyzab`)      |
| subject  | `Subject::Path` of the folder; removal `Removal::MoveToTrash` of the same path                                       |
| sizes    | the match's (the whole folder)                                                                                       |
| title    | `dev-caches.title.<row id>`; the Xcode row's takes `{project}`, the name before its last `-`                          |
| reason   | `dev-caches.finding.reason-download` or `dev-caches.finding.reason-build`, with `{tool}`                              |
| app      | `for_app(app)` for a row with an app                                                                                 |

### 4. Categories, dangers and texts

| Category                        | Danger | Name                     |
| ------------------------------- | ------ | ------------------------ |
| `dev-caches.npm`                | Low    | npm's download cache     |
| `dev-caches.yarn`               | Low    | Yarn's download cache    |
| `dev-caches.homebrew`           | Low    | Homebrew's downloads     |
| `dev-caches.cargo`              | Low    | Cargo's downloaded crates |
| `dev-caches.cocoapods`          | Low    | CocoaPods' download cache |
| `dev-caches.gradle`             | Low    | Gradle's caches          |
| `dev-caches.pip`                | Low    | pip's download cache     |
| `dev-caches.go-build`           | Safe   | Go's build cache         |
| `dev-caches.xcode-derived-data` | Safe   | Xcode build data         |

Low, not Safe, for downloads: offline the next install fails, and a private registry needs its credentials again.

The five texts of each category (rule 7: plain, short, what you keep or lose), in `modules/dev-caches/locales/en.json`:

- **npm** — what: "Copies of every package npm has downloaded, kept so that installing the same version again needs no
  download." made-by: "npm, the package manager that comes with Node.js." if-removed: "It goes to the Trash; the space
  comes back when you empty it. Your projects and the packages installed in them keep working." comes-back: "Yes. npm
  downloads each package again the next time a project installs it, which needs the network and takes longer."
- **Yarn** — what: "Copies of the packages Yarn has downloaded, kept so that the next install needs no download."
  made-by: "Yarn, a package manager for JavaScript projects." if-removed: as npm's. comes-back: "Yes. Yarn downloads
  each package again on the next install that needs it, which needs the network and takes longer."
- **Homebrew** — what: "The packages and apps Homebrew downloaded to install or upgrade them. Once installed, they
  are not needed." made-by: "Homebrew, the package manager for the command line." if-removed: "They go to the Trash.
  Everything Homebrew installed keeps working; Homebrew tidies its own links to them on its next cleanup." comes-back:
  "Only when Homebrew installs or upgrades something: it downloads what it needs then. Homebrew also clears old
  downloads on its own."
- **Cargo** — what: "The crates (Rust libraries) Cargo downloaded from its registry, packed and unpacked, kept so
  that the next build needs no download." made-by: "Cargo, Rust's build tool and package manager." if-removed: "They
  go to the Trash. Your projects, Cargo's settings and the programs you installed with Cargo stay." comes-back: "Yes.
  Cargo downloads each crate again the next time a project builds with it, which needs the network."
- **CocoaPods** — what: "Copies of the libraries CocoaPods downloaded for iOS and macOS projects." made-by:
  "CocoaPods, a dependency manager for Swift and Objective-C projects." if-removed: "It goes to the Trash. Your
  projects keep the libraries already installed in them." comes-back: "Yes. CocoaPods downloads each library again
  the next time a project installs it."
- **Gradle** — what: "Libraries Gradle downloaded and files it prepared for builds of Java, Kotlin and Android
  projects." made-by: "Gradle, the build tool of Java, Kotlin and Android projects, and the IDEs that run it."
  if-removed: "They go to the Trash. Your projects and Gradle's settings stay." comes-back: "Yes. The next build
  downloads and prepares them again, so it takes longer and needs the network."
- **pip** — what: "Copies of the Python packages pip downloaded or built, kept so that the next install is faster."
  made-by: "pip, the package installer of Python." if-removed: as npm's. comes-back: "Yes. pip downloads each package
  again the next time it installs it."
- **Go** — what: "Compiled pieces of Go programs, kept so that the next build only compiles what changed." made-by:
  "The Go tools, on every build or test." if-removed: "It goes to the Trash. Your code and the programs you built
  stay." comes-back: "Yes. The next build compiles everything again, with no download; it is slower once."
- **Xcode** — what: "What Xcode makes while it builds and indexes a project: compiled files, the code index and build
  logs, one folder per project." made-by: "Xcode, Apple's app for building apps." if-removed: "It goes to the Trash.
  Your projects and their code stay; Xcode builds and indexes them again when you open them." comes-back: "Yes. Xcode
  makes it again on the next build of the project, with no download; that build is slower."

The finding texts: `dev-caches.title.npm` "npm cache", `.yarn` "Yarn cache", `.homebrew` "Homebrew downloads",
`.cargo-cache` "Cargo's downloaded crates", `.cargo-src` "Cargo's unpacked crate sources", `.cocoapods` "CocoaPods
cache", `.gradle` "Gradle caches", `.pip` "pip cache", `.go-build` "Go build cache", `.xcode-derived-data` "{project}";
`dev-caches.finding.reason-download` "{tool} downloads it again when it needs it.",
`dev-caches.finding.reason-build` "{tool} builds it again when it needs it."

The module's switch: `dev-caches.enabled.label` "Developer caches", `.description` "Finds the download caches of
package managers and Xcode's build data. The tools make them again when they need them." No other setting.

The left-out reasons, declared with `with_problems`:

| Key                              | Text                                                                                                   |
| -------------------------------- | ------------------------------------------------------------------------------------------------------ |
| `dev-caches.left-out.not-yours`  | Some of it belongs to the system or another user, or is locked, often after a command run with sudo. Removing it needs an administrator. |
| `dev-caches.left-out.special`    | It holds a connection a running program may be using, so it is kept.                                  |
| `dev-caches.left-out.app-open`   | Its app is open and writes here as it works. Quit the app and scan again.                              |
| `dev-caches.left-out.no-system`  | Fresher could not list your open apps, so it keeps this. Scan again.                                   |

### 5. The module

`modules/dev-caches` (`fresher-module-dev-caches`), id `dev-caches`, group **Development**, registered in every
build after `temp`. Code with a data table, not rules: rules cannot report left-outs yet
([deferred](../../deferred/match-explanations.md)), and the Cargo and Homebrew signs lie in another match.

| File               | What                                                                                       |
| ------------------ | ------------------------------------------------------------------------------------------ |
| `tools.toml`       | the table of section 2                                                                     |
| `src/lib.rs`       | `DevCachesModule`, `DevCachesError`, the descriptor, `interests`, `on_match`, `finish`     |
| `src/categories.rs`| the nine categories and their dangers                                                      |
| `src/tools.rs`     | `Tools::shipped()`, `Tools::parse(text, categories)`, `ToolsError`, the interests and the index map |
| `src/name.rs`      | the `project-hash` shape: `project(name) -> Option<&str>`, on bytes                         |
| `src/decide.rs`    | `decide(candidate, row, sign, running) -> Verdict` on plain values, rows 1 to 9             |
| `src/gathered.rs`  | the matches of one scan, keyed by its cancel token                                         |
| `locales/en.json`  | every text of section 4                                                                    |

Dependencies: `fresher-core`, `serde`, `thiserror`, `toml` (as `jetbrains`); dev: `fresher-testing`, `proptest`,
`serde_json`.

### 6. Tests

**Unit** (`modules/dev-caches/src/*`, negatives first): `tools.rs` — every refusal of section 2 by a table made for
it, then the shipped table read (ten rows, nine categories, twelve interests: ten folders claimed whole, two sign
holders as items) and its interests in order; `name.rs` — `Demo-abc` (short), 27 and 29 letters, an upper-case
letter, a digit, no project (`-abc…`), no dash, a name that is not UTF-8, then `Demo-<28>`, `My-App-<28>` (the project
`My-App`); `decide.rs` — each row's keep before its offer, the precedence (not a folder over everything, sign over
inside, special over not-yours, not-yours over the app), a `CACHEDIR.TAG` named but `Invalid` or `None`, a `sign-at`
match that is a link, then every offer; a property test: an offered candidate stays offered when names are added to
what its folder holds.

**Fixture** (`modules/dev-caches/tests/fixtures.rs`, `run_module`, `Catalogue::Required`); every root injected, never
the real home:

| Home                                  | Kind     | What                                                                                        | Expected                         |
| ------------------------------------- | -------- | ------------------------------------------------------------------------------------------- | -------------------------------- |
| `dev-caches-similar-names.toml`       | negative | `.npm/_cacache-old/` with both signs, `Library/Caches/Yarn2/v6/`, `.cargo/registry/cache-old/`, `.gradle/caches-old/` with a valid tag, `.npm/_npx/`, `.npm/_logs/`, `.cargo/registry/index/`, `.cargo/git/checkouts/`, `.cargo/bin/`, `.cargo/credentials.toml`, `.m2/repository/`, `Library/pnpm/store/`, `.gradle/wrapper/dists/`, `.gradle/gradle.properties`, `.npmrc`, `Library/Caches/Homebrew/{api,Cask}/` and a top-level link, `DerivedData/{ModuleCache.noindex,Demo-short,Plain}/` | nothing offered, nothing left out |
| `dev-caches-unsigned.toml`            | negative | every place without its sign: `_cacache` with `content-v2` only, `Yarn/v5`, Gradle's tag sparse (invalid), Cargo's registry with no tag, Homebrew with no `api`, `pip` with `wheels` only, `go-build` with `README` only, `CocoaPods/Specs` only, a project without `info.plist` | nothing offered, nothing left out |
| `dev-caches-not-folders.toml`         | negative | `go-build` a file; `CocoaPods` a link to `elsewhere/CocoaPods/` (with `Pods`); a project that is a link to `elsewhere/Link/` (with `info.plist`); a project that is a file (a tree cannot hold two entries at one path, so these are not in the unsigned home) | nothing offered, nothing left out |
| `dev-caches-kept.toml`                | negative | npm's cache holding a stated socket; Gradle's caches holding a file stated `owner = "system"`; Cargo's `src` stated `flags = ["user-lock"]`; a derived data project with Xcode open | special 1, not-yours 2, app-open 1 |
| (the same, running apps failing)      | negative | the derived data project                                                                    | no-system 1                      |
| `dev-caches-caches.toml`              | positive | every row's cache with its sign (valid tags, `fill = "cachedir-tag"`), two derived data projects (`Demo-…`, `My-App-…`) | eleven findings (nine rows, two projects), each in its category at its danger; the projects `for_app` Xcode; golden `fixtures/golden/dev-caches.json` |

Plus: the matches handed in two orders give the same answers; a restored run gives the same findings.

**Sandbox** (rule 8). The places are fixed, so a world where a sign is missing or Xcode is open cannot sit beside the
positives: three case files, as `jetbrains` has its own variants. The autotest runs each file alone; `all` joins them
for a look by hand only (there the unsigned files land in signed places), so no two files write the same path: a
place's file in one is never named as in another, and the cases that would need a different entry at the place itself
(an invalid tag, a link or a file at the place) are proven by the fixtures and the core e2e, or on a derived data
project of their own.

`fixtures/sandboxes/dev-caches.toml` (`feature = "dev-caches"`, golden `sandbox-dev-caches.json`), negatives first:

| Case (negative)                                                    | Files and stated facts                                                     | Expected                       |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------- | ------------------------------ |
| look-alike folders are never claimed                               | `home/.npm/_cacache-old/{index-v5,content-v2}/x`, `home/Library/Caches/Yarn2/v6/x`, `home/.cargo/registry/cache-old/x`, `home/.gradle/caches-old/CACHEDIR.TAG` (valid) | nothing offered; `[case.map]` `your-folders` at `home/.npm/_cacache-old` |
| a tool's settings, credentials and programs are never claimed      | `home/.npmrc`, `home/.cargo/credentials.toml`, `home/.cargo/bin/cargo-tool`, `home/.gradle/gradle.properties` | nothing offered |
| stores that are not plain caches are never claimed                 | `home/.npm/_npx/a/x`, `home/.m2/repository/x.jar`, `home/Library/pnpm/store/v3/x`, `home/.cargo/git/checkouts/x/y`, `home/.cargo/registry/index/x`, `home/.gradle/wrapper/dists/x.zip` | nothing offered; `your-folders` at `home/.cargo/registry/index` |
| Homebrew's own data beside its downloads is never claimed          | `home/Library/Caches/Homebrew/api/formula.jws.json`, `…/Cask/x`, `…/Backup/x` | nothing offered |
| derived data that is no project is never claimed                   | `…/DerivedData/ModuleCache.noindex/x`, `…/DerivedData/Demo-short/info.plist`  | nothing offered |
| a link or a file in a project's place is never claimed             | `…/DerivedData/Link-defghijklmnopqrstuvwxyzabcde` → `home/elsewhere/Link/` (holding `info.plist`), `…/DerivedData/File-efghijklmnopqrstuvwxyzabcdef` a file | nothing offered; the link's target unchanged after the cleanup |
| a project holding a socket is kept                                 | `…/DerivedData/Sock-abcdefghijklmnopqrstuvwxyzab/{info.plist, x.sock (kind = "socket")}` | left out `special` |
| a project holding the system's file is kept                        | `…/DerivedData/Root-abcdefghijklmnopqrstuvwxyzab/{info.plist, Build/x (owner = "system")}` | left out `not-yours` |

Then the positives, each filled with data, Xcode installed (`[[case.app]]`, `com.apple.dt.Xcode`) and closed:

| Case (positive)                                       | Files                                                                    | Expect                                    |
| ----------------------------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------- |
| npm's cache goes to the Trash                          | `home/.npm/_cacache/{index-v5,content-v2,tmp}/…`                         | `dev-caches.npm`, low, `in-trash`; map `explained` at `home/.npm/_cacache` |
| Yarn's cache goes                                      | `home/Library/Caches/Yarn/v6/…`                                          | `dev-caches.yarn`, low, `in-trash`        |
| Homebrew's downloads go, its links stay                | `home/Library/Caches/Homebrew/downloads/<hash>--x.bottle.tar.gz`; the link `Homebrew/x--1.0` → it | `dev-caches.homebrew`, low, `in-trash`; the link stays |
| Cargo's downloaded and unpacked crates go              | `home/.cargo/registry/CACHEDIR.TAG` (valid), `cache/index.crates.io-x/a.crate`, `src/index.crates.io-x/a/lib.rs` | `dev-caches.cargo` ×2, low, `in-trash`; `index` stays |
| CocoaPods' cache goes                                  | `home/Library/Caches/CocoaPods/Pods/Release/x`                           | `dev-caches.cocoapods`, low, `in-trash`   |
| Gradle's caches go                                     | `home/.gradle/caches/CACHEDIR.TAG` (valid), `modules-2/x.jar`, `modules-2/modules-2.lock` | `dev-caches.gradle`, low, `in-trash`      |
| pip's cache goes                                       | `home/Library/Caches/pip/{http-v2,wheels}/…`                             | `dev-caches.pip`, low, `in-trash`         |
| Go's build cache goes                                  | `home/Library/Caches/go-build/{README,trim.txt,00/x-a}`                  | `dev-caches.go-build`, safe, `in-trash`   |
| a closed Xcode's project build data goes, in Xcode's line | `…/DerivedData/Demo-abcdefghijklmnopqrstuvwxyzab/{info.plist,Build/x,Index.noindex/y}` | `dev-caches.xcode-derived-data`, safe, `in-trash`; map `explained` |

`fixtures/sandboxes/dev-caches-unsigned.toml` (`feature = "dev-caches-unsigned"`): negatives only, one per tool, each
place without its sign and with files named apart from `dev-caches.toml`'s ("npm's cache without its index is not
claimed", …, "a project's folder without Xcode's file is not claimed"), nothing offered and nothing left out.

`fixtures/sandboxes/dev-caches-xcode-open.toml` (`feature = "dev-caches-xcode-open"`): Xcode open (`[[case.running]]`);
negative "a project's build data is kept while Xcode is open" (left out `app-open`); positive "npm's cache still goes
while Xcode is open" (only Xcode's category waits).

**Core e2e** (`crates/e2e/tests/dev_caches.rs` and its harness `dev_caches_home/`, the engine with the real scanner,
the facts lens, the real settings, cache and journal files and the real `DiskCleaner` on a fixture home, with
`FakeClock`, `FakeTrash`, `FakeRunningApps`, `FakeInstalledApps`, `FakeOpenFiles`; fixture
`fixtures/homes/dev-caches-flow.toml`, golden `fixtures/golden/dev-caches-flow.json`). First, the deferred
shared helpers move into `crates/e2e/tests/support/`, and this harness uses
them.

1. **Scan → confirm → Trash → put back**: Development → Developer caches offers npm, Cargo's two folders, Gradle and
   a derived data project; the plan names Low; confirmed, they are in the Trash and the journal; put back restores
   them byte for byte. Cargo's `index`, `bin` and `credentials.toml` are untouched.
2. **Held open**: a file of Gradle's caches held open (`FakeOpenFiles`): Gradle skipped `InUse`, the rest done.
3. **Xcode open, then closed**: the project left out `app-open`; Xcode quits; a rescan offers it.
4. **The sign through the cache**: Gradle's tag invalid: not claimed; the tag rewritten valid, a rescan offers it; the
   tag removed, a rescan claims nothing (the mask recomputed by the rescan, never kept from the cache).
5. **Read in part**: a folder inside npm's cache made unreadable (mode 000, restored after): npm's cache not claimed,
   nothing else changes (the scanner does not count the folder as not read:
   [known issue](../../known-issues/scanner-unreadable-folder-in-a-claimed-match.md); the cache's node is `Partial`).
6. **Restore at launch**: the restored scan offers the same findings as the walk (signs from the cache).
7. **Ignore**: npm's category, then the module; restore brings each back.
8. **The storage map**: the offered caches are explained; the project counts in Xcode's line.

**UI e2e** (Playwright, mocked core; `app/ui/src/test/dev-caches-scan.ts` with the module's real texts):

1. A scan shows **Development → Developer caches** in the sidebar with its size; its page lists the categories with
   their dangers; a category shows its five texts.
2. The left-out lines ("Its app is open…", "Some of it belongs to the system…"), not selectable.
3. **Select → Review → confirm to the Trash → result → Cleanups → put back** on npm's cache and a project; the
   confirmation names Low.
4. **Ignore** npm's category from its page: the others stay; restore. Then ignore the module.
5. Settings → Developer caches: the switch alone.
6. The overview's by-meaning view: the project's size in Xcode's line.

Visual and accessibility: the module's page with its left-out lines, in `visual.spec.ts` and `axe.spec.ts`.

**By hand** (04/02): the three sandboxes walked with `EXPECTED.md`; on the developer's Mac a scan only (a development
build's cleaner reaches only a sandbox), compared with read-only commands; then one real cleanup of npm's cache made
with Finder and an `npm install`, which makes it again.

### 7. Where the code lives

| Crate or folder                     | What it gains                                                                            |
| ----------------------------------- | ---------------------------------------------------------------------------------------- |
| `crates/core`                       | `Interest::holding`, `holds`, `InterestError::{HoldsNotAnchored, TooManyHolds}`, `Match.holds`, `marker` for a held tag |
| `crates/scanner`                    | the mask in the walk, the hit, the codec (tree format + 1), the rescan                   |
| `crates/testing`                    | the mask in `NaiveWalk`; `fill = "cachedir-tag"` in the tree vocabulary                  |
| `modules/dev-caches/` (new)         | `fresher-module-dev-caches`                                                              |
| `crates/e2e`                        | the shared helpers in `support/`; the dev-caches flows                                   |
| `app/tauri`, `app/ui`               | registration; the case files' tests; the UI's test scan and scenarios                    |

No new third-party dependency.

## Interfaces with other plans

- **017** (merged): `Inside` (`complete`, `foreign`, `special`), the stated facts and `FactsLens`, the e2e homes whose
  helpers move here.
- **016** (merged): the embedded data table, `Gathered`, running and installed apps in the sandbox and the fakes.
- **015**: the group Development, a category's and a module's Ignore.
- **012**: `for_app`, the storage map.
- **011**: left-outs and declared problems.
- **008**: the guard, its in-use check, the Trash and put back.

## Deferred items

- Shared helpers of the core e2e homes: due at this harness; done in 03/02,
  its file and row removed there.
- Not due: [why an item matched](../../deferred/match-explanations.md) (the module is code, not rules);
  [the sandbox world](../../deferred/sandbox-world.md) (the module reads no open files and runs no tool).

## Open questions

None.

## Decisions

The user's, of 2026-11-18 (translated): "Approve, write the design and start the implementation." The draft was
approved as written, and its fourteen agent's calls became these decisions:

1. **Scope: seven package-manager caches, Go's build cache and Xcode's derived data**: the biggest plain caches.
2. **pnpm's store, Maven's repository, Cargo's git checkouts and `_npx` left out**: each may not come back as is.
3. **One category per tool**: per-tool texts, danger and Ignore, no per-tool setting needed.
4. **Low for downloads, Safe for derived data and Go's build cache**: a download needs the network; a rebuild does not.
5. **The Trash, never the tool's command**: recoverable, no `--force`, no external program in the deletion path. The
   tools' commands delete for good; `npm cache clean` needs `--force`, which the plan builder refuses; `brew cleanup`
   also removes old installed versions; Cargo has no stable command; a GUI app cannot rely on the user's `PATH`. Each
   tool makes its cache folder again on its next run. This settles the idea's "prefer the tool's own command" for
   caches; toolchains decide again in their own idea.
6. **Default places only, no environment or config read**: a moved cache is missed, never mistaken.
7. **A tool's sign required where it writes one**: the place's name alone is not enough (rule 1).
8. **Code with a data table, not rules**: left-out reasons are needed and rules cannot give them yet.
9. **Derived data left out while Xcode is open**: Xcode writes its index there continuously.
10. **No age threshold and no setting**: a cache is regenerable whatever its age; the in-use check covers writes.
11. **A folder read in part, foreign or holding a socket is left out**: the same refusals as `temp` (read in part: not
    claimed, as `temp` does).
12. **One finding per derived-data project, one per other cache folder**: the user chooses by project where it matters.
13. **Module id `dev-caches`, group Development.**
14. **The e2e helpers' move is done here**: its deferred item is due at this harness.
15. **No version bump**: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code; each with what it costs if wrong.

1. **A core prerequisite after all** (the draft said none): a sign inside the cache folder cannot be seen today
   (`siblings` look beside a match, `MarkerFile` is unanchored and would match every `target/` of the home, a module
   never reads the disk). `Interest::holding` and `Match.holds` are built first (plan 01). Cost if wrong: an unanchored
   `MarkerFile` interest and children-name matches instead, with a full pass over every listing of the home.
2. **Go's sign is `README` and `trim.txt`, not a `CACHEDIR.TAG`** (the draft's was wrong; to verify). Cost if wrong: a
   name changed in the table.
3. **Each tool's sign**, all names required: npm `index-v5` and `content-v2`; Yarn `v6`; Homebrew `api` beside
   `downloads`; Cargo a valid tag beside `cache` and `src`; CocoaPods `Pods`; Gradle a valid tag; pip `http-v2`; Xcode
   `info.plist` and the `<project>-<28 letters>` name. An older layout (Yarn `v5`, pip `http`) is missed, never
   mistaken. Cost if wrong: a cache missed until its name is added.
4. **A sign beside the folder is checked on its holder's own match** (`sign-at`, a `Claim::Item` interest), so the
   Cargo tag's validity is read, not only its presence. Cost if wrong: `siblings` (presence only) for Cargo and
   Homebrew.
5. **Every answer waits for `finish`** (gathered per scan, as `jetbrains`'s removed IDEs): two rows' signs lie in
   another match and matches come in no order. Cost if wrong: none for safety; the per-match rows decided earlier.
6. **Categories and dangers in code, places and signs in `tools.toml`**: a `CategoryId` is a `&'static str`, as
   `jetbrains` keeps its categories in `places.rs`. Cost if wrong: a leak or a core change for run-time category ids.
7. **Cargo is two rows of one category** (`cargo-cache`, `cargo-src`): two findings with their own titles, one
   category, one Ignore. Cost if wrong: one more category.
8. **A restored scan decides as a walk does**: nothing depends on times; the guard checks again at the act. Cost if
   wrong: a `scan-again` row, as `temp`'s.
9. **`inside.lock_named` is not a refusal**: Gradle's caches always hold `*.lock` files; the in-use check at the act
   skips what a daemon holds open. Cost if wrong: Gradle's caches never offered.
10. **One left-out for owner, flags and locks** (`not-yours`), unlike `temp`'s `system` and `locked`: `inside.foreign`
    does not tell them apart for an entry inside, and a cache written with `sudo` is the common case. Cost if wrong:
    two texts.
11. **Derived data's `ModuleCache.noindex` is not claimed** (no project, no `info.plist`); it stays in the idea. Cost
    if wrong: a row added.
12. **Three case files** (`dev-caches`, `dev-caches-unsigned`, `dev-caches-xcode-open`): the places are fixed and the
    open apps are a file's own. Cost if wrong: none; `all` joins them for a look by hand only.
13. **The real cleanup by hand is made with Finder**, not the app: a development build's cleaner reaches only a
    sandbox (the draft's "one real cleanup of npm's cache" assumed the app). Cost if wrong: none.
14. **The plans gain a first one** (core prerequisites), so the draft's 01, 02, 03 are 02, 03, 04.

## Conclusions

Not written yet.
