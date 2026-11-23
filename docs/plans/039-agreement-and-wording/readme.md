# 039 — The agreement, the danger zone and the wording

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 ·
**Modules:** core, engine, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, repo, docs

Written as a one-file draft on 2026-11-22 and turned into plans and steps the same day from a read of the code on
`main` (33d51a6c). Approved by the user's delegation of 2026-11-22: build autonomously, take the recommended decisions
and the agent's calls, write down what the user must look at. The design, with its agent's calls:
[`00-design.md`](./00-design.md). Refreshed on 2026-11-23 against `main` 740cf3c3 (plans 031 to 038 merged): every
step names the code as it stands there; the design's agent's calls 14 to 18 record what the refresh found.

## Intent

The user's task (translated): "Think whether we should add some agreements, for example to protect ourselves if the
user deletes something by accident or because of a bug in the app. Perhaps mark in the review area that it is a danger
zone. And improve the hint texts across the whole app. Make this a plan too."

The app deletes a real person's files. Before the first release three things must be true: the person has been told
plainly whose decision a removal is and what the app does not promise; the last screen before a removal looks as
serious as it is; and every sentence in the app helps them decide.

## Goals

1. Before the first cleanup the user has read and accepted a short agreement, written in plain English, and the engine
   refuses a cleanup without it.
2. Review shows risky items apart, as a danger zone, and a High item cannot be removed in passing: the confirmation
   asks for its own tick, and the core refuses a High plan without it.
3. When something does go wrong, the app says how to get it back and how to report it.
4. Every choice, level, reason and result reads in one voice, and a check over the dictionaries keeps it so.

## Non-goals

- Fear: no alarm colours on safe things, no "your Mac is at risk", no countdown ([best practices](../../ideas/best-practices.md), 24).
- An agreement that replaces safety: the confirmation, the Trash and the journal stay exactly as they are.
- Accounts, a licence server, sending the acceptance anywhere: it is a local setting.
- Translations: this plan makes the English final; other languages are [an idea](../../ideas/ui-translations.md).
- The README's "No warranty" section and the release page: plan [040](../040-readme/readme.md) and plan
  [013](../013-release/readme.md) carry the agreement's words; this plan writes them once, in the core's dictionary
  and in [`deletion-safety.md`](../../restrictions/deletion-safety.md).

## Two parts, built at two times

The plan is built in two parts on one branch (`feat/039-agreement-and-wording`), squash-merged into `main` once, when
part B is done ([`../../workflow.md`](../../workflow.md)). On 2026-11-23 every plan both parts wait for is on `main`,
so part B follows part A at once (a rebase only if `main` moved):

- **Part A — the agreement, the danger zone, the help** (plans 01 to 03): built right after plans 035, 036 and 037 are
  on `main` (the branch is rebased onto it first). It changes the core's confirmation and the engine's preconditions,
  so it goes in before the last modules, which then inherit it.
- **Part B — the wording** (plans 04 and 05): built last before the security audit (plan 041), after every other plan
  of the session (031 to 038) is on `main`, over the texts as they then stand. Its steps are procedures, not lists:
  they read whatever dictionaries, modules and screens exist at that time.

## Plans

| #   | Plan                                          | Part | Status      | Depends on | What                                                                                |
| --- | --------------------------------------------- | ---- | ----------- | ---------- | ----------------------------------------------------------------------------------- |
| 01  | [The agreement](./01-agreement/readme.md)     | A    | 🔨 awaiting the user's walk | 036        | the text and its version, the engine's refusal, the commands, the sheet, About      |
| 02  | [The danger zone](./02-danger-zone/readme.md) | A    | 🔨 awaiting the user's walk | 01         | the core's High consent, Review's two parts, the confirmation's High tick, Scan     |
| 03  | [Help](./03-help/readme.md)                   | A    | 🔨 awaiting the user's walk | 02         | "Removed something you need?", the issue template                                   |
| 04  | [The wording](./04-wording/readme.md)         | B    | 🔨 awaiting the user's walk | 03, all    | `docs/writing.md`, the text check, the pass over every dictionary, the hints        |
| 05  | [Finish](./05-finish/readme.md)               | B    | 🔨 awaiting the user's walk | all        | the docs, the walk, the conclusions, the user's review list                         |

### Order

```
part A:  01/01 ── 01/02 ── 01/03 ── 02/01 ── 02/02 ── 03/01
part B:  (rebase onto main) ── 04/01 ── 04/02 ── 04/03 ── 05/01
```

Every step is one commit and leaves the workspace green (`cargo nextest`, `npm run test`, clippy, tsc). The steps run
one at a time: 01/01, 01/02 and 02/01 share the core's confirmation and the engine's test helpers; 01/03, 02/02 and
03/01 share `app-frame.tsx`, the mocks and the e2e specs; 04/02 and 04/03 share the screenshot baselines.

Which files each plan touches:

