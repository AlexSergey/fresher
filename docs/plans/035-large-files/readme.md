# 035 — The largest files

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, scanner, engine, testing, sandbox, e2e, app, app/ui, docs

> While High is withheld (plan [042](../042-plain-interface/readme.md)), Largest leaves the sidebar and the core
> refuses its pick; the code and its tests stay for when High comes back.

Written as a draft on 2026-11-22; designed and approved the same day by the user's delegation (build autonomously, the
recommended decisions and the agent's calls taken). The design, with its facts and agent's calls:
[`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind."

A handful of forgotten files often holds most of the space, and a list of them is expected of every tool in the
field. Since plan [027](../027-scan-screen/readme.md) removed the view by place, the app shows nothing outside its
categories; the core's place queries wait without a UI
([deferred](../../deferred/place-queries-without-ui.md)).

## Goal

A sidebar entry **Largest** answers "what are my biggest files and folders" from the scan already made, in two views
of one tree: **Files** (the home's files and packages no module claims, largest first, with kind, place, size and
"changed", filtered by kind, place and "not changed for", above a minimum size that is a setting) and **Folders** (the
place view brought back, with a breadcrumb and drill-down). The user's own files are never called junk, never
pre-selected, never shown on Scan: a ticked row becomes a finding of the core's category **Your own files**
(`core.own-file`, High) and goes through the normal Review, confirmation, Trash and put back. A row that cannot be
chosen says why (a module explains it, a standard folder, `Library`, the Trash, hidden, an app, not seen, protected,
read in part, ignored). What the scan could not see is said with the coverage notices. Archives beside a folder of the
same name are pointed out.

**Core changes, as the draft said** (core, scanner, engine): one trait method `DiskState::items` (the scanner's one pass
over the committed tree; `FixedDiskState` follows), the query and answer types, the kinds table, the own-file category,
one setting and one refusal in the core, and the engine's queries and pick. **The walk, the cache and the cleaner do
not change**: no scan bench is due; the new query is measured by hand on the `LARGE` tree (budget 50 ms, 5 MB).

**Not in this plan**: "last opened" and "the app that made it" (the facts checked on this Mac show neither is
reliable without a new port), a treemap, synced-folder markers, places outside the home. They are
[deferred](../../deferred/largest-files-more.md) with what each needs.

Branch: `feat/035-large-files`, from `main` at 33d51a6c (plans 033 and 034 merged). Plans 037, 031 and 032 merge into
`main` before this plan is built, in that order; the branch is rebased onto them first (see "Shared files"). Depends on
plans [004](../004-fast-scan/readme.md), [006](../006-permissions/readme.md), [008](../008-safe-removal/readme.md),
[012](../012-storage-map/readme.md) and [027](../027-scan-screen/readme.md).

## Plans

| #   | Plan                                       | Status      | Depends on | What                                                                                     |
| --- | ------------------------------------------ | ----------- | ---------- | ---------------------------------------------------------------------------------------- |
| 01  | [The query](./01-query/readme.md)          | 🔨 in progress | —          | the core's types, kinds, texts and setting; the scanner's items query; the engine's queries |
| 02  | [The selection](./02-selection/readme.md)  | 🔨 in progress | 01         | a picked row as a `core.own-file` finding through the existing plan and cleaner; the commands |
| 03  | [The screen](./03-screen/readme.md)        | 🔨 in progress | 02         | the Largest screen: Files, filters, Folders, the selection                                |
| 04  | [End to end](./04-end-to-end/readme.md)    | 🔨 in progress | 02, 03     | the core e2e and budget, the sandbox case file and `[case.largest]`, the UI e2e           |
| 05  | [Finish](./05-finish/readme.md)            | 🔨 in progress | all        | the docs, the deferred items, the conclusions, the user's review list                     |

### Order and parallel work

```
01/01 ── 01/02 ── 01/03 ── 02/01 ──┬── 03/01 ── 03/02 ── 04/03 ──┬── 05/01
                                   ├── 04/01 ────────────────────┤
                                   └── 04/02 ────────────────────┘
```

- In order: 01/01, 01/02, 01/03, 02/01 (each needs the one before).
- Then in parallel: 03/01 → 03/02 → 04/03 (`app/ui` only), 04/01 (`crates/e2e` only) and 04/02 (`crates/sandbox`,
  `app/tauri/tests`, `fixtures`). 04/01 and 04/02 both add lines to `docs/testing.md`: the second to finish rebases.
- Last: 05/01.

### Shared files

Files other plans in flight also touch; each change here is an addition in its own place, so the rebase onto 037, 031
and 032 is mechanical. `Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand:
regenerate them after the rebase and run the checks.

| Plan | Owns                                                                                                          | Shared files it touches                                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/largest/`, `crates/scanner/src/state/items.rs`, `crates/engine/src/largest/`                 | `crates/core/src/{lib.rs,disk_state.rs,error.rs,texts.rs}`, `crates/core/src/settings/core_schema.rs`, `crates/core/locales/en.json`, `crates/scanner/src/state/mod.rs`, `crates/testing/src/fixed_disk_state.rs`, `crates/engine/src/{lib.rs,results.rs}`, `crates/engine/src/disk_tests.rs` (its `SlowState`) |
| 02   | `crates/engine/src/own.rs`, `app/tauri/src/largest_commands.rs`, `docs/categories/core-own-file.md`           | `crates/engine/src/{results.rs,cleanup/pending.rs,cleanup/operation.rs}`, `app/tauri/build.rs`, `app/tauri/src/lib.rs` (the handler list), `app/tauri/capabilities/default.json`, `app/tauri/src/storage_commands.rs`, `app/ui/src/ipc/bindings.generated.ts`, `docs/restrictions/deletion-safety.md`, `docs/categories/README.md` |
| 03   | `app/ui/src/screens/largest/`, `app/ui/src/product/own-item-row/`, `app/ui/src/test/largest-scan.ts`         | `app/ui/src/ipc/index.ts`, `app/ui/src/state/navigation.ts`, `app/ui/src/screens/{frame-sidebar.tsx,screen-stack.tsx,app-frame.tsx}`, `app/ui/src/locales/en.json`, `app/ui/src/gallery/registry.ts`, `app/ui/src/design/icons/icons.ts` |
| 04   | `crates/e2e/tests/{largest.rs,largest_budget.rs,largest_home/}`, `fixtures/sandboxes/large-files.toml`, `fixtures/golden/sandbox-large-files.json`, `app/tauri/tests/sandboxes/largest.rs` | `crates/sandbox/src/{case.rs,expected.rs,lib.rs}`, `app/tauri/tests/{sandboxes.rs,sandboxes/harness.rs}`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 05   | `docs/deferred/largest-files-more.md`, `docs/categories/core-own-file.md` (checked)                           | `docs/` (architecture, testing, manual-testing, categories, ideas, deferred, plans README), `docs/deferred/user-review-2026-11-22.md` |

Where the lists are ordered (the commands in `build.rs`, the handler and the capability), the three new commands take
the place of `get_storage_place`, which this plan removes, so they never land on the lines other plans append to. The
sandbox report gets a `largest` section only for a case file with a `[case.largest]`, so no other golden changes.
Plan 036 (Uninstall, planned in parallel) adds a sidebar entry too: the second to merge adds its `NavItem` beside the
other's.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- `git diff <base> -- crates/cleaner crates/scanner/src/walk crates/scanner/src/codec crates/scanner/src/rescan` is
  empty: the walk, the cache and the cleaner do not change.
- The Files list holds no node a module claims, nothing under `~/Library` or `~/.Trash`, no hidden name, no app
  bundle, nothing inside a package, nothing below the minimum (the core e2e and the sandbox's negatives).
- A picked row is a `core.own-file` finding at High; it is never in the overview's totals, a category card or the
  map's explained part; it goes to the Trash through the normal confirmation and comes back by put back (the core
  e2e and the sandbox).
- `npm run sandbox:test` passes with `large-files.toml` and every other case file, and fails when a
  `[case.largest]` expectation is changed on purpose (done once by hand per kind, written in 04/02).
- `npm run test:e2e` passes with the Largest scenarios.
- The budget test prints under 50 ms and 5 MB for the items of the `LARGE` tree (run by hand, written in 05/01).
- `npm run dev:sandbox -- large-files`: every line of `EXPECTED.md` holds by hand (the user's walk).

## Ideas and deferred items this touches

- This plan replaces the idea "Large Files Finder" (2026-11-15, the user's); everything it held is here or deferred.
  The "big download not opened" rule of plan [020](../020-downloads/readme.md) stays a category of its own.
  [Duplicate Finder](../../ideas/duplicate-finder.md) would sit on the same rows.
- [The core's place queries without a UI](../../deferred/place-queries-without-ui.md): closed by this plan (the view
  returns as Folders; `get_storage_place` removed), its file and row removed in 05/01.
- [A folder's age from its direct children](../../deferred/folder-age-from-children.md): touched (a folder's "Changed"
  is its own time), not changed.
- [The largest files, what waits](../../deferred/largest-files-more.md): new, written in 05/01.

## Risks

See [`00-design.md`](./00-design.md), "Risks": the user removing what they need (High, never pre-selected, the Trash),
a sync client's folder in the home, a package in use (the cleaner's open-file check), a folder's own "Changed" date,
the folders' sizes after a cleanup, the rebase.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken. This design
  and its calls are approved by it; the user checks them later
  ([`../../deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md)).
