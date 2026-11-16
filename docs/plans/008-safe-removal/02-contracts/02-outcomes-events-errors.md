# 008/02/02 — Outcomes, events, commands and errors

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core, engine, app

## Files

- `crates/core/src/cleanup/outcome.rs` (new) — `SkipReason` (every reason of the design, sections 3 and 5:
  `not-in-sandbox`, `outside-roots`, `protected`, `whole-folder`, `ignored`, `through-link`, `vanished`,
  `other-volume`, `replaced`, `kind-changed`, `not-removable { removability }`, `missing-capability { capability }`,
  `in-use { app }`, `grown`, `no-trash`, `covered`, `not-the-subject`, `forced-tool`, `check-failed`), `ItemOutcome`
  (`moved`, `emptied { moved, skipped }`, `deleted`, `tool-done`, `skipped { reason }`, `failed { reason: Text }`,
  `not-reached`), `ItemReport { index, finding, outcome }`, `Ended` (`completed`, `cancelled`, `stopped`),
  `CleanupReport` (method, ended, counts, `moved: Sizes`, `deleted: Sizes`, `measured: Option<ByteSize>`), and each
  one's `text()`.
- `crates/core/src/cleanup/view.rs` (new) — what the commands return: `CleanupPlanView` (summary, groups by category
  with their items and how they go, left out, `permanentAvailable`), `CleanupSummary` and `CleanupsPage`,
  `CleanupDetail` (items with outcome and `TrashState`), `PutBackReport`, `BlockedReason` (`settings`, `journal`,
  `open-files`). camelCase fields, kebab-case tags, no path except the display path of a plan item.
- `crates/core/src/event.rs` — `CleanupStarted`, `CleanupItems { cleanup, done, items }`, `CleanupFinished`,
  `FindingsRemoved { scan, findings }`.
- `crates/core/src/command.rs` — `EngineStatus::Cleaning { cleanup }`; `LastScan.changedSince: Option<CleanupId>`.
- `crates/core/src/error.rs` — `UnknownCategory`, `NothingToClean`, `TooManyItems { limit }`, `PlanGone`,
  `PlanMismatch`, `CleaningBlocked { reason }`, `UnknownCleanup { id }`, `UnknownCleanupItem`; `CODES` and `text()`.
