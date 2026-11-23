# 039 — The agreement, the danger zone and the wording: design

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 ·
**Modules:** core, engine, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, repo, docs

The design the plans argue from. Written from the draft of 2026-11-22 and a read of the code on `main` (33d51a6c);
refreshed on 2026-11-23 against `main` 740cf3c3, where plans 031 to 038 are merged. Each step still reads its code
again before it changes it.

## Intent

The user's task (translated): "Think whether we should add some agreements, for example to protect ourselves if the
user deletes something by accident or because of a bug in the app. Perhaps mark in the review area that it is a danger
zone. And improve the hint texts across the whole app. Make this a plan too."

## Goals

1. Before the first cleanup the user has read and accepted a short agreement; the engine refuses a cleanup without it.
2. Review shows "Safe to remove" and a "Danger zone" apart; a High plan needs its own tick, which the core checks.
3. The result screen and Cleanups say how to get a removed item back and how to report a wrong finding.
4. One voice for every text, written down in `docs/writing.md` and kept by `npm run check:texts` in pre-push.

## Non-goals

- Alarm: no red on Safe or Low, no "your Mac is at risk", no countdown, no scroll-to-accept.
- Any change to what is removed, how, or when: the confirmation, the Trash, the journal, the cleaner stay as they are.
- Sending or syncing the acceptance; accounts; a licence server.
- Translations ([idea](../../ideas/ui-translations.md)); the README and the release page (plans 040 and 013).
- The bug-report file ([deferred](../../deferred/bug-report-file.md)); the licence notices of dependencies
  ([deferred](../../deferred/licence-notices.md)).

## What already exists (read on 33d51a6c)

| What                                   | Where                                                                                       | Plan |
| -------------------------------------- | ------------------------------------------------------------------------------------------- | ---- |
| The confirmation echo and its refusals | `crates/core/src/cleanup/confirm.rs` (`Confirmation`, `ConfirmError`, `PermanentConsent`)    | 008  |
| The cleanup's preconditions            | `crates/engine/src/cleanup/blocked.rs` (`check`, at prepare and at confirm), `BlockedReason` in `crates/core/src/cleanup/outcome.rs` (`Settings`, `Journal`, `DiskFull`, `OpenFiles`; texts `core.cleanup.blocked.<id>`) | 008, 011 |
| The settings and a reset that keeps some keys | `crates/core/src/settings/core_schema.rs`, `crates/engine/src/settings.rs` (`IGNORE_LISTS` are kept by `reset`), `SettingsView::new` in `crates/core/src/command.rs` | 005, 015 |
| Refusals with the core's text          | `CommandError` and `Refusal` in `crates/core/src/error.rs`; texts `core.error.<kebab-variant>` | 003  |
| Review in zones by level               | `app/ui/src/screens/review/` (`reviewZones`), `app/ui/src/product/danger-zone/` (`DangerZone`, `ZONE_ORDER`); texts `core.danger.<level>.{name,zone,meaning}` | 027  |
| The confirmation dialog                | `app/ui/src/product/confirm-dialog/confirm-dialog.tsx`: highest level, groups, left-out, the permanent tick reset at each opening, Cancel first focus | 008  |
| Scan's "Select all" per zone           | `app/ui/src/screens/scan/scan-zones.tsx`, `zones.ts` (`zoneSelectAll`); nothing pre-selected (`state/defaults.ts`, `PRESELECTED`) | 027  |
| Settings → About                       | `settings-screen.tsx`: the version and `ui.screen.settings.about.licence`                    | 005  |
| Design pieces                          | `design/layout/{sheet,dialog}`, `design/primitives/{tooltip,checkbox}`                      | 003  |
| The module contract over texts         | `crates/testing/src/contract.rs`: five category texts, setting label and description, choices, declared problems | 002  |
| The licences                           | `LICENSE-MIT` (21 lines), `LICENSE-APACHE` (177 lines) at the repository root                | 001  |
| Every plan's tail and its guard        | `rehearse_and_keep` in `crates/engine/src/cleanup/pending.rs` calls `blocked::check`; `prepare` and `apps::uninstall::prepare` both end in it; `operation::confirm` calls `blocked::check` again | 036  |
| The uninstall                          | `crates/engine/src/apps/uninstall.rs`; categories `core.uninstall-keep-data` (Medium) and `core.uninstall-everything` (High) in `crates/core/src/apps/categories.rs`; confirmed by `confirm_cleanup`, a permanent echo refused (`PlanMismatch`); UI: `prepareUninstallPlan` in `state/cleanup-actions.ts`, the same `ConfirmDialog` on `screens/apps/app-screen.tsx` (no Review) | 036  |
| The largest files                      | `core.own-file`, always High (`crates/core/src/largest/own_file.rs`); a row is made a finding by `Engine::pick_own_item` (`crates/engine/src/own.rs`), joins the selection, and goes through Review, `prepare_cleanup` and `confirm_cleanup` (judged again by `own::check_picks`) | 035  |
| The Trash                              | `crates/engine/src/cleanup/trash.rs`: reads and opens a Trash, never empties one; no cleanup path of its own | 037  |
| The demo recordings                    | `fixtures/demos/scenarios/{overview,cleanup}.toml` (`cleanup` moves "Xcode build data", Safe, to the Trash), every target traced to a text key by `crates/sandbox/src/scenario/traced.rs`; the player `app/ui/src/demo/`; `npm run demo:check`; each run claims a fresh world folder, so its data folder holds no acceptance; no GIF is committed yet | 038  |
| The command lists                      | `app/tauri/build.rs` (`COMMANDS`), `app/tauri/capabilities/default.json` (one grant each), `app/tauri/src/dev_runs/commands.rs` (`ACTIONS` / `READS`, a test holds every command in one) | 001, 029 |

