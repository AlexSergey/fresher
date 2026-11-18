# 018 — Developer caches

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, scanner, testing, modules, sandbox, e2e, app, app/ui, docs

Drafted on 2026-11-18 as one file and approved by the user the same day as written, with its fourteen agent's calls,
which became decisions ("approve, write the design and start the implementation"). The design, its decisions and the
agent's calls made while writing it from the code are in [`00-design.md`](./00-design.md); the tools' facts marked "to
verify" are checked by hand, read-only, in 04/02.

## Goal

The download caches of package managers (npm, Yarn, Homebrew, Cargo, CocoaPods, Gradle, pip), Go's build cache and
Xcode's derived data — the largest regenerable folders on a developer's Mac that nobody looks at (on the developer's
Mac on 2026-11-18: npm's cache 8.9 GB, Gradle's 1.7 GB, Cargo's registry 1.4 GB, Homebrew's 0.5 GB) — are found at each
tool's default place, recognized by the tool's own sign, explained in plain English (which tool, what comes back and
what it costs) and offered to the Trash: one category per tool, Low for a download, Safe for a rebuild. What is
recognized and not offered (a cache holding a socket, a cache not the user's, derived data while Xcode is open) is
reported as left out with its reason. No tool's command runs. The module is `modules/dev-caches`
(`fresher-module-dev-caches`), in the group Development.

The core first gains what the code lacks: a matched folder reports which names it holds, so a sign inside a cache
folder (npm's `index-v5`, Gradle's `CACHEDIR.TAG`) can be checked; the tree vocabulary gains a valid `CACHEDIR.TAG`.
Tests at every level, as plans [016](../016-jetbrains/readme.md) and [017](../017-temp-files/readme.md) did: unit,
fixture, sandbox, core e2e, UI e2e, runs by hand.

Branch: `feat/018-developer-caches` (from `main` 002ed0c), squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md),
[009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md), [012](../012-storage-map/readme.md),
[015](../015-module-groups/readme.md), [016](../016-jetbrains/readme.md) and [017](../017-temp-files/readme.md), all
in `main`.

## Plans

| #   | Plan                                                    | Status      | Depends on | What                                                                                          |
| --- | ------------------------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------------------- |
| 01  | [Core prerequisites](./01-core-prerequisites/readme.md) | ✅ approved | —          | what a matched folder holds: `Interest::holding`, `Match.holds`, a held tag's validity        |
| 02  | [The module](./02-module/readme.md)                     | ✅ approved | 01         | the crate, `tools.toml`, the categories and texts, the decision and its left-outs, fixtures, registration |
| 03  | [End to end](./03-end-to-end/readme.md)                 | ✅ approved | 01, 02     | the three case files, the shared e2e helpers and the core flows, the UI scenarios             |
| 04  | [Finish](./04-finish/readme.md)                         | ✅ approved | all        | the docs, the runs by hand, the conclusions                                                   |

### Order and parallel work

```
01/01 ── 02/01 ──┬── 02/02 ── 02/03 ──┬── 03/01 ──┬── 04/01 ── 04/02
                 │                    └── 03/02 ──┤
                 └── 03/03 ───────────────────────┘
```

- **Wave 1**: 01/01 alone (`crates/core`, `crates/scanner`, `crates/testing/src/naive_walk.rs`).
- **Wave 2**: 02/01 (the crate's descriptor and interests need `Interest::holding`).
- **Wave 3, in parallel**: 02/02 (`modules/dev-caches/src/` only) and 03/03 (`app/ui` only, once 02/01's texts exist).
- **Wave 4**: 02/03 (fixtures, the tag in the tree vocabulary, registration).
- **Wave 5, in parallel**: 03/01 (case files, `app/tauri/tests/sandboxes.rs`) and 03/02 (`crates/e2e`).
- **Wave 6**: 04.

Which files each plan touches:

| Plan | Owns                                                                                                         | Shared files it touches                                                                                                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | —                                                                                                            | `crates/core/src/{interest,lib}.rs`, `crates/scanner/src/{matcher/,walk/,tree/,codec/,rescan/}`, `crates/testing/src/naive_walk.rs`, the scanner's tests and goldens, `docs/architecture/modules.md` |
| 02   | `modules/dev-caches/`, `fixtures/homes/dev-caches-*.toml`, `fixtures/golden/dev-caches.json`, `docs/categories/dev-caches-*.md` | root `Cargo.toml` (the member), `Cargo.lock`, `crates/testing/src/tree_description.rs`, `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` (`modules()`), `docs/categories/README.md`, `docs/dependencies.md` |
| 03   | `fixtures/sandboxes/dev-caches*.toml`, `fixtures/golden/{sandbox-dev-caches*,dev-caches-flow}.json`, `fixtures/homes/dev-caches-flow.toml`, `crates/e2e/tests/{dev_caches.rs,dev_caches_home/}`, `app/ui/src/test/dev-caches-scan.ts` | `crates/e2e/Cargo.toml`, `crates/e2e/tests/{support/,jetbrains_home/,temp_home/,places.rs,cleaning/}`, `app/tauri/tests/sandboxes.rs` (`case_files!`), `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/deferred/` |
| 04   | —                                                                                                            | `docs/` (architecture, testing, categories, ideas, plans README)                                                                                                                         |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/dev-caches` itself at or above 90 %.
- The scan bench moves by no more than 5 % with plan 01 (rule 4), written in 01/01's conclusions; the scan of the
  developer's home stays within 5 % of `main`'s (04/02).
- The module's contract test passes with `Catalogue::Required`: nine category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every removal a matched path.
- Nothing outside the table's ten places can be offered: the look-alike, settings, credentials, programs and stores
  negatives of the fixtures and the case files prove it; a place without its sign is never claimed.
- `npm run sandbox:test` passes for the three case files; each fails when a case's expectation (a danger, a left-out's
  items, a sign removed from a positive) is changed on purpose (done once by hand per kind, written in 03/01).
- `npm run test:e2e` passes with the eight core flows and the six UI scenarios; the deferred
  shared helpers are moved and the item removed.
- `npm run dev:sandbox -- dev-caches` (and the two other files): every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, scan only): every offered item is one of the table's places with its sign;
  the sizes match `du`'s; nothing of Cargo's index, git checkouts, `bin` or credentials, npm's `_npx`, Gradle's wrapper
  or Homebrew's `api` is claimed.

## Ideas and deferred items this touches

- [Developer Environment Cleaner](../../ideas/developer-environment-cleaner.md): its package-manager and DerivedData
  lines become this plan; Docker, simulators, device support, Archives, pnpm's store, Maven's repository and Cargo's
  git caches stay there.
- [Unused toolchain versions](../../ideas/unused-toolchain-versions.md): decides again whether to use a tool's command.
- [Stale project cleanup](../../ideas/stale-project-cleanup.md): `node_modules`, `target/`; may reuse `holds` for a
  project's signs.
- The deferred shared helpers of the core e2e homes: done in 03/02.

## Risks

- **A tool changes its layout** (npm's `_cacache` versions, Yarn's `v6`): the sign is then missing and nothing is
  claimed; the run by hand checks the current layouts.
- **One huge finding**: npm's cache moves as one folder; the Trash takes it on the same volume as a rename, and the
  guard's growth check catches an install that ran in between.
- **A general app-caches module later** must not claim these folders twice: one claim per path, this module first.
- **The walk's hot path** (plan 01): the mask is built only for matched folders of interests that hold names; the
  bench says.

## Conclusions

Not written yet.