| Plan | Owns                                                                                                   | Shared files it touches                                                                                                                                    |
| ---- | ------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/agreement.rs`, `crates/e2e/tests/agreement.rs`, `app/ui/src/product/agreement-sheet/`, `app/ui/src/product/licence-dialog/`, `app/ui/e2e/support/agreement-mock.ts` | `crates/core/src/{lib.rs,settings.rs,settings/core_schema.rs,command.rs,error.rs,texts.rs,cleanup/outcome.rs}`, `crates/core/locales/en.json`, `crates/engine/src/{engine.rs,settings.rs,cleanup/blocked.rs,cleanup/pending.rs,cleanup/operation.rs,testing.rs}`, `app/tauri/{build.rs,capabilities/default.json}`, `app/tauri/src/{commands.rs,lib.rs,testing.rs,dev_runs/commands.rs,sandbox/tests.rs}`, `app/tauri/tests/{bindings.rs,sandboxes/harness.rs,sandboxes/uninstall.rs}`, `crates/sandbox/src/expected.rs`, `crates/e2e/tests/{support/home_engine.rs,cleaning/mod.rs,places.rs,jetbrains_home/mod.rs}`, `fixtures/demos/scenarios/cleanup.toml`, `app/ui/src/{ipc,state,screens/app-frame.tsx,screens/settings,screens/apps/app-screen.tsx}`, the mocks, the e2e specs, `app/ui/src/locales/en.json` |
| 02   | `app/ui/src/screens/review/review-part.tsx`                                                            | `crates/core/src/{cleanup/confirm.rs,error.rs}`, `crates/core/tests/compile_fail/confirmation_literal.*`, the echo helpers of every crate (02/01 lists them, `crates/cleaner`'s two test files and `crates/testing/src/cleanup_fakes.rs` among them), `app/ui/src/{product/confirm-dialog,product/danger-zone,screens/review,screens/scan,screens/apps/app-screen.tsx,state/cleanup.ts,state/cleanup-actions.ts}`, `app/ui/e2e/support/cleanup-mock.ts`, the e2e specs, both dictionaries |
| 03   | `app/ui/src/product/recovery-help/`, `.github/ISSUE_TEMPLATE/`                                         | `app/ui/src/product/result-summary/`, `app/ui/src/screens/cleanups/`, the e2e specs, `app/ui/src/locales/en.json`                                           |
| 04   | `docs/writing.md`, `scripts/check-texts.ts`, `scripts/check-texts.test.ts`                             | every `locales/en.json`, `docs/categories/`, `fixtures/demos/scenarios/`, UI components that gain a hint, `package.json`, `docs/quality.md`, the screenshot baselines |
| 05   | —                                                                                                      | `docs/` (architecture, testing, manual-testing, restrictions, ideas, plans README, deferred)                                                               |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a rebase
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit, `check:texts` included; coverage stays at or above 90 % of
  lines in Rust and the UI.
- A cleanup cannot start before the agreement is accepted: `prepare_cleanup` and `confirm_cleanup` answer
  `CleaningBlocked { reason: Agreement }` (the engine's tests, the core e2e, and the sandbox autotest for every case
  file); put back is never refused for it.
- A plan whose highest level is High cannot be confirmed without `highRiskUnderstood: true` (the core's tests and
  the core e2e); the confirmation's button stays inactive until its tick is set, and the tick is never remembered.
- Review shows "Safe to remove" and "Danger zone" apart; nothing in the danger zone is pre-selected; Scan's "Select
  all" of a Medium or High zone asks first.
- `npm run check:texts` reports nothing on every dictionary; `docs/writing.md` states every rule the check enforces.
- `npm run test:e2e` passes; every changed screenshot baseline was looked at before it was committed.
- The walk of 05/01 by hand holds in a sandbox with one item of each level.
- No change to `crates/scanner`, `crates/macos`, `crates/system`, `crates/store`, `crates/rules`; `crates/cleaner`
  changes only in its two test echoes (`src/test_support.rs`, `tests/never.rs`, design call 15); modules change only
  in their `locales/en.json` (part B).

## Ideas and plans this touches

- The "trust contract" of [best practices](../../ideas/best-practices.md);
  [Cleanup Preview & Safe Removal](../../ideas/cleanup-preview-and-safe-removal.md) (nothing pre-selected above Safe).
- Plan [014](../014-onboarding/readme.md): when it is built, the agreement becomes the last of its first-run screens
  (the sheet and the engine's guard stay).
- Plans [040](../040-readme/readme.md) and [013](../013-release/readme.md): the "No warranty" words.
- Plan [036](../036-uninstall/readme.md): its uninstall plan goes through the same agreement gate and the same High
  tick ("everything" is High). Plan [035](../035-large-files/readme.md): a picked file is High and takes the tick.
  Plan [038](../038-demo-recordings/readme.md): the `cleanup` scenario accepts the agreement as a user would.

## Risks

- **The agreement is not legal advice**: it is the agent's best effort; a lawyer's look is the user's question (see
  "Open questions").
- **A missed prepare path**: a cleanup path that does not call `blocked::check` would skip the agreement. 01/01 lists
  every caller and the core e2e proves each.
- **Plans merged before part A** (031 to 038) added cleanup flows and demo scenarios that meet the sheet and the High
  tick: the steps name them (the `cleanup` scenario accepts in 01/03; the High flows tick in 02/02). Part B runs the
  full e2e, `npm run demo:check` where the app can start, and the scenario trace test after its text changes.
- **The text pass changes hundreds of strings**: screenshot churn, and a text that changes meaning by accident. The
  pass never changes a key, a placeholder or a danger level, and each step lists what it changed.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. The agreement comes before the first cleanup, not the first launch, and is accepted once per version of its text.
2. Five plain lines, with the licence one click away; no scroll-to-accept.
3. The engine enforces it, at prepare and at confirm; put back never needs it.
4. Medium and High form the danger zone; only High needs the extra tick, every time, and the core enforces it.
5. A writing guide and an automatic check, so the texts stay good after this pass.
6. Calm wording stays: the zone is marked by structure and one colour, not by alarm.
7. The acceptance lives in the settings file as two core settings of a group the screen never shows, kept by a reset.
8. The full licence texts are bundled into the UI at build time and open in a dialog: no browser, no network.
9. The sandbox autotest asserts the agreement guard for every case file, instead of one case of `cleanup-guards`.
10. "A changed text asks again" is proven by the core's tests and the UI's unit tests, not by a UI e2e scenario.
11. The address for reporting a wrong finding is shown as selectable text; the app opens nothing.

## Open questions

None that blocks the build. The draft's question 1 (a lawyer's look at the agreement and the licences before the first
public release) is the user's, recorded in [the user's review](../../deferred/user-review-2026-11-22.md).

## Decisions

- **2026-11-22** — the user delegated: build autonomously, the recommended decisions and agent's calls taken; what the
  user must look at goes to [`../../deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md).