- `crates/core/src/texts.rs`, `crates/core/locales/en.json` — `core.error.<code>` for the new refusals;
  `core.cleanup.skip.<reason>` (what happened) and `.how` where there is something to do ("Quit {app} and try
  again."); `core.cleanup.outcome.<kind>`; `core.cleanup.blocked.<reason>`; plain English, rule 7 of
  [`rules.md`](../../../rules.md).
- `crates/engine/` — the status mapping for `Cleaning` (the state itself comes in 06/02); nothing else.
- `app/tauri/tests/bindings.rs` — the new types listed; `app/ui/src/ipc/bindings.generated.ts` regenerated
  (`UPDATE_BINDINGS=1`); the UI's reducer ignores the new events until plan 07 (one case each).

## Produces

The IPC surface of plan 06 and the state of plan 07; the texts every refusal and outcome shows.

## Checks

- Negative: `every_skip_reason_has_its_text`, `every_new_code_has_its_text` (the existing exhaustive test of
  `CommandError` grows), `no_view_holds_a_path_but_the_display_one` (serialized views of a sample hold no other path).
- Positive: `outcomes_serialize_with_kebab_case_tags`, `the_report_counts_by_outcome`,
  `cleaning_status_carries_its_cleanup`, `exports_the_cleanup_types_to_typescript`.
- The bindings test passes after regeneration; `npm run test:ui` passes (the reducer's new cases).

## Conclusions

- Built as written: `cleanup/outcome.rs` and `cleanup/view.rs`, the five events, `EngineStatus::Cleaning { cleanup }`,
  `LastScan.changedSince`, the new `CommandError`s with their texts, and 67 new keys in `crates/core/locales/en.json`.
  The bindings are regenerated (32 new declarations, listed by `cleanup_declarations()` in `bindings.rs`); the UI's
  reducer passes the five new events through unchanged (one test case each) until plan 07, and two tests take the new
  `cleaning` status with its cleanup.
- Added because the core is frozen after this plan and later plans need them (design, sections 8 and 10):
  - **`Event::QuitRequested`** (`quit-requested`), which 06/02's `request_quit` publishes.
  - **The commands' inputs**: `CleanupsQuery` (`MAX_LIMIT` 100), `PutBackRequest` (`items: null` puts back all) and
    `CleanupItemRef` (for `reveal_in_trash`), all `deny_unknown_fields` and ids only.
  - **`CommandError::NotInTrash`** for `reveal_in_trash` of an item no longer there (06/03), and
    `BlockedReason::DiskFull` for the full disk of call 22, beside `settings`, `journal` and `open-files`.
  - **`Failure`** (`trash-refused`, `landed-elsewhere`, `delete-failed`, `tool-failed { error }`, `tool-missing`):
    the cleaner builds `ItemOutcome::Failed { reason: Failure::….text() }`, so every failure text is a core key.
  - **`SkipCode`**, a skip reason without its details, and **`RecordedOutcome`** (with `Unknown` for an item a crash
    left without outcome): what the journal keeps and the history shows, since a `Capability::Tool` holds a
    `&'static str` and cannot be read back. `ItemOutcome::recorded(status)` maps one to the other.
  - **`TrashState` and `PutBackOutcome`** moved here from 04 (`cleanup/outcome.rs`), since the views carry them;
    `PutBackOutcome` also has `NotTrashed` (a deleted or tool item) and `Refused { reason: SkipCode }`.
  - **`Ended::Interrupted`**, only in the history (a `begin` with no `end`).
- Not spelled out by the step, taken the simplest way:
  - **Texts**: `core.cleanup.skip.<code>` has no placeholder, so the history shows it from the code alone;
    `.how` exists for `replaced`, `kind-changed`, `missing-capability`, `in-use` ("Quit {app} and try again."),
    `grown`, `no-trash` and `check-failed`; `NotRemovable`'s instruction is the removability's own. Also
    `core.cleanup.{outcome,failed,ended,trash,put-back,blocked}.<id>`. The UI builds these keys from the enums' tags,
    as it does for danger levels; `PutBackItem` carries its text, as 06/03 asks.
  - **`core.error.cleaning-blocked`** says only that nothing was removed; the error carries `reason`, whose text
    (`core.cleanup.blocked.<reason>`) says what to do. The key of every error stays `core.error.<code>`.
  - **`LastScan.changedSince`** is optional in TypeScript (`changedSince?: CleanupId`) and left out of the JSON when
    there is none, so no golden file and no TypeScript sample of a last scan changes.
  - **Views**: `CleanupPlanView { plan, summary, groups, leftOut, permanentAvailable }` with `PlanGroup { category,
    action, summary, items }` (by category and by `ActionKind`) and `PlanItemView::of(&PlanItem)`; the engine builds
    the view (06/01). `CleanupsPage { cleanups, offset, total, waiting, blocked }`, `CleanupSummary` (with `counts`,
    `moved`, `deleted`, `inTrash`), `CleanupDetail { summary, items }` with `CleanupItemView` (display path,
    category, danger, sizes, action, outcome, `trash`, `putBack`). Views have public fields, like `Overview`.
  - **`CleanupReport::new(method)` and `count(outcome, sizes)`**: a moved item adds to `moved`, a deleted or a tool's
    item to `deleted`, an emptied folder to the one of the method; sums saturate (marked partial) instead of failing.
  - The engine has no cleaning state yet (06/02 adds it), so its status mapping is unchanged; it only sets
    `changed_since: None` where it builds a `LastScan`. The texts test reuses `error::tests::one_of_each` instead of
    a copy of it.

