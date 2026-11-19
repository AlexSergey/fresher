# 006/05/02 — Needs an administrator, in the UI

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, engine, app, app/ui

## What exists already

- `Finding.removability: Removability` (`removable` | `needs-administrator` | `locked` | `protected-by-system`),
  set by the engine (05/01); the texts `core.removability.<state>.name` and `.how` (none for `removable`; the
  `protected-by-system` text explains and instructs nothing).
- The plan builder already leaves out an item that is not removable: `SkipReason::NotRemovable { removability }`
  (`crates/engine/src/cleanup/plan.rs`), whether it was picked one by one or through its whole category; the review
  screen shows the left-out items with `core.removability.<state>.how` (`app/ui/src/product/cleanup-texts.ts`). So
  "select all" needs no new count in the UI: the core leaves them out and the review says how many and why.
- `Engine::finding(id) -> Result<Finding, CommandError>` (an id of another scan → `UnknownFinding`);
  `Engine::reveal_in_trash` and `cleanup::history::reveal` show the pattern of a reveal through
  `ports.system_ui.reveal` (error → `EngineError::SystemUi`).
- `Subject` is `Path(PathBuf)` or `External { tool, id }` (a Docker image).
- `MacSystemUi::reveal` (03/03) runs `/usr/bin/open -R <path>`.
- `app/ui/src/product/finding-row/finding-row.tsx` — a 52 px row inside a virtual list: checkbox, title over subject,
  danger badge, size; `app/ui/src/product/finding-list/finding-list.tsx` renders the rows;
  `app/ui/src/screens/module/category-section.tsx` passes `onToggle`.

## Files