- **2026-11-22** — the draft's open question 1 (a lawyer's look) is not settled by the agent: the plan is built
  without it, the text is the agent's best effort and says nothing a lawyer would have to promise; the question goes
  to the user's review list (agent's call 2 of the design records the wording choice).

## Conclusions

**What was built** (the commits of `feat/039-agreement-and-wording`):

- **The agreement** (01): five lines in `core.agreement.*`, `AGREEMENT_VERSION = 1`, accepted once in two hidden
  settings; the engine refuses every prepare and confirm without it (`BlockedReason::Agreement`); commands
  `get_agreement` and `accept_agreement`; the sandbox harness asserts the guard for every case file; the sheet before
  the first cleanup, the licence dialog and Settings, About.
- **The danger zone** (02): `highRiskUnderstood` checked by the core; Review's safe part and danger zone; the High tick;
  Scan's question before a Select all of a Medium or High zone. Nothing is pre-selected.
- **Help** (03): "Removed something you need?" and the wrong-finding issue form.
- **The wording** (04): `docs/writing.md` and `scripts/check-texts.ts` (in pre-push); nine texts changed in the core
  and the modules, none in the interface; six kinds of hint.

**The numbers.** The last `npm run pre-push`: 4956 Rust tests (6 skipped), 97.57 % of lines; 1480 UI tests, 98.13 %
of lines. The check: 22 violations at its first run (core 2, app-leftovers 2, jetbrains 3, interface 15), 0 after.
Texts changed per dictionary: core 2, app-leftovers 2, jetbrains 5, the other modules 0, the interface 0. Hints added:
the level badge, the found size in the ring, the selection total, the selection bar's level, the three Largest
filters and the Apps filter.

**What changed from the design, and why.**

- The "what changed" key is `core.agreement.changes.v<n>`: a text key may not have a digits-only segment.
- `crates/cleaner`'s two test files gained the new field (call 15 of the refresh): they deserialize a `Confirmation`.
- A demo accepts by clicking "I understand" (call 16): the GIF shows the sheet.
- `accept_agreement` is refused on a settings file that was not read (the final review): a broken file was replaced.
- The text check's rules skip the interface where it labels a screen (parts, descriptions) and nine headings are
  exempt, each with its reason; two rule tweaks (a placeholder is not "trash").
- The hints are anchors and described controls, not a new component; `Select` and `SegmentedControl` gained a
  `description` prop. The sidebar's Review count, the ring's legend and the storage map's figures have none.
- A case file's `[settings]` can pre-accept the agreement in a hand-built sandbox: [deferred](../../deferred/sandbox-case-settings-agreement.md).

**Calls taken while building.** The jetbrains names read "... of older JetBrains IDEs"; the Trash line is reworded
("until the Trash is emptied", "what an app's own tool removes"); texts whose meaning looked wrong were listed, not
rewritten (constraint of part B); the docker case file offers nothing to remove, so the walk does not use it.
Every one of these is on the user's list.

**Status:** built, awaiting the user's walk (05/01). It is not done until the user has looked.