The dictionaries held **918 texts** on 33d51a6c and **1164 texts** in 16 dictionaries on 740cf3c3: the core's 335,
the interface's 353 and fourteen `modules/*/locales/en.json` (xcode 35 and docker 36 among them). A census on
740cf3c3: no `!` anywhere; every non-`ui` `.label` has its `.description` except the four `docker.part.*.label`
(a part is described by its `detail` and `effect`, plan 032); the longest `name` is 52 characters, the longest
`if-removed` 238; 18 texts use "delete", two of them (`app-leftovers.choice.*.description`) not about a permanent
delete.

## Design

### 1. The agreement (plan 01)

**The text** lives in the core's dictionary, versioned by the core:

| Key                                  | Text                                                                                                   |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| `core.agreement.title`               | Before your first cleanup                                                                              |
| `core.agreement.intro`               | Five things to know before Fresher removes anything:                                                   |
| `core.agreement.line.no-warranty`    | Fresher is free and open source, and it comes with no warranty.                                        |
| `core.agreement.line.your-decision`  | It removes only what you confirm, so each removal is your decision.                                    |
| `core.agreement.line.best-judgement` | A danger level is Fresher's best judgement, not a guarantee.                                           |
| `core.agreement.line.backup`         | Keep a backup of anything you cannot afford to lose.                                                   |
| `core.agreement.line.trash`          | Items go to the Trash, where you can put them back until the Trash is emptied. A permanent delete, and what an app's own tool removes, cannot be undone. |

`crates/core/src/agreement.rs`: `pub const AGREEMENT_VERSION: u32 = 1;` the line ids in order
(`AGREEMENT_LINES: [&str; 5]`); `Acceptance { version: u32, at: Timestamp }`; `AgreementView { version, title,
intro, lines: Vec<Text>, accepted: Option<Acceptance>, changes: Option<Text> }` (camelCase, `TS`), built by
`AgreementView::new(accepted)` over the current version and, for tests, `AgreementView::of(current, accepted)`.
`changes` is `core.agreement.changes.v<current>` when an older version was accepted; a core test holds that every
version from 2 to the current one has its `changes` text. `is_accepted()` is `accepted.version == current`. Changing a
line means raising `AGREEMENT_VERSION` and writing its `changes` text, in the same change
([`deletion-safety.md`](../../restrictions/deletion-safety.md), "The agreement").

**The state** is two core settings in the settings file, of a new `SettingGroup::Agreement`:
`core.agreement.version` (`Number { min: 0, max: 1000 }`, default 0 = never accepted) and `core.agreement.accepted-at`
(`Number { min: 0, max: i64::MAX }`, seconds since the epoch, default 0). `SettingsView` leaves the group out (the
screen never shows it); the generic `update_setting` refuses its keys (`UnknownSetting`); `reset` keeps them, as it
keeps the ignore lists. Only `Engine::accept_agreement(version)` writes them, with the time of the engine's clock.

