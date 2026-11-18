# 018 — Developer caches

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, core, sandbox, testing, app,
app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The next module in the user's agreed order (developer caches, then uninstalled-app leftovers, downloads and
installers, permissions): the download caches of package managers and Xcode's derived data — the largest regenerable
folders on a developer's Mac that nobody looks at (on the developer's Mac on 2026-11-18: npm's cache 8.9 GB, Gradle's
1.7 GB, Cargo's registry 1.4 GB, Homebrew's 0.5 GB). Each is found at its tool's known place, explained in plain
English (which tool, what comes back and what it costs) and offered to the Trash. Taken from the "package managers"
and "Xcode DerivedData" lines of [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md).

## Goals

1. One category per tool, so each has its own texts ("npm downloads it again on the next install"), danger level and
   Ignore ([plan 015](../015-module-groups/readme.md)).
2. Only the tool's own cache folder at its known place is ever offered; the tool's settings, credentials, installed
   programs and the user's projects never are.
3. What the module recognizes and does not offer is reported as left out with its reason, never dropped.
4. No external tool runs: the module is read-only facts and paths, the cleaner moves to the Trash.

## Non-goals (each stays in its idea)

- Docker images, containers, volumes, build cache and its disk image:
  [container disk image](../../ideas/container-disk-image.md) and the Docker line of
  [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md) — always through the Docker CLI.
- Simulators, simulator runtimes, device support files, Xcode Archives (High: shipped builds' symbols): the Xcode line
  of [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md).
- `node_modules`, `target/` and other project artifacts: [stale project cleanup](../../ideas/stale-project-cleanup.md).
- Toolchain versions (rustup, nvm, pyenv, JDKs, Playwright's browsers):
  [unused toolchain versions](../../ideas/unused-toolchain-versions.md).
- Stores and repositories that are not plain caches: pnpm's store (hard-linked into projects), Maven's
  `~/.m2/repository` (may hold the only copy of what `mvn install` put there), Cargo's git checkouts (a repository
  deleted upstream does not come back), npm's `_npx`. Each is a later addition, with its own danger.

## Anti-goals

- No cache relocated by an environment variable or a tool's config (`npm_config_cache`, `CARGO_HOME`,
  `GRADLE_USER_HOME`, `GOCACHE`, …) is guessed at: only the default places, so a moved cache is missed, never mistaken.
- No age threshold and no "unused for N days": a cache is regenerable whatever its age; the cleaner's in-use check
  protects a cache being written.

## Scope and danger levels

| Category                       | Place (known folder + path)                          | Danger | Why                                                    |
| ------------------------------ | ---------------------------------------------------- | ------ | ------------------------------------------------------ |
| `dev-caches.npm`               | `Home` `.npm/_cacache`                               | Low    | downloaded again on the next install; needs the network |
| `dev-caches.yarn`              | `AppCaches` `Yarn`                                   | Low    | the same                                               |
| `dev-caches.homebrew`          | `AppCaches` `Homebrew/downloads`                     | Low    | the same; Homebrew also prunes it on its own            |
| `dev-caches.cargo`             | `Home` `.cargo/registry/cache`, `.cargo/registry/src` | Low    | the same; `src` is unpacked from `cache`                 |
| `dev-caches.cocoapods`         | `AppCaches` `CocoaPods`                              | Low    | the same                                               |
| `dev-caches.gradle`            | `Home` `.gradle/caches`                              | Low    | the same; the next build is slower                      |
| `dev-caches.pip`               | `AppCaches` `pip`                                    | Low    | the same                                               |
| `dev-caches.go-build`          | `AppCaches` `go-build`                               | Safe   | rebuilt by the next build, no download                  |
| `dev-caches.xcode-derived-data` | `Library` `Developer/Xcode/DerivedData/*`           | Safe   | rebuilt and reindexed by Xcode on its own               |

Low, not Safe, for downloads: offline the next install fails, and a private registry needs its credentials again. A
finding names its tool and, for derived data, the project (the folder's name before its hash), and counts in Xcode's
line of the storage map (`for_app`). Every place is data (`tools.toml`), never an OS path in code (rule 5).

## How a cache is recognized and judged

- **The place**: an interest `Under(known folder, path)`; the claim is the whole cache folder (`Claim::Whole`), one
  finding per folder, one per project for derived data. The Library's and the home's own folders are never claimed.
- **The tool's own sign, where it writes one**: a valid `CACHEDIR.TAG` in or next to the folder (Gradle's caches,
  Cargo's registry, Go's build cache) or a characteristic child (`index-v5` and `content-v2` in npm's `_cacache`),
  checked through the match's siblings and marker. A folder at the place without its sign is not claimed (rule 1).
- **Code, not rules** (as `jetbrains`): a pure decision table in `decide.rs`, rows checked in order, each refusal a
  left-out with its reason. Rules cannot report left-outs yet ([deferred](../../deferred/match-explanations.md)).

**Never offered** (left out with a reason, or not claimed): a link at the place (not claimed); a folder read in part
(`inside.complete` false); one holding an entry not the user's, locked or flagged (`inside.foreign` — a cache once
written with `sudo`); one holding a socket or a pipe; derived data while Xcode is open (`running_apps()`), "quit
Xcode first"; anything outside the places in the table. A cache a command-line tool holds open at removal is skipped by
the cleaner's in-use check ([`deletion-safety.md`](../../restrictions/deletion-safety.md)).

