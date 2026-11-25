# 042 — A plain interface

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-25 ·
**Modules:** core, engine, modules, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, docs

Written as a one-file draft on 2026-11-24 and turned into plans and steps the same day from a read of the code on
`main` (b12959dc). Approved by the user's delegation of 2026-11-24: build autonomously, take the recommended decisions
and the agent's calls, write down what the user must look at. The design, with its agent's calls:
[`00-design.md`](./00-design.md). Goes before 041 (the audit runs on the interface that ships). Branch
`feat/042-plain-interface`, squash-merged into `main` once, when plan 09 is done.

## Intent

The user's task (translated, from the manual test of 2026-11-24,
[`../../manual-testing/2026-11-24-feedback.md`](../../manual-testing/2026-11-24-feedback.md), items 1-27): "The UI
looks bad and I do not understand what is happening on the screens: it must be simplified. A more laconic,
understandable interface, with short clear descriptions and a clear separation of sections. A balance between how much
data a screen shows and how beautiful it is. For the first release, hide High entirely. Settings is overloaded: a
sidebar with its structure and a global search."

The design was found on mockups (three rounds; the user approved the third):
[`docs/mockups/0.1.0/`](../../mockups/README.md). The screens are built from them.

## Goals

1. Scan answers "what can I remove, and how much" at a glance: a compact disk card with a coloured ring, a separator,
   then tiles grouped by what removing them costs.
2. Review shows what goes and what the user has checked, category by category, items apart.
3. Every text a person reads is short and plain: no "Swap and sleep image", no "Comes back by itself when needed".
4. Settings has its own sidebar (a tree of sections) and a search that jumps to a setting; numbers are sliders.
5. No High finding is offered in the first release, and the core enforces it.

## Non-goals

- New categories or modules; changing a category's level (only its texts, its icon and its one line).
- A light theme, translations, onboarding (014).
- Removing High from the code: the categories, their tests and the High consent of 039 stay, switched off.

## Plans

| #   | Plan                                                  | Status      | Depends on | What                                                                                          |
| --- | ----------------------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------------------- |
| 01  | [The core](./01-core/readme.md)                       | 🔨 awaiting the user's walk | —          | the High switch and its refusals, the app's switch, the five parts, icons and the line, the group texts, the scan's estimate |
| 02  | [Design pieces](./02-design-pieces/readme.md)         | 🔨 awaiting the user's walk | 01         | tokens, the button's sheen and the menu's hover, separator, tile, icon badge, disk card, swoosh, slider |
| 03  | [Scan](./03-scan/readme.md)                           | 🔨 awaiting the user's walk | 02         | the screen, the scanning card, what has no meaning without High                               |
| 04  | [Review](./04-review/readme.md)                       | 🔨 awaiting the user's walk | 03         | the tick model, the screen                                                                    |
| 05  | [Category details and menus](./05-details/readme.md)  | 🔨 awaiting the user's walk | 04         | the details screen, the menus                                                                 |
| 06  | [Settings](./06-settings/readme.md)                   | 🔨 awaiting the user's walk | 02         | its sidebar and pages, the search, the sliders                                                |
| 07  | [The logo](./07-logo/readme.md)                       | 🔨 awaiting the user's walk | 02         | the sidebar's logo, the app icons and the social preview                                      |
| 08  | [The wording](./08-wording/readme.md)                 | 🔨 awaiting the user's walk | 01-07      | the core and module texts, the interface texts, the guide and the check                      |
| 09  | [Finish](./09-finish/readme.md)                       | 🔨 awaiting the user's walk | all        | the docs, the baselines, the demo, the conclusions, the user's review list                    |

### Order

```
01/01 ─ 01/02 ─ 01/03 ─ 01/04 ─ 01/05 ─ 01/06 ─ 02/01 ─ 02/02 ─ 02/03 ─ 02/04 ─ 02/05
  ─ 03/01 ─ 03/02 ─ 03/03 ─ 04/01 ─ 04/02 ─ 05/01 ─ 05/02 ─ 06/01 ─ 06/02 ─ 06/03 ─ 07/01 ─ 07/02
  ─ 08/01 ─ 08/02 ─ 08/03 ─ 09/01
```