**The engine** (`crates/engine`): `Engine::agreement() -> Result<AgreementView, CommandError>`;
`Engine::accept_agreement(version: u32) -> Result<AgreementView, CommandError>`, refused with the new
`CommandError::AgreementChanged` when `version` is not the current one (the text changed while the sheet was open), and
with `SettingsReadOnly` or `Storage` as any save. `blocked::check` takes the settings and refuses with the new
`BlockedReason::Agreement` after the settings file and before the journal: every prepare and every confirm checks it
(`prepare_cleanup` and plan 036's `prepare_uninstall`, both through `rehearse_and_keep`, and `confirm_cleanup`). Put
back, picking a Largest row, the Trash view, scans and settings never check it.

**The commands and the guards** (`app/tauri`): `get_agreement`, `accept_agreement`; the bindings. The sandbox autotest
asserts, for every case file that offers a positive, that its first prepare is refused with `Agreement`, then accepts,
as a user would. The core e2e (`crates/e2e/tests/agreement.rs`) proves the refusal, the acceptance on a file store
across a restart, the version refusal and the reset.

**The interface** (`app/ui`): the action layer (`state/cleanup-actions.ts`) asks before it prepares: when the
agreement is not accepted (or not loaded yet), it dispatches `agreement-asked` with what to do next, and the frame shows
the **agreement sheet** (`product/agreement-sheet/`, on `design/layout/sheet`): the title, the "what changed" notice
when there is one, the intro and the five lines, "Read the full licence" (the **licence dialog**,
`product/licence-dialog/`, the two licence files bundled at build time), then "Not now" (first focus) and
"I understand". "Not now" starts nothing; scanning goes on. "I understand" accepts, then goes on with the action it
held. Settings → About always shows the five lines, when they were accepted (or that they were not yet), and the
licence button.

### 2. The danger zone (plan 02)

**The core's High consent.** `Confirmation` gains `high_risk_understood: bool` (JSON `highRiskUnderstood`, required).
`CleanupPlan::confirm` refuses with `ConfirmError::HighRiskNotUnderstood` when the plan's highest level is High and the
field is false, after the echo's fields are checked; the engine answers the new `CommandError::HighRiskNotUnderstood`
(text `core.error.high-risk-not-understood`), and the plan is consumed, as for any refused confirmation. A field set
true on a plan below High changes nothing. Nothing remembers it: it is per confirmation by construction.

**Review in two parts.** A fixed table beside `ZONE_ORDER` puts Safe and Low in the part `safe` and Medium and High in
the part `danger`. Review shows the parts in that order, each a section with its name and meaning from the core
(`core.danger.part.safe.{name,meaning}` "Safe to remove" / "Each of these comes back by itself or with a download.";
`core.danger.part.danger.{name,meaning}` "Danger zone" / "Each of these may hold settings or work that do not come
back by themselves. Check them one by one."); inside a part, the per-level zones as today, their heading one level
lower and named by the level (`core.danger.<level>.name`). The danger part is set apart by a border in the colour of
its highest level; the safe part has none. The summary names the highest level with its meaning, and when it is Medium
or High adds `ui.screen.review.reaches` before the button.

**The confirmation's High tick.** When the plan's highest level is High, the dialog shows a checkbox
"I understand these may be the only copy of my data" (`ui.confirm.high`, with `ui.confirm.high-description`), above
the permanent tick and never merged with it; the confirm button is inactive until it is ticked; both ticks start
cleared at every opening; the echo carries it.

**Scan.** "Select all" of a Medium or High zone, when it would select, opens a question first
(`ui.danger-zone.ask.*`: the level's meaning, "Nothing is removed until you confirm it in Review.", Cancel first
focus). Unticking never asks. Safe and Low zones select at once, as today. Nothing is pre-selected anywhere.

### 3. Help (plan 03)

A button "Removed something you need?" on the result summary and on Cleanups opens a dialog
(`product/recovery-help/`) with four short parts: put it back from Cleanups; find it in the Trash; what a permanent
delete means; how to report a wrong finding — the issue address as selectable text, never opened by the app. The
repository gains an issue form `.github/ISSUE_TEMPLATE/wrong-finding.yml` ("Fresher offered something it should not
have": what was offered, its category and level, why it should not have been, what was lost, macOS and Fresher
versions; a reminder to replace private paths with `~`).

### 4. The wording (plan 04)

**The guide**, `docs/writing.md`: the voice (plain, calm, second person, present tense, no jargon, no exclamation
marks, "Fresher" for the app); the shape of each kind of text (a category's five parts, a choice and a setting, a
left-out or skip reason, a result, a refusal, a hint); the glossary (remove, move to the Trash, delete permanently,
empty, put back, left out, skipped, danger level, the level names); the length limits; the banned words; how to change
the agreement. Each rule the check enforces is marked so.

**The check**, `scripts/check-texts.ts` (`npm run check:texts`): over the core's, the UI's and every module's
dictionary — no `!`; length limits by the key's kind; banned words; "delete" only about a permanent delete; "Trash"
capitalised; a description for every non-`ui` `.label`; all five parts of a category once any is present; no double
space, no space at either end; the long kinds end as sentences. Exemptions, if any, sit in the script with their
reason. It joins pre-push in the last pass step, once it reports nothing.

**The pass** is a procedure run over the texts as they stand at part B: the core's and the modules' dictionaries
(04/02), then the interface's, screen by screen, with the hints every control, number and icon lacks (04/03). It never
changes a key, a placeholder, a plural form, a danger level or what a text promises about safety; a category doc that
quotes a text changes with it; every changed screen is looked at in the gallery and the screenshots.

### 5. Tests

- Rust, negatives first: the agreement view and versions (core); the settings group, the update refusal, the reset,
  the blocked check order, the acceptance and its refusals (engine); the echo's High consent (core); the commands
  (app/tauri); the sandbox autotest for every case file; the core e2e `agreement.rs` and a High flow in it.
- UI, negatives first: the gate (not accepted, not loaded, Not now), the sheet, the licence dialog, About; Review's
  parts; the High tick; Scan's question; the help dialog; the mocks state only what the core can produce together
  (the agreement accepted by default; a script option for not accepted).
- e2e (Playwright): "Not now starts no cleanup", "a High plan cannot be confirmed without its tick", "Select all in
  the danger zone asks first" before the positive flows; visual and axe for the sheet, About, the two parts, the High
  tick, the question and the help dialog.
- The text check's own tests (`scripts/check-texts.test.ts`), negatives first.
- Coverage at or above 90 % of lines in Rust and in the UI.

## Risks

- **The agreement's legal worth**: lower than a lawyer's text; it makes the licences' disclaimer visible and recorded.
  The user decides on a lawyer (open question, the user's review list).
- **A cleanup path that skips `blocked::check`**: 01/01 lists the callers (`rehearse_and_keep` and
  `operation::confirm`); the engine's tests prove prepare, the uninstall's prepare and confirm; the e2e proves prepare.
- **The test helpers**: every engine built by a test that cleans must accept the agreement; done once per helper
  (`crates/engine/src/testing.rs` `Fixture::engine`, `app/tauri/src/testing.rs`, `crates/e2e/tests/support/home_engine.rs`,
  `cleaning/mod.rs`, `places.rs`, `jetbrains_home/mod.rs`, `app/tauri/src/sandbox/tests.rs`, the sandbox harness),
  never by weakening the check.
- **The confirmation echo changes shape**: every echo builder gains the field in the same step (02/01 lists the 19
  places on 740cf3c3, the cleaner's two test files and `crates/testing` among them), or the tests fail to read it.
- **A demo or a flow meets the sheet or the High tick**: the `cleanup` scenario accepts in 01/03; the e2e flows that
  confirm a High plan tick in 02/02; part B updates a scenario target whose text it changes.
- **The pass changes meaning by accident**: the rules of 04/02 and 04/03, the category docs read beside, the
  conclusions list what changed.

## Open questions

1. **A lawyer's look at the agreement and the licences** before the first public release: which country's law the
   author is under, and whether a consumer there can waive anything at all. It costs money and names a legal identity
   (as plan [013](../013-release/readme.md), question 2), so it is the user's. Not a blocker: recorded in
   [the user's review](../../deferred/user-review-2026-11-22.md).

## Decisions

- **2026-11-22** — the user delegated: build autonomously, the recommended decisions and agent's calls taken.
- **2026-11-22** — open question 1 stays the user's; the plan is built without it (agent's call 2).
- **2026-11-23** — the user delegated again; the steps were refreshed against `main` 740cf3c3 (agent's calls 14 to
  19). No decision of the design changed; two words of the agreement's Trash line follow today's code (call 19).

## Agent's calls (to check)

1. **Before the first cleanup, not the first launch**; once per version of the text. Cost: a person can scan without
   having read it; scanning changes nothing.
2. **Five plain lines and the licence one click away**; no scroll-to-accept, no legal wall; no promise a lawyer would
   have to stand behind. Cost: weaker than a lawyer's text.
3. **The engine enforces the agreement** at prepare and confirm (`BlockedReason::Agreement`), not only the sheet; put
   back never needs it. Cost: every test helper that cleans accepts it.
4. **Medium and High form the danger zone; only High needs the extra tick**, every time, and the core refuses a High
   plan without it (`ConfirmError::HighRiskNotUnderstood`), as it does for the permanent consent. Cost: the echo gains
   a field; every echo builder changes.
5. **A writing guide and an automatic check**, the check in pre-push. Cost: a new rule to satisfy on every text.
6. **Calm wording**: the zone is marked by structure, a border and the level's colour, not by alarm words.
7. **The acceptance in the settings file**, as two core settings of a group the screen never shows, written only by
   `accept_agreement`, kept by a reset (the ignore lists' precedent), instead of a file and a port of its own. Cost:
   the settings schema holds two values that are not choices.
8. **The full licence texts bundled into the UI** at build time from `LICENSE-MIT` and `LICENSE-APACHE`, opened in a
   dialog: no browser, no network. Cost: about 200 lines of text in the bundle.
9. **The sandbox autotest asserts the guard for every case file** that offers a positive, instead of one named case in
   `cleanup-guards.toml` (which gains a comment): the case-file format needs no new kind of case. Cost: the guard is
   not a line of one file's `EXPECTED.md`; a header sentence of every `EXPECTED.md` says the first cleanup asks.
10. **"A changed text asks again" is proven in the core's tests and the UI's unit tests**, not by a UI e2e scenario:
    the mock may state only what the core can produce, and no second version exists yet. Cost: no browser-level proof
    until a second version exists.
11. **The report address as selectable text**; the app opens no browser. Cost: one copy and paste for the user.
12. **The danger part's table lives in the UI** (`zones.ts`, beside `ZONE_ORDER`), its names and meanings in the core:
    a grouping of levels for display, like the zones' order. Cost: one more fixed table in the UI.
13. **The mocks state the agreement accepted by default**, with a script option for not accepted, so the existing
    flows are unchanged. Cost: the default mock is not a fresh install.
14. **Refreshed 2026-11-23: one guard site for every prepare.** `blocked::check` already runs in `rehearse_and_keep`,
    the tail of `prepare` and of the uninstall's `prepare`, and in `operation::confirm`; 01/01 changes only its
    signature, and a test proves the uninstall is refused too. Cost: none.
15. **Refreshed 2026-11-23: the cleaner's own test echoes gain the field.** `crates/cleaner/src/test_support.rs` (a
    `#[cfg(test)]` module) and `crates/cleaner/tests/never.rs` deserialize a `Confirmation`; a required field breaks
    them. They gain `"highRiskUnderstood"` and nothing else in `crates/cleaner` changes. Cost: the "no change to the
    cleaner" check becomes "no change outside those two test files".
16. **Refreshed 2026-11-23: a demo accepts as a user would.** `fixtures/demos/scenarios/cleanup.toml` gains a click on
    "I understand" (traced to `ui.agreement.accept`) after "Move to Trash…", in the step that adds the key (01/03).
    The world states nothing about the agreement and no production hook is added. Its plan is Safe, so no High tick.
    Cost: the recording shows the sheet for a moment; no GIF is committed yet, so nothing is recorded again now.
17. **Refreshed 2026-11-23: the text check's `description` rule skips a part's label.** A `.label` whose prefix also
    has an `.effect` (Docker's parts) is described by its `detail` and `effect`; adding four keys the module never
    emits, or four exemptions, would be worse. Cost: one more condition in the rule, tested.
18. **Refreshed 2026-11-23: High plans the UI already confirms.** Uninstall "everything", a Largest pick, Xcode
    archives and the downloads' large untouched files are High: their e2e flows tick the High box before "Move to
    Trash", and the e2e cleanup mock refuses a High echo without it, as the core does. Cost: more e2e edits in 02/02.
19. **Refreshed 2026-11-23: two words of the texts follow today's code.** The Trash line said "until you empty it"
    and named Docker as a tool that removes; since plan 037 Finder may empty the Trash by itself after 30 days, and
    since plan 032 Fresher never removes anything of Docker's. The line now reads "until the Trash is emptied" and
    "what an app's own tool removes"; the help dialog's two texts (03/01) follow. Version 1 is not yet given to anyone,
    so it is edited in place. `ui.confirm.how.tool` ("such as Docker") is listed for part B's pass. Cost: none.

## Conclusions

Not written yet.