- No version bump: 0.1.0.

## Agent's calls (to check)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Two views, Files and Folders.
2. A picked row is a finding of the core's category `core.own-file` (High), kept apart from the scan's findings.
3. No "last opened": "Changed" instead (Spotlight's date is missing for nearly every file; the access time is only the
   first read after a change).
4. No "the app that made it".
5. Ranked by allocated size: a cloud file and a sparse file are not listed.
6. The sandbox uses small data-filled files and a 10 MB minimum, not sparse files.
7. A picked row's removability is the cleaner rehearsal's to judge.
8. No synced-folder marker.
9. Hidden names are never offered.
10. The minimum is a choice of five sizes, 100 MB by default.
11. `get_storage_place` is removed from the commands.
12. A claimed place is skipped even when its module or category is ignored.
13. At most 10 000 items per state.

## Conclusions

Built on `feat/035-large-files` over plan 038's head (050f2878), **awaiting the user's walk**: the plan is not marked
done until the user has looked at the result ([`05-finish/01-docs-and-conclusions.md`](./05-finish/01-docs-and-conclusions.md),
`../../manual-testing.md` items 1.20 and 2.18).

**What was built.** A new screen, Largest, in the sidebar between Scan and Review. Files lists the biggest things below
the home that no module explains, ranked by allocated size, with filters for kind, place and "Changed"; Folders shows a
folder's children with a breadcrumb back. One judgement in the engine (`judge_reached`) decides, for every row, offered
or withheld with a reason; the same judgement answers at the pick, at prepare and at confirm. A row the user ticks
becomes a finding of the core's category `core.own-file` (High, never pre-selected, Trash by default) held apart from
the scan's findings, so Review, the cleaner, the journal and put back run unchanged. The setting `core.largest.min-size`
(10, 50, 100, 500 MB, 1 GB; 100 MB by default) sets the minimum.