## Tool commands or the Trash

**The Trash** (`MoveToTrash` of the cache folder). The tools' own commands delete for good, which breaks "Trash
first" for no gain; `npm cache clean` requires `--force`, which the plan builder refuses; `brew cleanup` also removes
old installed versions, beyond the cache; Cargo has no stable command; a GUI app cannot rely on the user's `PATH`; and
a command adds an external program to a deletion path that needs none. Each tool recreates its cache folder on its next
run. This settles the idea's "prefer the tool's own command" for caches; toolchains decide again in their own idea.

## Group and settings

Group **Development**. No settings beyond the module's `dev-caches.enabled` switch: per-tool choice is the category's
Ignore (plan 015); there is no threshold to tune.

## Testing

- **Unit**: `decide.rs`, every row, negatives first; `tools.toml` parsed and checked (unique ids, known folders only).
- **Fixtures**: `fixtures/homes/dev-caches-*.toml` under an injected root, never the real home: look-alike names
  (`.npm/_cacache-old`, `Caches/Yarn2`), a missing or invalid sign, a link at the place, a foreign entry, Xcode open;
  then each tool's cache; golden `fixtures/golden/dev-caches.json`; the contract test with `Catalogue::Required`.
- **Sandbox**: `fixtures/sandboxes/dev-caches.toml`, negatives first; `npm run dev:sandbox -- dev-caches` and
  `EXPECTED.md` walked by hand.
- **Core e2e**: scan, confirm, to the Trash and put back on a fixture home; a cache held open is skipped `InUse`.
- **UI e2e**: the Development group with the new categories, the left-out reasons, the confirmation's danger level.
- **By hand**: a scan of the developer's Mac (scan only): every offered path is in the table, the numbers above
  match, the scan time stays within 5 % of `main` (rule 4); then one real cleanup of npm's cache and an `npm install`.

## Plans (after approval)

| #   | Plan        | What                                                                                  |
| --- | ----------- | ------------------------------------------------------------------------------------- |
| 01  | The module  | the crate, `tools.toml`, the decision and its left-outs, texts, category docs, fixtures |
| 02  | End to end  | the sandbox case file, the core e2e flows (with the shared helpers), the UI e2e        |
| 03  | Finish      | the docs, the runs by hand, the conclusions                                           |

No core change is expected: the known folders, `inside`, the `CACHEDIR.TAG` marker and `running_apps()` exist (plans
009, 016, 017). Depends on plans 015, 016 and 017 merged.

## Risks

- **A tool changes its layout** (npm's `_cacache` versions, Yarn's `v6`): the sign is then missing and nothing is
  claimed; the run by hand checks the current layouts.
- **One huge finding**: npm's cache moves as one folder; the Trash takes it on the same volume as a rename, the guard's
  growth check catches an install that ran in between.
- **A general app-caches module later** must not claim these folders twice: one claim per path, this module first.

## Deferred items due

[Shared helpers of the core e2e homes](../../deferred/e2e-shared-helpers.md) is due at the next core e2e harness:
done in plan 02. [Why an item matched](../../deferred/match-explanations.md) is not due (the module is code, not
rules); [the sandbox world](../../deferred/sandbox-world.md) is not (no open files or tools are read by the module).

## Open questions

None: every choice above is recommended and listed below for the user's check.

## Decisions

None yet.

## Agent's calls (to check)

1. **Scope: seven package-manager caches, Go's build cache and Xcode's derived data**: the biggest plain caches.
2. **pnpm's store, Maven's repository, Cargo's git checkouts and `_npx` left out**: each may not come back as is.
3. **One category per tool**: per-tool texts, danger and Ignore, no per-tool setting needed.
4. **Low for downloads, Safe for derived data and Go's build cache**: a download needs the network; a rebuild does not.
5. **The Trash, never the tool's command**: recoverable, no `--force`, no external program in the deletion path.
6. **Default places only, no environment or config read**: a moved cache is missed, never mistaken.
7. **A tool's sign required where it writes one**: the place's name alone is not enough (rule 1).
8. **Code with a data table, not rules**: left-out reasons are needed and rules cannot give them yet.
9. **Derived data left out while Xcode is open**: Xcode writes its index there continuously.
10. **No age threshold and no setting**: a cache is regenerable whatever its age; the in-use check covers writes.
11. **A folder read in part, foreign or holding a socket is left out**: the same refusals as `temp`.
12. **One finding per derived-data project, one per other cache folder**: the user chooses by project where it matters.
13. **Module id `dev-caches`, group Development.**
14. **The e2e helpers' move is done here**: its deferred item is due at this harness.

## Conclusions

Not written yet.
