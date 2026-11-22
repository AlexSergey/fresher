# 034 — Browser and web-engine caches

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Approved by the user's delegation of 2026-11-22 (build autonomously, the recommended decisions and agent's calls
taken). The design, with the facts checked on the developer's Mac and every agent's call:
[`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

Browser caches are what a person expects first from a cleaner, and apps built on a web engine keep the same kind of
cache in their support folder. Plan [030](../030-installed-app-caches/readme.md) left both out because the folders are
named after the app, not its bundle id ([deferred](../../deferred/installed-app-caches-more-places.md)).

## Goal

The module `app-caches` gains a table, `apps.toml`, that ties a place to an installed app by its bundle id and names
the exact cache folders inside it, and two categories:

| Category                | Level | What                                                                                       |
| ----------------------- | ----- | ------------------------------------------------------------------------------------------ |
| `app-caches.browsers`   | Low   | a browser's cache folders, one finding per cache folder of each profile                    |
| `app-caches.web-engine` | Low   | the cache folders inside the support folder of a known app built on a web engine           |

First rows, each checked on a real install: Google Chrome, Chromium, Firefox (`~/Library/Caches`, per profile) and
Redis Insight (`~/Library/Application Support/RedisInsight`). A name not in the table is never touched, so a profile's
or an app's data beside its cache is out of reach by construction. Emptied, the folder stays
(`Removal::EmptyFolder`), only while the app and its helpers are closed (`app-open`). Nothing in the core, the scanner,
the cleaner or the engine changes; no new dependency, no new UI component. The walk gains eleven anchored interests and
no new walked place, so no scan bench is due ([`../../testing.md`](../../testing.md), "Performance").

Branch: `feat/034-browser-caches`, from `main` (97d9b314), rebased onto plan 033 before it is built, squash-merged
into `main` once when done ([`../../workflow.md`](../../workflow.md)). Depends on plan
[030](../030-installed-app-caches/readme.md).

## Plans

| #   | Plan                                    | Status      | Depends on | What                                                                           |
| --- | --------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------ |
| 01  | [The table](./01-table/readme.md)       | 🔨 in progress | —          | the table, the two categories, the owner and the decision, fixtures, the texts |
| 02  | [End to end](./02-end-to-end/readme.md) | 🔨 in progress | 01         | case files, golden reports, the core e2e flow, the UI                          |
| 03  | [Finish](./03-finish/readme.md)         | 🔨 in progress | all        | docs, the deferred item narrowed, the review list, the conclusions             |

### Order

```
01/01 ── 01/02 ── 01/03 ──┬── 02/01 ──┬── 03/01
                          ├── 02/02 ──┤
                          └── 02/03 ──┘
```

Steps run in the order of the table, each leaving the workspace green; 02/01, 02/02 and 02/03 touch different files
and may run in parallel (02/01 and 02/02 both add a paragraph to `docs/testing.md`: the second rebases).

| Plan | Owns                                                                                                                    | Shared files it touches                                                                                 |
| ---- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| 01   | `modules/app-caches/` (its new `apps.toml`, `src/apps.rs`, `src/apps/tests.rs`), `fixtures/homes/app-caches-tables-*.toml`, `fixtures/golden/app-caches-tables.json`, `docs/categories/app-caches-{browsers,web-engine}.md` | `docs/categories/README.md`                                                                             |
| 02   | `fixtures/sandboxes/app-caches-browsers{,-open}.toml`, `fixtures/golden/sandbox-app-caches-browsers*.json`, `fixtures/homes/app-caches-browsers-flow.toml`, `fixtures/golden/app-caches-browsers-flow.json`, `crates/e2e/tests/{app_caches_browsers.rs,app_caches_browsers_home/}`, `app/ui/src/test/app-caches-browsers-scan.ts` | `app/tauri/tests/sandboxes.rs`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 03   | `docs/deferred/user-review-2026-11-22.md` (its section)                                                                 | `docs/` (architecture, categories, testing, manual-testing, ideas, deferred, plans README)              |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage at or above 90 % of lines in Rust and the UI;
  `modules/app-caches` itself at or above 90 %.
- `git diff $(git merge-base HEAD main) -- crates/core crates/scanner crates/cleaner crates/engine` is empty.
- The module's contract test passes with `Catalogue::Required` for its three categories; every text and left-out is
  used by a fixture run; every removal is an `EmptyFolder` of a matched path.
- Nothing but a cache folder named in the table, of an installed app, is offered: a profile's history and cookies, an
  app's own data, a name not in the row, a link, a file, an empty folder, a not-installed browser's cache and an
  unlisted app's cache-named folder are never claimed (fixtures, case files, core e2e).
- `npm run sandbox:test` passes for both new case files and every other.
- `npm run test:e2e` passes with the new core flows and UI scenarios.
- The user's walks (`npm run dev:sandbox -- app-caches-browsers`, `-- app-caches-browsers-open`, a dry run on the real
  Mac) are listed in [`docs/deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md).

