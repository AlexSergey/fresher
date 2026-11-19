# 020 — Downloads and installers

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, scanner, system, engine, testing, modules, sandbox, e2e, app, app/ui, docs

Drafted on 2026-11-19 as one file and approved by the user the same day with its ten agent's calls, which became
decisions ("start implementing, work autonomously, take the recommended decisions"). The design, its decisions and the
agent's calls made while writing it from the code and from a read-only look at this Mac are in
[`00-design.md`](./00-design.md); the macOS facts marked "to verify" are checked by hand in 05/02.

## Goal

The Downloads folder collects what the user thinks of as temporary and nothing removes: downloads that never finished,
Safari downloads that were never resumed, installers of apps that are installed now, old macOS installers, big files
downloaded once and forgotten. The module `downloads` (`fresher-module-downloads`, group **Your files**) reads
`~/Downloads`, one level deep plus bundles, only once the user allowed the folder, and offers them in five categories:
unfinished downloads (Low), Safari's resumable downloads (Medium), installers of installed apps (Low), old macOS
installers (Low) and big files not opened for 30 days (a setting; High, the user's call). An installer is claimed only
when its receipt or its name proves it is an installed app's, at that version or older; a disk image is never mounted,
an installer never run. What the module recognizes and keeps is reported as left out with its reason. The idea:
[downloads and installers](../../ideas/downloads-and-installers.md).

The core first gains what the code lacks: the day setting once (a deferred item), a `*` that matches only names with
a suffix (so the user's folders in Downloads stay open), an installed app's change time, the installer's receipts and
the system's version, a module that needs a guarded folder running in the scan that first tries it, and the group
**Your files**. The sandbox learns access, installs and an app's change time (a part of a deferred item). Tests at
every level, as plans 016 to 019 did: unit, fixture, sandbox, core e2e, UI e2e, runs by hand.

Branch: `feat/020-downloads` (from `main` 5c61c57), squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [006](../006-permissions/readme.md),
[008](../008-safe-removal/readme.md), [009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md),
[012](../012-storage-map/readme.md), [015](../015-module-groups/readme.md) and
[019](../019-uninstalled-app-leftovers/readme.md), all in `main`.

## Plans

| #   | Plan                                                    | Status      | Depends on | What                                                                                                      |
| --- | ------------------------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------------------------------- |
| 01  | [Core prerequisites](./01-core-prerequisites/readme.md) | ✔️ done | —          | the shared day setting, suffixed names, an app's change time and the install records, availability after the places, the group Your files |
| 02  | [The sandbox's world](./02-sandbox-world/readme.md)     | ✔️ done     | 01         | access in the world and the banner; installs and an app's change time in the world                         |
| 03  | [The module](./03-module/readme.md)                     | ✔️ done     | 01         | the crate, its tables and texts, the decision, fixtures, registration                                      |
| 04  | [End to end](./04-end-to-end/readme.md)                 | ✔️ done     | 01, 02, 03 | the three case files, the core e2e flows, the UI                                                           |
| 05  | [Finish](./05-finish/readme.md)                         | ✅ approved | all        | the docs, the runs by hand, the conclusions                                                                |

### Order and parallel work

```
01/01, 01/02, 01/05 ── 03/01 ──┐
01/03 ─────────────────────────┴── 03/02 ── 03/03 ──┐
01/04 ── 02/01 ──┬──────────────────────────────────┼── 04/01, 04/02, 04/03 ── 05/01 ── 05/02
01/03 ───────────┴── 02/02 ─────────────────────────┘
```

- **Wave 1, in parallel**: 01/01 (`crates/core/src/settings/`, `modules/{jetbrains,app-leftovers,temp}/src/`),
  01/02 (`crates/core/src/interest.rs`, `crates/scanner/src/matcher/`, `crates/scanner/benches/scan.rs`,
  `crates/scanner/tests/reference.rs`, `crates/testing/src/naive_walk.rs`), 01/03 (`crates/core/src/system/`,
  `crates/system/src/{apps,installs,lib}.rs`, `crates/testing/src/{system_fakes,run_module}.rs`, every
  `SystemPorts {` and `InstalledApp {` site), 01/04 (`crates/engine/src/{session,coverage,access}.rs`) and 01/05
  (`crates/core/src/group.rs`, `crates/core/locales/en.json`, `app/ui/src/product/module-groups.ts`, the bindings).
- **Wave 2, in parallel**: 02/01 after 01/04 (`crates/sandbox/src/{case,world,expected}.rs`,
  `crates/core/src/app_info.rs`, `app/tauri/src/{sandbox,adapters}.rs`, `app/tauri/tests/sandboxes/harness.rs`,
  `app/ui/src/product/build-banner/`) and 03/01 after 01/01, 01/02 and 01/05 (`modules/downloads/` only, beside
  the catalogue).
- **Wave 3, in parallel**: 02/02 after 01/03 and 02/01 (the same sandbox files,
  `crates/testing/src/{stated_apps,tree_description}.rs`, `app/tauri/src/compose.rs`) and 03/02 after 01/03 and 03/01
  (`modules/downloads/src/` only).
- **Wave 4**: 03/03 (fixtures, golden, registration).
- **Wave 5, in parallel**: 04/01 (case files, `app/tauri/tests/sandboxes.rs`), 04/02 (`crates/e2e`) and 04/03
  (`app/ui`).
- **Wave 6**: 05.

`crates/core/src/lib.rs`, `crates/testing/src/lib.rs` and `crates/core/locales/en.json` take lines from several wave
1 steps: each step only adds its own, and a merge keeps both sides. 01/03 touches the 14 `SystemPorts {` sites and
the 26 `InstalledApp {` sites, some of them in the modules 01/01 edits: when both run in parallel worktrees, the second
merge keeps both sides.

Which files each plan touches:

| Plan | Owns                                                                                                                    | Shared files it touches                                                                                                                                                                 |
| ---- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/settings/days.rs`, `crates/system/src/installs.rs`                                                     | `crates/core/src/{interest,group,lib}.rs`, `crates/core/src/system/`, `crates/core/locales/en.json`, `crates/scanner/src/matcher/`, `crates/scanner/{benches/scan.rs,tests/reference.rs}`, `crates/system/src/{apps,lib}.rs`, `crates/engine/src/{session,coverage,access,facts_tests}.rs`, `crates/engine/src/storage/`, `crates/testing/src/{naive_walk,system_fakes,run_module,lib}.rs`, `crates/e2e/tests/`, `modules/{jetbrains,app-leftovers,temp}/`, `app/tauri/src/compose.rs`, `app/ui/src/{ipc/bindings.generated.ts,product/module-groups.ts}`, `docs/architecture/`, `docs/deferred/` |
| 02   | `crates/testing/src/stated_apps.rs`                                                                                     | `crates/sandbox/src/{case,world,expected}.rs`, `crates/testing/src/{tree_description,lib}.rs`, `crates/core/src/app_info.rs`, `app/tauri/src/{sandbox,compose,adapters}.rs`, `app/tauri/tests/sandboxes/harness.rs`, `app/ui/src/product/build-banner/`, `app/ui/src/locales/en.json`, `docs/testing.md`, `docs/hacks/README.md`, `docs/architecture/permissions.md`, `docs/deferred/` |
| 03   | `modules/downloads/`, `fixtures/homes/downloads-*.toml`, `fixtures/golden/downloads.json`, `docs/categories/downloads-*.md` | root `Cargo.toml` (the member), `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` (`modules()`), `docs/categories/README.md`, `docs/dependencies.md`                       |
| 04   | `fixtures/sandboxes/downloads*.toml`, `fixtures/golden/{sandbox-downloads*,downloads-flow}.json`, `fixtures/homes/downloads-flow.toml`, `crates/e2e/tests/{downloads.rs,downloads_home/}`, `app/ui/src/test/downloads-scan.ts` | `app/tauri/tests/sandboxes.rs`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md`, `docs/known-issues/` |
| 05   | —                                                                                                                       | `docs/` (architecture, testing, categories, ideas, plans README)                                                                                                                        |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/downloads` itself at or above 90 %.
- The scan bench moves by no more than 5 % with 01/02 (rule 4), written in its conclusions with a suffixed interest in
  the bench; the scan of the developer's home stays within 5 % of `main`'s (05/02).
- `jetbrains`, `app-leftovers` and `temp` keep their settings' keys, ranges and defaults; their tests and goldens pass
  with the shared day setting, and the deferred item is removed.
- The module's contract test passes with `Catalogue::Required`: five category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every removal a matched path.
- Nothing in Downloads is read unless the user allowed it: Full Disk Access, or the Downloads switch (the core e2e
  asserts no `try_place` and no match before the opt-in, and none with the switch off after an earlier grant), and
  the module runs in the first scan after the opt-in.
- Nothing but a proven item can be offered: a document, a disk image of no installed app, a copy of an installer, a
  link, a folder of the user's and anything below the first level are never claimed (the fixtures' and the case
  files' negatives).
- `npm run sandbox:test` passes for the three case files; each fails when a case's expectation (a danger, a left-out's
  items, an access value) is changed on purpose (done once by hand per kind, written in 04/01).
- `npm run test:e2e` passes with the nine core flows and the UI scenarios.
- `npm run dev:sandbox -- downloads` (and the two other files): every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, Downloads allowed, scan only): every offered item checked by hand (05/02).

## Ideas and deferred items this touches

- [Downloads and installers](../../ideas/downloads-and-installers.md): becomes this plan; looking outside
  `~/Downloads` stays there.
- [Large files finder](../../ideas/large-files-finder.md): may reuse the large-untouched rule (the status change as
  "not opened").
- [Duplicate finder](../../ideas/duplicate-finder.md): copies of a download (`file (1).dmg`) are left to it.
- [Smart uninstaller](../../ideas/smart-uninstaller.md): macOS installers in `/Applications` are left to it.
- [The Trash holds the freed space](../../ideas/trash-holds-freed-space.md): installers are large; the result already
  says "moved to the Trash".
- A shared day setting in the core (a deferred item): done in 01/01; its file is removed.
- [The sandbox world](../../deferred/sandbox-world.md): access, installs and an app's change time taken in 02; the
  rest stays deferred.

## Risks

- **The worst false positive is a High big file the user still needs**: it is offered only after 30 days without a
  change or an open from Finder, the user confirms it with High shown, and it goes to the Trash.
- **An installer of a newer version than the installed app** (agent's call 2): a migration or a system update moves
  every app's change time at once, so the date check can pass; the version in the name catches most, the rest is Low
  and downloads again.
- **Few `.pkg` files proven by receipt** (agent's call 3): the name rule carries most of them.
- **The browsers' ids and the macOS installers' names are data** (agent's call 15): a wrong id weakens one guard, the
  days remain; a missing name keeps an installer.
- **The trie's hot path** (01/02): the suffix check runs only for names under a node with a suffix branch; the bench
  says.
- **The first-scan order change in the engine** (01/04) touches every scan: the engine's coverage and access tests
  run unchanged, plus the new ones.

## Conclusions

Not written yet.
