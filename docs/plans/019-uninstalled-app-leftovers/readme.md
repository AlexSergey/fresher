# 019 — Uninstalled app leftovers

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, scanner, macos, system, engine, testing, modules, sandbox, e2e, app, app/ui, docs

Drafted on 2026-11-18 as one file and approved by the user the same day with its sixteen agent's calls, which became
decisions, and two answers: group containers are left out of this plan; an app's support folder and container are
offered in this plan, as High, in their own category ("approve, write the design and start the implementation").
On 2026-11-18 the user also decided that the "Remove everything / Keep settings" choice is per module for now (the
recommended option); a choice per app stays an idea. The
design, its decisions and the agent's calls made while writing it from the code are in
[`00-design.md`](./00-design.md); the macOS facts marked "to verify" are checked by hand, read-only, in 04/02.

## Goal

Dragging an app to the Trash removes only its bundle; its caches, logs, saved windows, settings, web data, support
folder and container stay in `~/Library` for good. The module `app-leftovers` (`fresher-module-app-leftovers`, group
**Apps**) finds the data of apps that are provably no longer installed — matched by bundle id only, an id proven an
app's by its container or saved state, not installed, not running, not found by Spotlight on any indexed volume,
unchanged for 30 days (a setting) — explains it in plain English and offers it to the Trash in three categories: Safe
caches, Medium settings, High data. Two choices on its page select the categories: _Remove everything_ and _Keep
settings_. What it recognizes as a leftover and does not offer is reported as left out with its reason. It generalizes
plan [016](../016-jetbrains/readme.md)'s removed IDEs to every app. The idea:
[uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md).

The core first gains what the code lacks: interests that match only bundle-id names (so a whole claim of
`Caches/*` never swallows `Caches/JetBrains` or `Caches/Homebrew`), what the walk saw of each interest's place (the
global check would block the module on every Mac without Full Disk Access), a check that Spotlight can see every
mounted volume (closing a [known issue](../../known-issues/app-spotlight-misses-an-excluded-volume.md) of `jetbrains`
too), a removed app's name in the storage map (a deferred item, done in 01/04), the group Apps
and a module's selection choices. Tests at every level, as plans 016 to 018 did: unit, fixture, sandbox, core e2e, UI
e2e, runs by hand.

Branch: `feat/019-uninstalled-app-leftovers` (from `main` 2aa9f67), squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md),
[009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md), [012](../012-storage-map/readme.md),
[015](../015-module-groups/readme.md), [016](../016-jetbrains/readme.md), [017](../017-temp-files/readme.md) and
[018](../018-developer-caches/readme.md), all in `main`.

## Plans

| #   | Plan                                                    | Status      | Depends on | What                                                                                                   |
| --- | ------------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------------------------ |
| 01  | [Core prerequisites](./01-core-prerequisites/readme.md) | ✅ approved | —          | bundle-id names, the places seen, Spotlight's reach, a removed app's name, the group Apps, the choices |
| 02  | [The module](./02-module/readme.md)                     | ✅ approved | 01         | the crate, `places.toml`, `exclude.toml`, the categories, texts and choices, the decision, fixtures, registration |
| 03  | [End to end](./03-end-to-end/readme.md)                 | ✅ approved | 01, 02     | the sandbox's Spotlight answers and case files, the shared e2e home and the flows, the UI               |
| 04  | [Finish](./04-finish/readme.md)                         | ✅ approved | all        | the docs, the runs by hand, the conclusions                                                            |

### Order and parallel work

```
01/01 ── 01/02 ──────────────┐
01/03 ───────────────────────┼── 02/02 ── 02/03 ──┬── 03/01 ──┬── 04/01 ── 04/02
01/04 ──┬── 02/01 ───────────┘                    └── 03/02 ──┤
        │     └── 03/03 ───────────────────────────────────────┘
```

- **Wave 1, in parallel**: 01/01 (`crates/core/src/{interest,system/ids}.rs`, `crates/scanner/src/matcher/`,
  `crates/testing/src/naive_walk.rs`), 01/03 (`crates/core/src/system/`, `crates/macos/src/volume.rs`,
  `crates/system/`, `crates/testing/src/system_fakes.rs`, `app/tauri/src/{compose,sandbox}.rs`) and 01/04
  (`crates/core/src/{finding,group,module,command}.rs`, `crates/engine/src/storage/`, `modules/jetbrains/src/lib.rs`,
  `app/ui/src/product/module-groups.ts`).
- **Wave 2, in parallel**: 01/02 after 01/01 (`crates/core/src/{walk,module}.rs`, `crates/scanner/src/`,
  `crates/engine/src/session.rs`, `crates/testing/src/{naive_walk,run_module}.rs`) and 02/01 after 01/01's
  `only_bundle_ids` and 01/04's group and choices (`modules/app-leftovers/` only, beside the catalogue).
- **Wave 3, in parallel**: 02/02 after 01/02 and 01/03 (`modules/app-leftovers/src/` only) and 03/03 after 01/04 and
  02/01 (`app/ui` only).
- **Wave 4**: 02/03 (fixtures, golden, registration).
- **Wave 5, in parallel**: 03/01 (`crates/sandbox`, case files, `app/tauri/src/sandbox.rs`, `app/tauri/tests/`) and
  03/02 (`crates/e2e`).
- **Wave 6**: 04.

`crates/core/src/lib.rs` and `crates/testing/src/lib.rs` take re-exports from several wave 1 steps: each step only
adds its own lines, and a merge keeps both sides.