## Ideas this touches

[Caches of installed apps](../../ideas/installed-app-caches.md); the deferred item
[More places of installed apps' caches](../../deferred/installed-app-caches-more-places.md) is narrowed (03/01).

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken.
- 2026-11-22 — settled by the planner (agent's calls to check, [`00-design.md`](./00-design.md)): one finding per cache
  folder rather than per profile (call 5: `EmptyFolder` names one path, and an emptied item of several folders would be
  a core change); only the rows checked on the developer's Mac ship, the other well-known browsers and web-engine apps
  wait for a check (call 4); the browsers' small support-folder graphics caches are left out (call 6).

## Agent's calls (to check)

1. **Caches only, by exact name from a table**; never history, cookies or sessions.
2. **Known apps only**; no generic finder.
3. **Safari stays deferred.**
4. **Part of `app-caches`**, not a new module.
5. **One finding per cache folder**, titled by browser, profile and folder.
6. **Only rows checked on a real install**: Chrome, Chromium, Firefox, Redis Insight.
7. **No browser support-folder caches** (small; protected without Full Disk Access).

The full list with costs: [`00-design.md`](./00-design.md), "Agent's calls (to check)".

## Conclusions

**Built.** The module `app-caches` reads a second table, `apps.toml`, that ties a place named after an app to the app
by its bundle id and names the exact cache folders in it, and reports two new categories, `app-caches.browsers` and
`app-caches.web-engine` (both Low, group System), after `library-caches`. Four rows ship, each checked on the
developer's Mac: Chrome, Chromium and Firefox (`~/Library/Caches`, per profile) and Redis Insight
(`~/Library/Application Support/RedisInsight`). The owner of a row is the installed app with the row's bundle id; the
decision is plan 030's eleven rows (rows 2 and 4 by the table). One finding per cache folder, emptied
(`Removal::EmptyFolder`), only while the app and its helpers are closed. One new text, `app-caches.title.profile`, and
ten category texts. Nothing changed in the core, the scanner, the cleaner, the engine or `app/tauri/src`; no new
dependency or component.

**Numbers.** `modules/app-caches`: 139 tests, 92 to 100 % of lines per file (`lib.rs` 92.07, `report.rs` 94.01, the
rest above 98). `cargo nextest run -p fresher-app --test sandboxes`: 63; `npm run sandbox -- all`: 214 cases, the same
5 files left out as before the plan. Core e2e: 14 tests of the module's two files, `npm run test:e2e:core` 144 passed
and 2 skipped. UI: vitest 1160 tests, 97.76 % of lines; e2e 552 passed, one webkit hover flake that passed on rerun.
`npm run pre-push` on the branch's last commit: passed (Rust 3887 tests, workspace line coverage 97.54 %; UI lines
98.60 %, 97.76 % of statements). By-hand mutations: of the case files (a danger level changed,
a left-out's items changed: both fail their autotest) and of the core e2e (a widened Chrome path fails 5 of 6 flows,
`image_cache` added to the caches fails all 6).

**Changes from the design.**

- `AppRow.profile` is an `Option<usize>`, not a `bool` (the step's contract).
- The second case file puts Redis Insight at `Applications/RedisInsight.app`: with the design's path `npm run sandbox --
  all` left the file out whole, as it wrote a path an earlier file wrote.
- A non-UTF-8 profile or cache name suppresses only the finding; a left-out is still reported, as in `library-caches`.
- The test support of the module was split in three files (`mod`, `plain`, `tables`), since one shared file broke
  `dead_code` in every test target.
- The UI mock's cleanup empties `Cache` (4 moved) and `Code Cache` (0 moved) to test the "Emptied" row.

**Calls taken while building.**

- The planner wrote the steps from a fresh read of the code, with no separate preflight scan; the agent's calls of the
  draft (one finding per cache folder, only four rows, no support-folder caches) were accepted on the user's
  delegation.
- Plan 034 was built in its own worktree beside plan 033, with its own cargo target directory (the shared one gave
  stale artifacts across worktrees); it rebases onto `main` after 033 merges, and the shared files (the tauri lists,
  the docs) are resolved then, keeping a section per plan in the review list.
- The same sandbox world made Chrome the only open browser, Chromium the closed one, Firefox never installed.

**What the real Mac must still show** (the user's walks, in
[`../../deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md)): the two sandbox walks, the dry
run, Redis Insight open and closed, a browser's next start after an emptied `Cache`, Firefox's `startupCache`. Worth
the user's look: only four rows ship (Edge, Brave, Vivaldi, Arc, Visual Studio Code, Slack, Discord and others wait for
a check), one finding per cache folder rather than per profile, the browsers' support-folder caches left out (they need
Full Disk Access to verify).

**Known gaps.** A webkit hover flake in `scan.spec` (passed on rerun) and the known flaky chromium baseline
`category-details-app-leftovers-chosen`. The status stays 🔨 until the user has walked the plan.