- `crates/core/src/error.rs` — `CommandError::NothingToReveal` (code `nothing-to-reveal`, kebab-case like the others,
  in the error's sample test) and its text `core.error.nothing-to-reveal` in `crates/core/locales/en.json` ("This item
  is not a file or folder, so it cannot be shown in Finder.").
- `crates/engine/src/engine.rs` — `pub fn reveal_finding(&self, id: FindingId) -> Result<(), CommandError>`:
  `self.finding(id)?`; a `Subject::External` → `NothingToReveal`; a path → `ports.system_ui.reveal(path)`, an error →
  `EngineError::SystemUi` as `reveal_in_trash` maps it. If `engine.rs` (769 lines) would pass the size the project
  splits at, the body goes to a small `crates/engine/src/reveal.rs` and the method delegates.
- `app/tauri/src/commands.rs` — `reveal_finding(engine, finding: FindingId) -> Result<(), Refusal>`, takes an id,
  never a path; `#[tauri::command(async)]`.
- `app/tauri/src/lib.rs`, `app/tauri/build.rs`, `app/tauri/capabilities/default.json` — `reveal_finding` /
  `allow-reveal-finding`, in the same order in all three.
- `app/ui/src/ipc/index.ts`, `index.test.ts` — `revealFinding(finding: FindingId)` and its key in `COMMANDS`.
- `app/ui/src/product/finding-row/finding-row.tsx` — a finding that is not `removable` keeps its size and danger, has
  no checkbox (it cannot be ticked), and shows a badge with the short label (icon and word,
  `core.removability.<state>.name`); its instruction `.how` on demand in a tooltip or popover of the badge (the row's
  height stays 52 px); `[Show in Finder]` (an icon button with its label) for `needs-administrator` and `locked` with
  a path subject, calling an `onReveal(finding)` prop; none for `protected-by-system`.
- `app/ui/src/product/finding-list/finding-list.tsx`, `app/ui/src/screens/module/category-section.tsx` — pass
  `onReveal` (`revealFinding(finding.id)`), a refusal shown like the other command failures.
- `app/ui/src/product/finding-row/finding-row.stories.tsx` — a row in each `Removability`.
- A test over both dictionaries (`crates/core/locales/en.json` in `crates/core/src/texts.rs`, `app/ui/src/locales/en.json`
  in `app/ui/src/i18n/`) that no text contains `sudo` (agent's call 9).

## Produces

The visible half of "the app never runs as administrator": the user sees the space and how to free it themselves.

## Checks

- Negative (first): `an_item_that_needs_an_administrator_has_no_checkbox`,
  `a_protected_item_offers_no_instruction_and_no_finder_button`, `reveal_of_an_unknown_id_is_refused`,
  `reveal_of_an_external_subject_is_refused` (`NothingToReveal`), `a_failed_reveal_is_reported`,
  `no_text_mentions_sudo` (both dictionaries), and the existing plan test that a category selection leaves out an
  item that is not removable (`crates/engine/src/cleanup/plan.rs`; added only if missing).
- Positive: `show_in_finder_reveals_the_findings_path` (`FakeSystemUi` records the path),
  `the_instruction_is_shown_on_demand`, `reveal_finding_is_allowlisted_everywhere` (the config test), the UI IPC test.
- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-app`; `npx vitest run` on the changed UI files;
  the gallery shows a row in each `Removability`; the Playwright screenshots are added and looked at.

## Conclusions

Built as the step says: an item the core says the app may not remove is shown with its level and size, has no
checkbox, says why in a short badge, gives what the user can do on demand, and, where the user can remove it by hand,
offers "Show in Finder" by the finding's id.

- **Core.** `CommandError::NothingToReveal` (code `nothing-to-reveal`, the last of `CODES`, now 27) with its text
  `core.error.nothing-to-reveal`; it is in the error's sample, so the code, text and argument tests cover it.
- **Engine.** `Engine::reveal_finding(id)` lives in `crates/engine/src/reveal.rs` as an `impl Engine` block of its own,
  the pattern of `ignore.rs`, rather than a method in `engine.rs` that delegates (decision: `engine.rs` stays as it
  was, at 769 lines, and the body and its tests sit together). `self.finding(id)?` refuses an id of another scan or
  an ignored one; a tool's object is `NothingToReveal`; a path goes to `SystemUi::reveal`, whose error is
  `EngineError::SystemUi`, an internal error, as `reveal_in_trash` maps it. The engine reveals only the finding's own
  path: the UI never sends one.
- **Shell.** `reveal_finding(engine, finding: FindingId)`, `#[tauri::command(async)]`, after `open_access_settings`
  in `lib.rs`, `build.rs` and `capabilities/default.json`; `reveal_finding_is_allowlisted_everywhere` checks it is at
  the same place in all three. The bindings were regenerated by the generator (`UPDATE_BINDINGS=1`): only the
  `CommandError` union changed.
- **UI.** `revealFinding(finding)` in the IPC module. `FindingRow` takes an optional `onReveal`; for a finding that is
  not `removable` the checkbox's place stays empty (the titles still line up), the row is never shown selected, its
  accessible name adds the removability's word (`ui.finding-list.row-kept`), and before the level column stands
  `RemovabilityNote` (`product/finding-row/removability-note.tsx`): a badge with the state's icon (a key for
  "Needs an administrator", a lock for "Locked", a keyhole lock for "Protected by macOS") and the core's word, a tab
  stop whose tooltip and description are the core's `.how` sentence, and an icon button "Show in Finder" for
  `needs-administrator` and `locked` with a path subject. A protected item and a tool's object get no button. The row
  stays 52 px. `FindingList` passes `onReveal` and never toggles such a row on Space; the module page's category
  passes `revealFinding(finding.id)` and a refusal goes to the store as `command-failed`, as every command's does. The
  review screen's lists pass no `onReveal`: they show the badge, no button.
- **Decision: the demo fixture of the UI tests is removable.** `DEMO_FINDINGS` (`app/ui/src/test/demo-scan.ts`) said
  `needs-administrator` since plan 008, a value nothing read; with this step it would have taken the checkbox off
  every demo row the selection, review and cleanup tests tick. It is `removable` now; the non-removable rows have
  fixtures of their own (`ADMIN_FINDING`, `LOCKED_FINDING`, `PROTECTED_FINDING`, `ADMIN_EXTERNAL_FINDING`).
- **No `sudo`.** `no_text_mentions_sudo` in `crates/core/src/texts.rs` and "no text mentions sudo" in
  `app/ui/src/i18n/chrome-keys.test.ts` read every text of the two dictionaries.
- **Category selection.** `a_category_selection_leaves_out_an_item_that_is_not_removable` was added to
  `crates/engine/src/cleanup/plan.rs`: the whole category is selected, and the item that needs an administrator is
  left out with `NotRemovable`. The existing test covered only the sort of a list of findings.
- **Checks run.** `cargo nextest run -p fresher-core -p fresher-engine -p fresher-app`: 925 passed; `cargo fmt`,
  `cargo clippy --workspace --all-targets -- -D warnings`: clean; `npx vitest run`: 1105 passed; `tsc`, ESLint and
  knip: clean. The gallery shows a row in each `Removability` (`FindingRow/locked`, `needs-administrator`,
  `protected-by-system`; `safe` is removable); their three Playwright screenshots were added and looked at, and the
  keyboard test's focus photograph of `FindingRow` (the badge is now a focus stop) was added. Four WebKit failures
  are older than this step and fail the same on the commit before it: the two module page screenshots of Removed
  apps' leftovers, the Temporary files number field, and the keyboard test's part 8 (`PermissionCard` focus; a
  missing `PermissionRows` focus baseline is written by any run).
