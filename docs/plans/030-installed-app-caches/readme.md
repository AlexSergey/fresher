# 030 — Installed app caches

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-21 from the idea [Caches of installed apps](../../ideas/installed-app-caches.md) and from a read of
the code on plan 024's design commit (39ec314f). Approved by the user's choice of 2026-11-21: after plans 024 and 029,
build the caches of installed apps autonomously, take the recommended decision on every question and leave each as an
agent's call; the user checks everything later. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Goal

The apps the user still uses keep caches nobody empties. The module `app-caches` (`fresher-module-app-caches`, group
**System**) offers, one finding per folder, the folders `~/Library/Caches/<bundle id>` that an installed app provably
owns (its id, a helper's, or an id under them), in one category, **Caches of your apps** (Low). Each finding names the
app and the size it frees and is counted in the app's line of the storage map. A cache is offered only while its app
and its helpers are closed. The folder is **emptied** into the Trash and stays (`Removal::EmptyFolder`, plan 008's,
here used by a real module for the first time). Nothing of Apple's, nothing of Fresher's own, and nothing whose owner
cannot be told by bundle id is claimed.

**One claim per folder**: `app-caches` claims exactly the id-named folders that `app-leftovers` (plan 019) leaves
unclaimed because an installed app owns them, from the same installed apps of the same scan. The other modules of
`Library/Caches` (`dev-caches`, `jetbrains`) claim names that are not ids.

**Not in this plan** (agent's call 1): the web-engine caches inside `Application Support/<name>`, browsers' and game
launchers' caches (named after the app's name, not its id), the Metal shader cache (a protected place), containers and
Apple's apps. They are [deferred](../../deferred/installed-app-caches-more-places.md) with what each needs.

Nothing in the core, the scanner, the cleaner or the engine changes: the module is two small tables, a pure owner and
decision, and texts, in the shape of `dev-caches` and `local-ai-models`. **The walk does not change** (the interest is
the same shaped `Caches/*` that `app-leftovers` has), so no scan bench is due; no hot path of the UI changes, so no
performance run either ([`../../testing.md`](../../testing.md), "Performance"). One dev-tool sentence changes: the
sandbox checklist names the case file's module in a negative (02/01).

Branch: `feat/030-installed-app-caches`, stacked on plan 029's head, squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md) (`EmptyFolder`),
[009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md), [015](../015-module-groups/readme.md)
and [019](../019-uninstalled-app-leftovers/readme.md) (`Interest::only_bundle_ids`); not on plan 029's code.

## Plans

| #   | Plan                                     | Status      | Depends on | What                                                                                  |
| --- | ---------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------- |
| 01  | [The module](./01-module/readme.md)      | 🔨 in progress | —          | the crate, the tables, the category and texts, the owner and the decision, fixtures, registration |
| 02  | [End to end](./02-end-to-end/readme.md)  | 🔨 in progress | 01         | the two case files and the checklist's sentence, the core e2e flows, the UI           |
| 03  | [Finish](./03-finish/readme.md)          | 🔨 in progress | all        | the docs, the runs by hand, the conclusions                                           |

### Order and parallel work

```
01/01 ──┬── 01/02 ── 01/03 ──┬── 02/01 ──┬── 03/01 ── 03/02
        │                    └── 02/02 ──┤
        └── 02/03 ───────────────────────┘
```

- **Wave 1**: 01/01 alone (the crate, its tables, texts and category doc).
- **Wave 2, in parallel**: 01/02 (`modules/app-caches/src/` only) and 02/03 (`app/ui` only, once 01/01's texts
  exist).
- **Wave 3**: 01/03 (fixtures, golden, the contract over the fixture runs).
- **Wave 4, in parallel**: 02/01 (case files, `crates/sandbox/src/expected.rs`, `app/tauri/tests/sandboxes.rs`, and the
  app's registration of the module) and 02/02 (`crates/e2e`).
- **Wave 5**: 03.

Which files each plan touches:

| Plan | Owns                                                                                                                  | Shared files it touches                                                                                                    |
| ---- | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| 01   | `modules/app-caches/`, `fixtures/homes/app-caches-*.toml`, `fixtures/golden/app-caches.json`, `docs/categories/app-caches-library-caches.md` | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md` |
| 02   | `fixtures/sandboxes/app-caches{,-app-open}.toml`, `fixtures/golden/sandbox-app-caches*.json`, `fixtures/homes/app-caches-flow.toml`, `fixtures/golden/app-caches-flow.json`, `crates/e2e/tests/{app_caches.rs,app_caches_home/}`, `app/ui/src/test/app-caches-scan.ts` | `crates/sandbox/src/expected.rs`, `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the registration and the module lists), `app/tauri/tests/sandboxes.rs`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 03   | `docs/deferred/installed-app-caches-more-places.md` (checked)                                                         | `docs/` (architecture, testing, manual-testing, categories, ideas, plans README)                                           |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/app-caches` itself at or above 90 %.
- `git diff <plan 029's head> -- crates/core crates/scanner crates/cleaner crates/engine` is empty: no core change.
- The module's contract test passes with `Catalogue::Required`: one category file with the same danger, every text and
  every declared left-out reason used by a fixture run, every removal an `EmptyFolder` of a matched path.
- Nothing but an installed app's id-named cache folder can be offered: a name that is not an id, Apple's, Fresher's
  own, a removed app's, a vendor prefix, a sibling id, a link, a file and an empty folder are never claimed (the
  fixtures' and the case files' negatives); the core e2e shows no folder offered by both `app-caches` and
  `app-leftovers`.