Which files each plan touches:

| Plan | Owns                                                                                                          | Shared files it touches                                                                                                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/system/src/spotlight.rs`                                                                              | `crates/core/src/{interest,walk,module,finding,group,command,lib}.rs`, `crates/core/src/system/`, `crates/core/locales/en.json`, `crates/scanner/src/{matcher/,scanner.rs,places_seen.rs,bench.rs}`, `crates/scanner/tests/reference.rs`, `crates/macos/src/{volume,lib}.rs`, `crates/system/src/{apps,lib}.rs`, `crates/engine/src/{session.rs,storage/}`, `crates/testing/src/{naive_walk,run_module,system_fakes,contract,lib}.rs`, `modules/jetbrains/`, `fixtures/golden/jetbrains.json`, `app/tauri/src/{compose,sandbox}.rs`, `app/ui/src/{ipc/bindings.generated.ts,product/module-groups.ts}`, `docs/architecture/modules.md`, `docs/categories/jetbrains-uninstalled.md`, `docs/deferred/`, `docs/known-issues/` |
| 02   | `modules/app-leftovers/`, `fixtures/homes/app-leftovers-*.toml`, `fixtures/golden/app-leftovers.json`, `docs/categories/app-leftovers-*.md` | root `Cargo.toml` (the member), `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` (`modules()`), `docs/categories/README.md`, `docs/dependencies.md`                       |
| 03   | `fixtures/sandboxes/app-leftovers*.toml`, `fixtures/golden/{sandbox-app-leftovers*,app-leftovers-flow}.json`, `fixtures/homes/app-leftovers-flow.toml`, `crates/e2e/tests/{app_leftovers.rs,app_leftovers_home/,support/home.rs}`, `app/ui/src/test/app-leftovers-scan.ts`, `app/ui/src/screens/module/module-choices.tsx` | `crates/sandbox/src/{case,world,expected}.rs`, `app/tauri/src/sandbox.rs`, `app/tauri/tests/{sandboxes.rs,sandboxes/harness.rs}`, `crates/e2e/Cargo.toml`, `crates/e2e/tests/{temp_home/,dev_caches_home/}`, `app/ui/src/{state/selection.ts,screens/module/module-screen.tsx}`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md`, `docs/deferred/` |
| 04   | —                                                                                                             | `docs/` (architecture, testing, categories, ideas, plans README)                                                                                                                        |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/app-leftovers` itself at or above 90 %.
- The scan bench moves by no more than 5 % with plan 01 (rule 4), written in 01/01's conclusions with a shaped interest
  in the bench; the scan of the developer's home stays within 5 % of `main`'s (04/02); the bench is retaken on a quiet
  Mac (04/02).
- The module's contract test passes with `Catalogue::Required`: three category files with the same dangers, every
  text, every choice's two texts and every declared left-out reason used by a fixture run, every removal a matched path.
- Nothing but an id-named leftover of a proven, removed, unseen, unchanged app can be offered: the not-ids, Apple,
  unproven, installed and kept negatives of the fixtures and the case files prove it; `jetbrains`' and `dev-caches`'
  fixtures and goldens are unchanged by the shaped interests.
- `jetbrains` keeps a removed IDE's data when a mounted volume is not indexed; the known issue is closed.
- `npm run sandbox:test` passes for both case files; each fails when a case's expectation (a danger, a left-out's
  items, a saved state removed from a positive) is changed on purpose (done once by hand per kind, written in 03/01).
- `npm run test:e2e` passes with the eleven core flows and the UI scenarios; the shared home harness is in
  `crates/e2e/tests/support/home.rs` and its deferred item removed.
- `npm run dev:sandbox -- app-leftovers` (and the other file): every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, scan only): every offered id is no installed app (`mdfind`, by hand); nothing
  under `com.apple.` or `Group Containers` is offered.

## Ideas and deferred items this touches

- [Uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md): becomes this plan, the vendor-uninstaller
  question included (the same rows cover it); group containers, launch agents, folders named after the app's name and
  a per-app choice stay there. [Smart uninstaller](../../ideas/smart-uninstaller.md) stays an idea.
- A removed app's name in the storage map (a deferred item): done in 01/04; its file is removed.
- The module harnesses of the core e2e share their code (a deferred item): done in 03/02; its file is removed.
- [The sandbox world](../../deferred/sandbox-world.md): Spotlight's answers taken in 03/01; the rest stays deferred.
- The known issue [Spotlight misses an excluded volume](../../known-issues/app-spotlight-misses-an-excluded-volume.md):
  closed in 01/03.

## Risks

- **The worst false positive is a High support folder of an app the checks missed**: rows 2 to 11 all must pass, the
  user confirms it with High shown, and it goes to the Trash.
- **Containers are not opened until plan 006** (agent's call 4): until then the proof is a saved state only, and the
  data category holds support folders only.
- **Too few proven ids**: apps that are not sandboxed and never saved a window are missed; the run by hand counts how
  much the proof leaves out, and a later proof is weighed then.
- **Whole claims hide the inside of id-named folders in the map's browse view** (agent's call 6), and a later module
  whose place lies inside one (the Docker disk image) must stop the claim there.
- **A mounted disk image or network share keeps everything** while mounted (agent's call 7): safe; the left-out text
  says what to do.
- **The walk's hot path** (01/01): the shape check runs only for names under a node with a shaped branch; the bench says.

## Conclusions

Not written yet.