Every step is one commit and leaves the workspace green (`cargo nextest`, `npm run test:ui`, clippy, tsc, and the e2e
specs it touches). The steps run one at a time: plan 01 shares the core's dictionary, `texts.rs` and the bindings;
plans 02 to 07 share the tokens, the locales, the e2e mock and the baselines; plan 08 reads every dictionary.

Which files each plan touches:

| Plan | Owns | Shared files it touches |
| --- | --- | --- |
| 01 | `crates/engine/src/high_tests.rs`, `crates/core/src/storage/summary.rs`, `crates/e2e/tests/high_withheld.rs`, `fixtures/sandboxes/high-withheld.toml` | `crates/core/src/{danger.rs,error.rs,app_info.rs,category.rs,event.rs,texts.rs,storage/map.rs,lib.rs}`, `crates/core/locales/en.json`, `crates/engine/src/{engine.rs,results.rs,own.rs,registry.rs,session.rs,storage/shares.rs,cleanup/pending.rs,apps/page.rs,apps/uninstall.rs,testing.rs}`, `crates/testing/src/contract.rs`, `crates/sandbox/src/{case.rs,expected.rs}`, `crates/e2e/tests/`, `app/tauri/src/{compose.rs,sandbox.rs,commands.rs,testing.rs}`, `app/tauri/tests/sandboxes/harness.rs`, every module's `src/lib.rs` (the icon) and `locales/en.json` (the line), `app/ui/src/ipc/bindings.generated.ts` (generated), the UI's mocks and fixtures, `docs/restrictions/deletion-safety.md`, `docs/architecture/modules.md` |
| 02 | `app/ui/src/design/{layout/section-separator,layout/tile,data/swoosh-progress,primitives/slider}/`, `app/ui/src/product/{category-icon,disk-card}/` | `app/ui/src/design/tokens/`, `design/primitives/{button,menu}/`, `design/data/storage-ring/`, `design/icons/icons.ts`, every baseline (02/01), the gallery |
| 03 | `app/ui/src/product/{category-tile,scanning-card}/`, `app/ui/src/screens/scan/scan-groups.tsx` | `screens/scan/`, `screens/frame-sidebar.tsx`, `state/{navigation,reduce}.ts`, `product/selection-bar/`, `e2e/core-mock.ts`, the e2e specs, `app/ui/src/locales/en.json` |
| 04 | `app/ui/src/screens/review/review-tile.tsx` | `state/{selection,selection-summary,reduce}.ts`, `screens/review/`, `product/{danger-zone,finding-row,finding-list,selection-bar}/`, `crates/core/locales/en.json`, `crates/core/src/texts.rs`, `fixtures/demos/scenarios/cleanup.toml`, the e2e specs |
| 05 | — | `screens/category/`, `product/{finding-row,category-tile,left-out-lines}/`, `design/primitives/menu/`, the locales, the e2e specs |
| 06 | `app/ui/src/screens/settings/{settings-sidebar,settings-search,settings-page}.tsx` | `screens/settings/`, `product/settings-row/`, `state/navigation.ts`, the locales, the e2e specs |
| 07 | `app/ui/src/design/brand/` | `screens/frame-sidebar.tsx`, `app/tauri/icons/`, `assets/social-preview.png`, `docs/dependencies.md` |
| 08 | — | every `locales/en.json`, `docs/categories/`, `docs/writing.md`, `scripts/check-texts.ts`, `scripts/check-texts.test.ts`, `fixtures/sandboxes/trial.toml` and the trial module, the UI's fixtures, the baselines |
| 09 | `docs/deferred/user-review-2026-11-24.md` | `docs/` (architecture, restrictions, testing, manual-testing, writing, categories, plans, deferred, ideas) |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them and run the
checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit, `check:texts` included; coverage stays at or above 90 % of
  lines in Rust and the UI.