- `npm run sandbox:test` passes for both case files and every other case file; it fails when a case's expectation (a
  danger, a left-out's items) is changed on purpose (done once by hand per kind, written in 02/01).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- `npm run dev:sandbox -- app-caches` and `-- app-caches-app-open`: every line of `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, a dry run): every offered cache checked by hand against `~/Library/Caches`
  and the installed apps (03/02).

## Ideas and deferred items this touches

- [Caches of installed apps](../../ideas/installed-app-caches.md): becomes this plan for `~/Library/Caches/<bundle id>`;
  the rest is deferred; the open questions are answered (no generic finder until a folder can be tied to its app;
  offline data of a chat or music app is a cache, at Low, with the texts saying it downloads again).
- [More places of installed apps' caches](../../deferred/installed-app-caches-more-places.md): a new deferred item,
  written with this plan.
- [Uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md), [Containers of an app's extensions](../../deferred/app-leftovers-containers.md):
  touched, not changed (the split of section 3 of the design; containers stay unread).

## Risks

- **An app keeps what does not come back in `Caches`**: Low, the texts, the Trash; 03/02 looks at the largest caches on
  the real Mac and adds an exclusion row where needed.
- **An app opened between the scan and the cleanup**: only an open file of the folder skips it at the act; apps accept
  a missing cache (macOS empties `Caches` itself when space runs low).
- **The first real module on `EmptyFolder`**: one Trash entry per top-level item; the core e2e and the UI prove the
  emptying and put back; the dry run on the real Mac cannot rehearse the act.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Only `~/Library/Caches/<bundle id>`; web-engine, browsers, launchers, Metal, containers and Apple's apps deferred.
2. Attribution by bundle id: the app's, a helper's, or an id under them; the longest naming id wins; never a prefix.
3. The folder is emptied, not moved (`EmptyFolder`).
4. Low, not Safe: a re-download may follow, per app unknown.
5. Kept while the app or a helper runs; open files are the cleaner's check at the act.
6. One finding per folder, not per app (no dependency on plan 029's several-path removal).
7. No age threshold, no size floor, no setting; an empty folder is not claimed.
8. Group System; module `app-caches`, category `app-caches.library-caches`.
9. The installed apps unreadable: nothing claimed, nothing said (`app-leftovers` reports it).
10. A one-row `places.toml` and an `exclude.toml` of its own (`com.apple.`, Fresher's own id).
11. The sandbox checklist names the case file's module in a negative's sentence.
12. The core e2e of `app-leftovers` is unchanged; the split is proven in `app_caches.rs`.
13. No new dependency, no new UI component.

## Open questions

None.

## Conclusions

Built on `feat/030-installed-app-caches` over plan 029's head (2b632478), **awaiting the user's walk**: the plan is not
marked done until the user has looked at the result (the list is
[`03-finish/02-run-and-conclusions.md`](./03-finish/02-run-and-conclusions.md)). The branch is stacked on
`feat/029-ollama-models` and is squash-merged after it.

**What was built.** The module `app-caches` offers `~/Library/Caches/<bundle id>` folders that an installed app owns:
the app's own id, a helper's, or an id under them (the longest naming id wins; a vendor prefix or a sibling id names
nothing). One finding per folder, Low, titled by its app (`Editor`, or `Player: com.example.Player.ShipIt`), removed as
`EmptyFolder`: the folder stays and its contents go to the Trash. Never offered: Apple's, Fresher's own, a removed
app's (that stays `app-leftovers`' to offer when proven and old), a link, a file, an empty folder, a folder read in
part, a name that is not an id. Left out with a reason: a socket inside (special), an entry not the user's, the
running apps unreadable, the app or a helper open. Installed apps unreadable: nothing is claimed and nothing said. No
age, no size floor, no setting. Around it: three fixture homes and a golden, two case files, seven core e2e flows,
the UI scenarios and baselines, and the docs as built. No core change (`git diff 2b632478 HEAD -- crates/core
crates/scanner crates/cleaner crates/engine` is empty), no new dependency and no new UI component.

**Numbers on the last code commit (d26b4b3d), on the reference Mac.**

| Check                                 | Result                                                                                 |
| ------------------------------------- | -------------------------------------------------------------------------------------- |
| `npm run pre-push`                    | green: lint, guards, knip, machete, deny, script tests (78), tests with coverage       |
| Rust tests and coverage               | 3,825 passed (5 skipped); 97.54 % of lines (floor 90 %); plan 029: 3,729               |
| `modules/app-caches` alone            | 85 tests; 99.79 % of lines (479 of 480; floor 90 %)                                    |
| `fresher-e2e --test app_caches`       | 8 tests                                                                                |
| UI unit tests and coverage            | 1,160 passed in 131 files; 98.6 % of lines (floor 90 %); plan 029: 1,157               |
| Playwright, chromium and WebKit       | 529 passed, 1 failed in one run: the watched chromium baseline                         |
|                                       | `category-details-app-leftovers-chosen` (a known flake; passed alone, `git status` clean) |
| No core change                        | `git diff 2b632478 HEAD -- crates/core crates/scanner crates/cleaner crates/engine` empty |
| Perf                                  | not run: a module only, no walk, scanner or UI component changed                       |
| What the module offered on the Mac    | pending: the dry run is the user's                                                     |

Playwright was run in the two browser projects (`--project chromium --project webkit`); `pre-push` does not run it.

**What changed from the design and why.**

1. **The registration moved** from step 01/03 to 02/01 (preflight F1): the app registration and the three
   expected-id lists came with the case files, so `fresher-app` stayed green between steps.
2. **The case files use unused names** (preflight F2: `OtherTool`, `com.apple.Maps`, `com.example.Lost`, `Scribe`,
   `Reader`), so `npm run sandbox -- all` leaves out nothing it did not already leave out (five files of other plans).
3. **"Emptied. The folder itself stays." shows in the cleanup detail row**, not in the result line (preflight F3): the
   result line counts moved items; the text shows on an item with no trashed entries.
4. **`decide` has two lifetimes** (the apps, the running set) and asks the running apps once, only when a candidate
   reaches row 9 (preflight F4). The order and restore properties live in `src/tests.rs`, since `decide` takes one
   candidate and no context; the row-9 property is in `decide/tests.rs`.
5. **A restored scan judges the open apps again**: a cache restored at launch while its app is open is left out
   `app-open`; the design's "cached matches are offered at launch" holds while the app is closed. The final review
   judged this right, and the design text says so.
6. **The open Mailer is an installed app that is running** (`[[case.app]]` with `running = true`), not a separate
   running entry.
7. **The generated checklist sentence names the module** ("never offered by `app-caches`"), so the line of the
   `app-leftovers` walk, which says the installed app's caches are offered by the other module, reads true.
8. **The core's generic enable switch** is used; the module has no setting of its own.
9. **The after-cleanup overview is outcome-based**: an emptied folder that skipped an entry is still removed from the
   results; the mock follows the engine.
10. **Reviews and fix rounds.** Tasks 1, 2, 4, 5 approved at once; task 3 needed a round (its contract test over the
    fixture runs was missing from the commit; added with the texts-used, reasons-given and removals-matched tests);
    task 6 needed a round (`reportOf` counted an emptied outcome into both the moved and the deleted sizes, now once
    by method; an unanchored row regex; scenario names). The whole branch was reviewed once at the end: nine minor
    findings, all fixed in the first commit of step 03/02. Not changed on purpose: `ExcludeError::Format` is reused for
    a bad id or prefix shape; a tie between two owners goes to the smaller app path (the title only).

**By-hand mutation checks of 02/01** (each reverted): a positive's danger changed from Low to Safe, and the socket
case's left-out items changed from 1 to 2, each failed the autotest as it should.

**The accepted risk (F7).** A running app's helper that is neither on the installed helper list nor named by the
folder is caught only by the cleaner's open-file check at the time of the cleanup. The risk is narrower than first
stated: `RunningSet::owner_of` keeps a folder whenever a running app's id names any component of its path, so a helper
whose id names the folder is caught by the module; the cleaner skips an entry held open and moves nothing of it
(proven in the e2e).

**The calls taken while building (for the user to look at).** One finding per folder; Low for all; the app's installed
helper list decides the owner and the running check; an installed app that cannot be listed claims nothing; no size
floor (to verify on a real Mac). The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to
check)".

**The user's walk.** Pending. The sandbox boxes, the dry run on the real Mac and its facts "to verify" are unchecked in
[`03-finish/02-run-and-conclusions.md`](./03-finish/02-run-and-conclusions.md). The facts: the largest ten caches
offered, looked into for what does not come back (an offline library, a draft, a download that needs a paid account),
and an `exclude.toml` row where there is one; how many small findings (under 1 MB) a real Mac shows and whether a size
floor is needed after all (agent's call 7); an app with an always-running helper, whether its cache is never offered
and whether the text tells the user what to do; an updater's cache holding a downloaded update, offered at Low and the
update downloading again. The tables, texts and category doc are corrected where the real Mac differs. This plan
becomes ✔️ done when it passes.
