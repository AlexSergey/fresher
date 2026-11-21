# 012 — The storage map: where the space went

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
macos, scanner, engine, testing, sandbox, app, app/ui

Expanded on 2026-11-17 from the one-file draft of 2026-11-15 and approved the same day by delegation: the user asked
the agent to take the recommended answer to every open question and record it ([`00-design.md`](./00-design.md),
"Decisions"). The agent's calls wait for the user's check.

## Goal

The overview answers "where did my space go" in one look: one model of the whole volume into which every module's
findings fit, whose parts add up to the space the Mac reports, with no "Other". Two views of the same numbers — by
meaning (apps and categories) and by place (folders) — and the space that is not files of the scan (the system's
volumes, swap and the sleep image, purgeable space) named and explained, never offered as junk. What the app could not
measure is said plainly, with the reason. The volume facts plan 009 deferred (kinds, free and purgeable space) are
built here, with their first consumer.

Branch: `feat/012-storage-map`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
Depends on plans [002](../002-base-architecture/readme.md), [003](../003-base-ui/readme.md),
[004](../004-fast-scan/readme.md) and [009](../009-system-knowledge/readme.md) (done); plan
[006](../006-permissions/readme.md)'s access status names the permission of an unreadable folder. Plan
[007](../007-safe-development/readme.md) is built in parallel; this plan's case file (05) waits for it to be merged into
`main`.

**Since plan 027 (2026-11-20)** the home volume's map is the Scan screen's ring and legend; the separate map block of
the home volume, the view by place and the rows of the view by meaning are gone from the UI. Other volumes keep their
map block. The core's queries stay; see [`../../deferred/place-queries-without-ui.md`](../../deferred/place-queries-without-ui.md).

## Plans

| #   | Plan                                                    | Status                            | Depends on   | What                                                                                       |
| --- | ------------------------------------------------------- | --------------------------------- | ------------ | ------------------------------------------------------------------------------------------ |
| 01  | [The model](./01-model/readme.md)                       | ✔️ done | —            | the map's types, owned nodes, the shares, attribution to apps, the invariants              |
| 02  | [Volume reconciliation](./02-volume/readme.md)          | ✔️ done | 01/01        | the volume facts (kind, capacity, free, purgeable, neighbours), the remainder and its reasons |
| 03  | [Places and the query](./03-places/readme.md)           | ✔️ done | 01, 02       | the by-place view with auto-descent, the map's query and its three phases                  |
| 04  | [The overview screen](./04-overview/readme.md)          | ✔️ done | 03           | the commands, the state, the bar, both views on the overview, live filling                 |
| 05  | [The sandbox case file](./05-sandbox/readme.md)         | ✔️ done | 03, plan 007 | `[case.map]` in the case format, `fixtures/sandboxes/storage-map.toml`                     |
| 06  | [Finish](./06-finish/readme.md)                         | 🔨 in progress | all          | the docs, the deferred item closed, a run by hand, the conclusions                         |

### Order and parallel work

```
01/01 ── 01/02 ── 01/03 ── 01/04 ──────────────┐
   └──── 02/01 ──┐                             ├── 02/04 ── 03/01 ── 03/02 ── 04/01 ── 04/02 ── 04/04 ── 04/05 ──┐
02/02 ───────────┴── 02/03 ────────────────────┘                                                  ▲               ├── 06
04/03 ────────────────────────────────────────────────────────────────────────────────────────────┘               │
                                                          (plan 007 merged into main) ── 05/01 ── 05/02 ──────────┘
```