- With `HighLevel::Withheld` no High finding is in a page, a total, the found size, the map's "Can be freed", a
  module's categories or choices, or a left-out; `prepare_cleanup` with a High item, `pick_own_item` and an uninstall
  of everything are refused with `HighWithheld` and act on nothing (`high_tests.rs`, `high_withheld.rs`,
  `high-withheld.toml`). The app always withholds.
- A "Look before removing" item goes only when the user ticked it: a Look tile ticked on Scan sends nothing (unit tests
  of the reducer, the e2e flows).
- Scan, Review, the details, the menus and Settings match the mockups of `docs/mockups/0.1.0/` (sizes, colours, texts),
  built with the design tokens; every changed baseline was looked at before it was committed.
- `git diff main -- crates/scanner crates/macos crates/system crates/store crates/rules crates/cleaner` is empty;
  `git diff main -- modules` touches only each module's `src/lib.rs` (the icon), `locales/en.json`, and its tests and
  fixtures where a text or a name changed.
- The walk of 09/01 by hand holds in the sandboxes it names.

## Ideas and plans this touches

- Plan [039](../039-agreement-and-wording/readme.md): its Review parts, "danger zone" and Scan question are replaced;
  its agreement, High consent and text check stay.
- Plan [027](../027-scan-screen/readme.md): the scan screen is rebuilt; its cards become tiles.
- Plans [035](../035-large-files/readme.md) and [036](../036-uninstall/readme.md): Largest and "Remove everything" are
  hidden while High is withheld; their code stays.
- Plan [038](../038-demo-recordings/readme.md): the `cleanup` scenario's Review button changes.
- Plan [041](../041-security-audit/readme.md): runs after this plan.

## Risks

The design's [risks](./00-design.md#risks): a path that still offers High, screenshot churn, the tick model, texts
that change twice.

## Agent's calls (summary)

The full list is in [`00-design.md`](./00-design.md), "Agent's calls (to check)": the switch set by the app (1), the
group names (2), the five parts merged in the core (3), Look items never ticked for the user (4), Largest hidden (5),
before 041 (6), High back in a later release (7), a fixed set of icon keys (8), the wording pass last (9), a sixth
text `line` (10), the remainder counts as the user's (11), withheld hides left-outs and module info (12), the core's
progress estimate (13), Docker described as now (14), details' made-by and scrolling list (15), menu descriptions as
hints (16, reversed: no hints), Settings' own data and switch order (17), the mock offers High by default (18), "2 minutes ago" (19), the
counts of the bar (20).

## Open questions

None. The draft's question 1 (when High comes back) and question 2 (icons per category) are answered by agent's calls
7 and 8, dated 2026-11-24.

## Decisions

- 2026-11-24: direction A (a summary and a list) chosen; after two rounds the third mockup approved: the disk card
  with a ring, a separator, tiles grouped by level (Scan), a summary card and per-category tiles with items (Review);
  Settings with its own sidebar and search, sliders, the details screen, the menu hover (the user).
- 2026-11-24: High hidden for the first release, by a switch in the core, all five High places go, Largest included
  (the user).
- 2026-11-24: the sidebar of the app stays (the user: "neat and extensible").
- 2026-11-24: logo variant A (graphite leaf, iris swoosh); the swoosh on the buttons and as the scan's progress; the
  logo small in the sidebar beside the name; no decorative swoosh at the bottom of Scan (the user).
- 2026-11-24: approved mockups live in the repository, `docs/mockups/<version>/` (the user).
- 2026-11-24 — the user delegated: build autonomously, the recommended decisions and agent's calls taken; what the
  user must look at goes to `docs/deferred/user-review-2026-11-24.md` (written by 09/01).

## Conclusions