**What changed from the design.**

- Withheld by the judgement: Library, the Trash, hidden names, app bundles, package interiors, claimed module folders
  (ignored or not), protected and standard folders (the protection list and the cleaner's re-check), folders holding a
  claimed node. A folder holding an ignored path is withheld but opens (call 12, reviewed twice).
- `TextKey` accepts a number-hyphen-word segment (`10-mb`, `1-gb`).
- `get_storage_place` is removed (the deferred item of the place queries is closed); the commands `get_largest`,
  `get_largest_place` and `pick_own_item` stand in its place.
- A pre-existing sandbox bug is fixed: a stated access was compared with paths without expanding `~`, so a denied
  folder was still scanned; the `downloads-denied` golden changed (one remainder reason).
- The items cap is `MAX_ITEMS + 1` so truncation is known; the confirmation has no separate High tick yet (plan 039).

**Numbers on the last code commit, on the reference Mac.**

| Check                                              | Result                                                                    |
| -------------------------------------------------- | ------------------------------------------------------------------------- |
| `npm run pre-push`                                 | green: lint, guards, knip, machete, deny, script tests, tests with coverage |
| Rust tests and coverage                            | 4 624 passed (6 skipped); 97.51 % of lines (floor 90 %)                   |
| UI tests and coverage                              | 1 339 passed in 147 files; 98.21 % of lines                               |
| `npm run sandbox:test`                             | 97 passed (every case file, `large_files` included)                       |
| `fresher-e2e --test largest`                       | 17 flows (8 negative, 9 positive)                                         |
| Items budget (release, `LargestBench`)             | 8.7 ms and 3.25 MB for a full list of 10 000 items (under 50 ms and 5 MB); 3 us at the default minimum on `LARGE` |
| Playwright, chromium and webkit                    | 699 passed, 8 failed under load and 7 of them passed on re-run (the 8th is the known flaky chromium baseline); gallery axe 4 passed on re-run |
| `git diff 050f2878 -- crates/cleaner crates/scanner/src/{walk,codec,rescan}` | empty                                           |

By hand, each reverted (step 04/02): a wrong kind, a wrong reason, a Trash file expected listed and a Documents folder
expected granted each failed the sandbox run with a message naming the case.

**Calls taken while building.** A pick is judged again at prepare and at confirm; `gone` outlives a cancelled scan; the
home itself is never pickable; a collapsed subtree is one item; texts of a pick are core helpers.

**Facts to verify** (the user's review list, section 035): Spotlight's last-used date on another Mac, a large
hidden-free folder an app keeps in the home, the item count on a real home, the query's time on a real home.
