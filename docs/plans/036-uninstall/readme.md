# 036 — Uninstall an app

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, system, engine, testing, sandbox, e2e, app, app/ui, docs

> While High is withheld (plan [042](../042-plain-interface/readme.md)), an app's page offers only "Keep my settings"
> and lists no file of a High kind, the Apps list leaves those parts out, and an uninstall of everything is refused.

Written as a one-file draft on 2026-11-22 and approved the same day by the user's delegation (Decisions, below). The
design, with the facts checked on the developer's Mac and every agent's call: [`00-design.md`](./00-design.md).

## Goal

The user picks an installed app on a new **Apps** screen and sees everything that belongs to it, with sizes, before
anything happens: the bundle and the files Fresher ties to it by bundle id, item by item, with the last time it was
used when that is known. **Uninstall…** offers two choices, "Remove the app, keep its settings and data" (the default)
and "Remove the app and everything it left"; either is **one item** of the normal confirmation, moved to the Trash
together and put back together. Apple's apps are not listed; an app a package manager installed, one that needs an
administrator (App Store apps among them), and an open app are shown with the reason and never offered. Files another
installed app may share are kept. The apps folder opens only whole bundles, and only for an uninstall plan.

Not a module: an engine view over the last scan, like the storage map (agent's call 1). The cleaner does not change;
the core, the engine, the system adapter, one row of the macOS protection list and one pane address do. No module
changes.

Branch: `feat/036-uninstall`, from `main` at 33d51a6c; plans 037, 031, 032 and 035 merge into `main` first, and the
branch is rebased on them before it is built (the shared files: design, section 10). Squash-merged into `main` once
when done ([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md),
[009](../009-system-knowledge/readme.md), [012](../012-storage-map/readme.md) (`AppPlaces`),
[019](../019-uninstalled-app-leftovers/readme.md), [027](../027-scan-screen/readme.md),
[029](../029-ollama-models/readme.md) (`MoveTogether`) and [030](../030-installed-app-caches/readme.md).

## Plans

| #   | Plan                                        | Status      | Depends on | What                                                                                     |
| --- | ------------------------------------------- | ----------- | ---------- | ---------------------------------------------------------------------------------------- |
| 01  | [Footprint](./01-footprint/readme.md)       | 🔨 awaiting the walk | —          | the core's types and texts, the apps folder's protection, the bundle's facts, the list   |
| 02  | [Removal](./02-removal/readme.md)           | 🔨 awaiting the walk | 01         | the uninstall plan of the two choices, the paths that went, App Management               |
| 03  | [Screen](./03-screen/readme.md)             | 🔨 awaiting the walk | 01, 02     | the Apps list, the app's page, the choices and the confirmation, the UI's e2e            |
| 04  | [End to end](./04-end-to-end/readme.md)     | 🔨 awaiting the walk | 01, 02     | the sandbox's case files and goldens, the core e2e                                       |
| 05  | [Finish](./05-finish/readme.md)             | 🔨 awaiting the walk | all        | the docs, the runs by hand, the conclusions, the user's review list                      |

### Order and parallel work

```
01/01 ──┬── 01/02 ──┬── 01/04 ── 02/01 ──┬── 02/02 ──┬── 03/02 ──┬── 05/01
        └── 01/03 ──┘                    ├── 03/01 ──┘           │
                                         ├── 04/01 ──────────────┤
                                         └── 04/02 ──────────────┘
```

- **Wave 1**: 01/01 (core types, texts, categories, the trait method).
- **Wave 2, in parallel**: 01/02 (`crates/core/src/cleanup/protection.rs`, `crates/macos/src/protection.rs`) and 01/03
  (`crates/system/src/bundle_facts/`, `crates/testing`).
- **Wave 3**: 01/04 (the engine's list and page, the commands).
- **Wave 4**: 02/01 (the uninstall plan, the context, the paths that went).
- **Wave 5, in parallel**: 02/02 (App Management: core, macos pane, system, engine), 03/01 (`app/ui` list), 04/01
  (sandbox) and 04/02 (`crates/e2e`). They share no file but `crates/core/locales/en.json` (02/02 only adds keys; 03/01,
  04/01 and 04/02 do not touch it) and the regenerated `bindings.generated.ts` (02/02 and 03/01: 03/01 starts from
  02/02's file when both run, else regenerates it after).
- **Wave 6**: 03/02 (the page and the flow; needs 02/02's notice).
- **Wave 7**: 05/01.

| Plan | Owns                                                                                                    | Shared files it touches                                                                         |
| ---- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/apps/`, `crates/system/src/bundle_facts/`, `crates/engine/src/apps/` (list), `app/tauri/src/apps_commands.rs` | `crates/core/src/{lib,error}.rs`, `crates/core/src/system/apps.rs`, `crates/core/src/cleanup/protection.rs`, `crates/core/locales/en.json`, `crates/macos/src/protection.rs`, `crates/system/src/{apps,lib}.rs`, `crates/testing/src/{system_fakes,stated_apps}.rs`, `crates/engine/src/{engine,lib,results}.rs`, `crates/engine/src/storage/attribution.rs`, `app/tauri/src/{lib,compose,sandbox}.rs`, `app/ui/src/ipc/bindings.generated.ts` |
| 02   | `crates/engine/src/apps/{uninstall,management}.rs`                                                     | `crates/engine/src/cleanup/{context,operation,pending}.rs`, `crates/engine/src/{results,access,engine}.rs`, `crates/core/src/access.rs`, `crates/core/locales/en.json`, `crates/macos/src/privacy/panes.rs`, `crates/system/src/access.rs` |
| 03   | `app/ui/src/screens/apps/`, `app/ui/src/product/app-row/`, `app/ui/src/product/footprint-list/`         | `app/ui/src/{ipc/index.ts,ipc/mock.ts,state/navigation.ts,state/cleanup.ts,screens/frame-sidebar.tsx,screens/screen-stack.tsx,design/icons/icons.ts,locales/en.json}`, `app/ui/e2e/`, `app/ui/src/gallery/` |
| 04   | `fixtures/sandboxes/uninstall{,-open}.toml`, `fixtures/golden/sandbox-uninstall*.json`, `app/tauri/tests/sandboxes/uninstall.rs`, `crates/e2e/tests/{uninstall.rs,uninstall_home/}` | `crates/sandbox/src/{case,expected,world,generate}.rs`, `crates/testing/src/app_bundle.rs`, `app/tauri/tests/sandboxes.rs`, `app/tauri/tests/sandboxes/harness.rs`, `app/tauri/src/sandbox.rs`, `crates/e2e/Cargo.toml` |
| 05   | `docs/deferred/uninstall-system-files.md`, `docs/categories/core-uninstall-*.md`                        | `docs/` (architecture, restrictions, categories, testing, manual-testing, ideas, plans README, deferred) |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
or a rebase and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `crates/engine/src/apps/` and `crates/system/src/bundle_facts/` at or above 90 % each.
- `git diff <base> -- crates/cleaner modules` is empty: the cleaner and the modules do not change.
- The protection's tests: nothing inside a bundle, the apps folder itself or a folder in it can pass; Fresher's own
  bundle never; an ordinary cleanup's item in the apps folder is skipped `Protected` (the engine's second wall).
- The core e2e: both choices move exactly their paths to the sandbox's Trash, put back brings the whole app back, and a
  part replaced, held open or grown, or the app opened after the plan, moves nothing.
- `npm run sandbox:test` passes for `uninstall` and `uninstall-open` and every other case file; it fails when an
  expectation is changed on purpose (once by hand, written in 04/01).
- `npm run test:e2e` passes with the Apps scenarios in chromium and WebKit; axe finds nothing.
- `npm run dev:sandbox -- uninstall` and `-- uninstall-open`: every line of `EXPECTED.md` holds by hand (the user's).
- On the developer's Mac (`npm run dev`, a dry run): the list checked by hand against `/Applications` (05/01, the
  user's).

## Ideas and deferred items this touches

- This plan replaces the ideas "Smart Uninstaller" and "Application Footprint" (2026-11-15, the user's).
  [App Resource Profile](../../ideas/app-resource-profile.md) would extend the Apps list; not in this plan.
- New deferred item (05/01): the system-wide files of an app, the files it keeps under its name, and package managers'
  own commands (`docs/deferred/uninstall-system-files.md`).
- Plan [010](../010-history/readme.md)'s watch mode would name an app whose footprint grows; it reads this plan's
  numbers when it is built.

## Risks

- **App Management**: whether macOS asks, refuses silently or shows its own notice on the first bundle moved is not
  known (no app may be moved to find out); the item fails whole and the page says what to do.
- **A wrong part** (a folder another app shares under the same id): kept `shared` when a second installed bundle names
  it; High at most, in the Trash, listed item by item before the choice.
- **The rebase** over 037, 031, 032 and 035: additions only; the design's section 10 lists every shared file.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. An engine view, not a module.
2. `InstalledApps::bundle` with a default answer.
3. Last used: Spotlight's date or the newest change of the app's data folders, never the executable's access time.
4. Keep-data is the default choice.
5. App Store apps are not offered (owned by root), against the draft.
6. Package-managed apps are shown with the manager's command, not removed; Homebrew casks only.
7. The footprint's places are a `const` table in the engine.
8. More than 64 paths: refused, never truncated.
9. One item of several paths: all or nothing, put back whole.
10. The apps folder opens only whole bundles, only for an uninstall plan.
11. App Management is learned, never probed.
12. No watch mode in this plan.
13. No drag and drop, no Trash watcher, no resource profile.
14. A finished uninstall keeps the Scan selection.
15. Containers are kept without other apps' data.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken (the user's
  words, translated: "work autonomously with subagents, take the recommended decisions; implement all the features;
  write into docs/deferred what I will need to look at — I will check everything tomorrow"). The draft's four calls
  stand. The draft's line "an app from the App Store is removed like any other bundle" is changed by a fact checked
  on this Mac (App Store bundles are owned by root): such an app is shown and not offered (agent's call 5).

## Conclusions

Built on `feat/036-uninstall` over main's head 7cbd6624 (plans 031, 032, 035, 037 and 038 first), **awaiting the user's
walk**: the plan is not marked done until the user has looked at the result
([`05-finish/01-docs-and-conclusions.md`](./05-finish/01-docs-and-conclusions.md),
[`../../manual-testing.md`](../../manual-testing.md) items 1.22 and 2.20, and the review list in
[`../../deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md)).

**What was built.** A sidebar entry **Apps** lists the installed apps the user may remove, with size, version and "not
used for"; an app's page lists its footprint item by item (the bundle and the files tied to it by bundle id, each with
its size and danger, the kept ones with the reason). **Uninstall...** offers "Remove the app, keep its settings and
data" (the default, Medium) and "Remove the app and everything it left" (High); either is one item of the normal
confirmation (`MoveTogether`, the bundle first), moved to the Trash together and put back together. Apple's apps are
not listed; a package manager's app, one that needs an administrator (App Store apps), a locked, a protected and an open
app are listed with the reason and never offered. Files another installed app may share are kept. The apps folder
opens only whole bundles, and only for an uninstall: the protection's first wall and the engine's second wall (both
apps folders refused for every other plan), plus a canonical-path check at prepare and at confirm. App Management is
learned from the first uninstall. The cleaner and the modules are unchanged.

**What changed from the design, and why.**

- App Store apps are not offered (owned by root), against the draft; the groups come from a read-only `id -G` because
  nothing else reads them.
- An incomplete installed list keeps every part but the bundle (a part may be another app's); a group container goes
  only with the app's own team id and when no other installed app shares it.
- The moved paths join the state's gone paths (no `AppsCache::forget`); an ordinary cleanup's cleared paths join them
  too, so Largest hides them until the next scan.
- `NeverKind` (`Closed`, `OpensBundles`, `Refused`) instead of a flag; a bundle matching a never or ignored place in
  another case is refused; `InstalledApps::is_as_listed` is a new port method; the keep-data choice also moves the
  launch agents (login items): an agent left without its app fails at every login.
- The text keys of App Management are `core.access.app-management.*` and the refusals `core.error.*`, as the core's
  schemes require; the choices are `role=radio` buttons; the UI's flows are in `e2e/apps.spec.ts`.
- Ordinary cleanups now refuse `~/Applications` too.

**The numbers.** Tests added or touched: core 512, system and testing and sandbox and app 1657 (step 01/03), engine
2319 with its neighbours (02/01), 2076 across the crates (02/02), sandboxes 100 and e2e 213 (17 new flows), vitest 1410
with 97.3 % of lines, Playwright 627 on the Apps specs in both engines. The final `npm run pre-push`:
passed on the docs-and-conclusions state (4895 Rust tests run, 4895 passed, 6 skipped, above the 90 % bar; vitest 1410 passed, 98.15 % of lines). The scan's time did not change: the scan, the scanner and the UI's hot paths are untouched, and the
measure runs only on the Apps screen, on demand, off the engine's locks, so no performance run was made
([`../../testing.md`](../../testing.md), "Performance"). `git diff 7cbd6624 -- crates/cleaner modules` is empty. The
sandbox runs `uninstall` and `uninstall-open` in `npm run sandbox -- all`; three expectations were changed on purpose
and failed naming their case (04/01).

**Calls taken while building** (the rulings, each with its cost in the ledger of the build):

- `/Applications` opens whole bundles in the protection, and its walls are lexical protection, links refused by the
  listing and the plan, a case-insensitive match against never and ignored places, and the canonical byte-for-byte
  check; the Unicode limit and the cleaner's names-as-written gap are known issues and a security-audit item.
- `id -G` is accepted; the cask links are read per listing; a partial installed list keeps every part; one measure for
  two listings, cancelled by a scan; Fresher's own id is never listed.
- App Management learned from any Trash refusal of the bundle (over-broad, unverified, deferred).
- The scan's folding of small Library files is accepted for this plan (deferred); the case files use 1 MiB parts.
- An app opened between the confirmation and the move is not re-checked (the cleaner is frozen; a known issue).

**Left for the user:** the two walks, the unverified macOS facts and the calls in the review list, the three deferred
items and the two known issues named above.