**The user's look** (2026-11-25): the sandbox walk is done and the interface approved ("on the whole I like
everything"); two menu changes followed (no hints, no "Stop looking for" item). The dry run on the real Mac is part
of the release preparation.

**What was built** (the commits of `feat/042-plain-interface`):

- **The core** (01): the High switch, `HighLevel::{Offered, Withheld}`, a required argument of the engine, set to
  `Withheld` by the app in every build; under it no finding at High is in a page, a total, the found size, "Can be
  freed", a module's categories or choices, a left-out, an insight part, an app page's items or an Apps row's parts,
  and every prepare with a High item, a Largest pick and an uninstall of everything is refused with `HighWithheld`;
  `AppInfo.high` and `AppInfo.home`; the disk's five parts (`VolumeMap.summary`); 14 icon keys and a sixth text, the
  line; the group names and lines; the scan's estimate (`ScanProgress.expected`).
- **The interface** (02 to 07): tokens for every mockup colour, the button's sheen, the menu's hover; the separator,
  the tile, the icon badge, the disk card, the swoosh, the slider; Scan with the disk card, the scanning card, the
  groups and tiles; Review with the tick model (`Selection.toCheck`), the summary card and a tile per category; the
  details with three answers and one menu everywhere; Settings with its sidebar, pages, search and sliders; Largest
  hidden while High is withheld; the logo in the sidebar, the app icons and the social preview.
- **The wording** (08): 315 texts changed (core 42, interface 44, the modules 229: app-caches 17, app-leftovers 14,
  demo 22, dev-caches 40, docker 1, downloads 17, jetbrains 22, local-ai-models 19, logs 9, stale-projects 13, temp
  6, trial 22, unused-toolchains 13, xcode 14); the trial's items renamed; `writing.md` gained the group names, the
  five parts, the line and a word list; the text check gained the line's 50 characters, its full stop and the name's
  four words.
- **The finish** (09): the Apps row's High parts left out, the docs as built, the walk (manual testing 1.24), every
  baseline taken again, the user's review list
  ([`user-review-2026-11-24.md`](../../deferred/user-review-2026-11-24.md)).

**The numbers.** The last `npm run pre-push`: 5020 Rust tests (6 skipped), 97.58 % of lines; 1573 UI tests,
98.02 % of lines; 207 tests of the scripts; `check:texts` 0 violations. Playwright: 890 passed, 1 skipped (gallery,
WebKit and Chromium). The text check under today's rules: 15 violations on the dictionaries before the pass
(jetbrains 4, interface 4, stale-projects 2, downloads 2, core 2, logs 1), 0 after. Baselines: 542 now; against
`main`, 55 added, 220 changed and 49 deleted, all taken again from scratch at the end.

**What changed from the design, and why.**

- High is hidden further than the design's five places: the published `FindingsAdded`, insight parts at High (the
  Docker card loses its volumes line), an app page's items and an Apps row's parts of a High kind; the user asked to
  hide High entirely.
- The detailed parts of the map stay in the core for its queries; the UI draws only the summary.
- `AppInfo.home` was added for Review's `~` short path; the full path stays in the tooltip.
- Review's rows gained the details' menu; Review's tile has an all-checkbox, the only sender of `tick-all`; a category
  whose level the overview does not tell counts as Look.
- No "modules failed" line on the scanning card: a failed module makes the scan partial and its notice says so.
- The number field still sends a typed number at once; only the slider sends on release.
- The lint-staged hook no longer formats `bindings.generated.ts`.

**Calls taken while building** (each is on the user's list): the selection bar's level badge removed; the group's
Select all description in a tooltip; the side inset 32 px; the bar's region "Ready to remove"; picked tiles "N
picked"; the ignored tabs "Sections", "Not looked for", "Items"; "Yes, move to the Trash"; the dimmed search tree a
little brighter than the mockup; the sidebar mark's small-size drawing; the app icon without a rounded tile; the trial
tool's command kept; the demo's names kept.

**Status:** built, awaiting the user's walk (09/01). It is not done until the user has looked.