- **Wave 1, in parallel**: 01/01 (core types), 02/02 (the Foundation call: `fresher-macos`'s `ffi` only) and 04/03 (the
  UI's segment tones: `app/ui/src/design/` only).
- **Wave 2, in parallel**: 01/02 (`DiskState`: core, scanner, testing) and 02/01 (the `Volumes` port: core's
  `volume.rs`, testing's `fakes.rs`); they touch different files of the core.
- **Wave 3, in parallel**: 01/03 (the engine's shares) and 02/03 (`MacVolumes::space`, `fresher-macos` only).
- **Wave 4, in order**: 01/04 and 02/04 (both add a file to `crates/engine/src/storage/` and a line to its `mod.rs`),
  then 03/01, 03/02, 04/01, 04/02, 04/04, 04/05 (the last two both edit the overview screen).
- **Plan 05 after plan 007 is merged into `main`** and `main` into this branch (decision 8); then 06.

Which files each plan touches, so this plan and plan 007 stay apart:

| Plan | Owns                                                                                                                       | Shared files it touches                                                                                                                                       |
| ---- | -------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/storage/`, `crates/engine/src/storage/{mod,shares,meaning}.rs`                                            | `crates/core/src/{lib,disk_state,finding,volume}.rs`, `crates/core/src/system/{ids,running}.rs`, `crates/core/locales/en.json`, `crates/core/tests/typescript.rs`, `crates/scanner/src/state/mod.rs`, `crates/testing/src/fixed_disk_state.rs`, `crates/engine/src/{lib,results,session}.rs`, `crates/engine/src/disk_tests.rs` |
| 02   | `crates/engine/src/storage/volume.rs`                                                                                      | `crates/core/src/{lib,volume}.rs`, `crates/core/locales/en.json`, `crates/testing/src/fakes.rs`, `crates/engine/src/storage/{mod,meaning}.rs`, `crates/macos/src/volume.rs`, `crates/macos/src/ffi/volume.rs`, `crates/macos/Cargo.toml`, root `Cargo.toml` (an `objc2-foundation` feature), `Cargo.lock`, `docs/dependencies.md` |
| 03   | `crates/engine/src/storage/{place,query}.rs`                                                                               | `crates/engine/src/{engine,lib,results}.rs`, `crates/core/src/error.rs`, `crates/core/locales/en.json`                                                        |
| 04   | `app/tauri/src/storage_commands.rs`, `app/ui/src/product/{storage-map,meaning-list,place-list}/`, `app/ui/src/state/storage.ts` | `app/tauri/src/lib.rs` (one `mod`, three handler lines), `app/tauri/build.rs` (`COMMANDS`), `app/tauri/capabilities/default.json` (two grants), `app/ui/src/ipc/{index,mock}.ts`, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/state/{refresh,store,reduce,navigation}.ts`, `app/ui/src/design/data/{segment.ts,chart-stories.ts,usage-bar/,storage-ring/}`, `app/ui/src/product/scan-hero/`, `app/ui/src/screens/overview/`, `app/ui/src/locales/en.json`, `app/ui/src/gallery/` |
| 05   | `fixtures/sandboxes/storage-map.toml`, `fixtures/golden/sandbox-storage-map.json`                                          | `crates/sandbox/src/{case,expected}.rs`, `app/tauri/tests/sandboxes.rs`, `docs/testing.md` ("Sandboxes")                                                       |
| 06   | —                                                                                                                          | `docs/` (architecture, deferred, ideas, plans README, testing)                                                                                                |

**Overlap with plan 007**: 007 owns `app/tauri/src/{compose,commands}.rs`, `app/tauri/src/sandbox.rs`,
`app/ui/src/screens/app-frame*`, `crates/cleaner/src/reach.rs` and `crates/core/src/cleanup/protection.rs`; this plan
touches none of them. Both may touch `app/tauri/src/lib.rs` (007: its setup; 012: one `mod` line and three handler lines), and
`app/tauri/build.rs` with `capabilities/default.json` if 007 adds a command (each side appends its own lines),
`app/ui/src/ipc/bindings.generated.ts` (never merged by hand: regenerate it after a merge) and `docs/testing.md`
(different sections). `crates/sandbox/` and `app/tauri/tests/sandboxes.rs` are 007's; plan 05 edits them only after
007 is in `main`. `Cargo.lock` and `package-lock.json` are never merged by hand: on a conflict take either side,
regenerate them and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- The property tests hold: in both views the leaves add up to the volume's capacity; every owned byte is counted
  once, nested claims and ignored findings included.
- On the developer's Mac (`npm run dev`, the default root `~`): the bar's parts add up to the disk's capacity as
  `df -k` reports it, within the remainder; "macOS" and "Swap and sleep image" match the used space `df` gives their
  volumes; "Outside the scanned folders" names its reasons; Free matches `df`'s available space; the purgeable note is
  at least zero and is shown only for a local volume.
- During a scan the bar fills as findings arrive, and "Explained" plus "Not scanned yet" always equals the volume's
  used space; at the end the full map replaces it in one move.
- By place: opening the home descends through a folder that holds 90 % of its parent; each row says how much of it is
  explained; switching views keeps the total.
- No part of the map other than "Explained" offers a cleanup; there is no "Other" anywhere.
- `npm run sandbox:test -- storage_map` passes, and fails when a `[case.map]` expectation is changed on purpose.
- `npm run dev:sandbox -- storage-map`: `EXPECTED.md`'s map lines hold by hand.

## Ideas this touches

- The [storage explainer](../../ideas/storage-explainer.md) and the
  [system data analyzer](../../ideas/system-data-analyzer.md): the parts and the reconciliation line become this plan;
  the analyzer's snapshot listing and thinning stay an idea (decision 5).
- The [application footprint](../../ideas/application-footprint.md): the per-app line of the by-meaning view; the
  last-used date stays an idea.
- [The Trash holds the freed space](../../ideas/trash-holds-freed-space.md): the Trash is a named place of your
  folders; the journal's share stays an idea.
- The [large files finder](../../ideas/large-files-finder.md): the by-place view is the list its chart would sit on;
  the chart stays an idea.
- [macOS specifics](../../ideas/architecture/macos.md): purgeable space, local snapshots, the system's volumes.

## Conclusions

Built on `feat/012-storage-map` on 2026-11-17, task by task, each commit through the hooks and each task reviewed;
a whole-branch review of plans 01 to 04 and 06/01 led to one fix wave and a re-review. Plan 05 (the sandbox case file) was built after plan 007 reached `main`.

- **Merged into `main` on 2026-11-17 at the user's request, before the run by hand** (06/02): the plan stays 🔨 in
  progress until the user has run its checklist; the results of 06/02 are written then, on `main`.
- **What is there now**: the map's model and the shares; the volume facts (`getattrlist` used space, neighbours, the
  parts and the remainder with its reasons); the place view and the three queries; the three commands, the state, the
  bar, the view by meaning (cards, apps, places) and the view by place. Closed: plan 009's volume-facts deferred item.
- **Changed from the design while building** (each in its step's conclusions): per-volume used space through
  `getattrlist` (02/03); `explained(state, id)` and the corrected shares rules (01/03); the meaning scoped to the roots
  of one volume (03/02); the committed volumes kept beside the committed state (03/02); places refused while a scan
  runs (03/02); `Descent` `through` or `stay` and the `NotScanned` texts (04/05); cards for every counted category with
  "more under apps" (final review); only the home volume gets the by-meaning list; the largest folders are only
  folders; `disk_changed` only when the map is complete (final review).
- **Measured**: the generated LARGE tree, release: `Shares::of` 3.9 ms and 625 226 heap bytes (budget 50 ms and 5 MB).
  `npm run pre-push` passes on the last code commit `b1288ba`: Rust 1731 passed, 5 skipped, lines at or above 90 %;
  UI 926 tests, lines 97.83 %. `npm run test:e2e` 223 passed. No version bump (0.1.0).
- **Left**: 06/02, the run by hand on the real Mac and
  in the sandbox (with the `Shares::of` measurement on the developer's home), waits for the user. The deferred items
  roots without an explained line and a way up from the meaning view (both folded into
  [the place queries without a UI](../../deferred/place-queries-without-ui.md) by plan 027). Parked minors of the final re-review: the
  one-render crumbs in the view by place have no test, and the UI sums sizes for "more under apps" (the core does not
  give that sum).
- **For the user to check**: the design's agent's calls, and the list below.

**Agent's calls taken during the build (to check):**

- Plan 05 waited for plan 007 in `main` (decision 8). Costs, if wrong: nothing built is lost; the case file is written later.
- The volumes are read through `CleanupPorts`, so `compose.rs`, which plan 007 owns, stays untouched. Costs, if wrong: a
  later move of the read into the composition, one file.
- The step's seven small readings of the types (private `VolumeMap` fields, strict part order, no `part.remainder`
  keys, `Trash` keys with `.name` and `.description`, the tags of the serialized types, `BundleId` serialized, texts
  without placeholders) were accepted. Costs, if wrong: renaming keys and tags in the core, the bindings and the UI.
- `FixedDiskState::with_owner` is reused instead of a new `owned_by`. Costs, if wrong: a rename across two files.
- An ignored inner owner is subtracted from the outer owner and its bytes go to your folders (the step's "counted"
  wording was wrong; the claims' rule says the same). Costs, if wrong: the category totals differ by the ignored bytes.
- `explained(state, id)`, strictly inside a counted owner, is the node's whole size, and the call is made for shown rows
  only. Costs, if wrong: an API that takes a path and a state; the cost of a call is the depth times the length.
- The used space of a volume comes from `getattrlist` `ATTR_VOL_SPACEUSED`, one call per neighbour (`statfs` gives the
  container's). Costs, if wrong: the parts of an APFS volume could not add up.
- The mount-point coverage is computed inside `volume_map` from the root paths. Costs, if wrong: one more input to pass.
- The developer-home measurement of `Shares::of` moved to 06/02, by hand: it needs the real data folder, which no test
  may touch. Costs, if wrong: the budget on a real home is unproven until the run.
- `useStoragePlace` takes the volume index, so a stale node reopens that volume's first root. Costs, if wrong: one
  argument less, and an ambiguous reopen with two volumes.
- The cards are the core's category groups (core sizes); the totals give only count and danger; with a card for every
  counted category and "more under apps" when part of it sits under apps. Costs, if wrong: a card size that differs
  from its selection, or a category wholly under apps without a card.
- The by-meaning list is for the home volume only. Costs, if wrong: other volumes show only their bar.
- An ignored finding that names an app stays in that app's data (design 00, decision 7 amended). Costs, if wrong: the
  app's size differs by the ignored bytes.
- The views refetch on the content of the volume's map and the phase, not on the phase alone. Costs, if wrong: stale
  views after an ignore, or one request more per map change during a scan (the place may flicker, if it is reachable).
- The volumes are read before `begin()` and stored with it; `disk_changed` only when the map is complete; the largest
  folders are only folders. Costs, if wrong: a note shown late, or a large file absent from the list.
- Places are refused while a scan runs, so a place never pairs the old tree with the new findings. Costs, if wrong: the
  by-place view is empty during a rescan.
- The sandbox harness sorts the report's lists by path (and drops the fake Trash's `<n>-` prefix), because the cleanup
  order follows the findings' ids. Costs, if wrong: none; a report that depends on ids would flicker.
- The goldens keep allocated sizes, with a guard for the APFS 4 KiB assumption that fails with a message. Costs, if
  wrong: the goldens need regenerating on another file system.
- Cases moved out of the places that prompt, and the nest is checked at the outer cache. Costs, if wrong: a case that
  does not reach the place it names; the map's rows for those places are unchecked.
