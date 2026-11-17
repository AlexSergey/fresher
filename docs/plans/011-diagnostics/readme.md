# 011 — Problems and reasons

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, cleaner, testing, modules, app, app/ui

Expanded on 2026-11-17 from the one-file draft "Errors, logs and diagnostics" of 2026-11-15, in the light form the
user asked for: only what the first feature modules need from day one. Approved the same day by delegation: the user
asked the agent to take the recommended answer to every open question and record it ([`00-design.md`](./00-design.md),
"Decisions"). The agent's calls wait for the user's check. Logs on disk, the bug-report file, match explanations and a
diagnostics screen are [deferred](../../deferred/README.md).

## Goal

No raw technical error ever reaches the user: every problem is a plain-English text with a next step where there is
one, and the tests check it. A module reports why in one way each: its part failed (`ModuleError::Failed`, as today),
or it recognized an item and left it out (`out.left_out(…)`, new), with texts it declares. The user sees what was left
out per category and why, and a failed start says so instead of the app vanishing. Nothing new is written to disk.

Branch: `feat/011-diagnostics`, squash-merged into `main` once when done ([`../../workflow.md`](../../workflow.md)).
**Built after plan [012](../012-storage-map/readme.md) is merged into `main`**, on this branch made again from `main`
then (decision 5): both plans edit `crates/core/src/error.rs` and the two dictionaries. Depends on plans
[002](../002-base-architecture/readme.md), [003](../003-base-ui/readme.md), [005](../005-settings/readme.md),
[007](../007-safe-development/readme.md) and [008](../008-safe-removal/readme.md) (done or merged).

## Plans

| #   | Plan                                           | Status                                  | Depends on | What                                                                                  |
| --- | ---------------------------------------------- | --------------------------------------- | ---------- | ------------------------------------------------------------------------------------- |
| 01  | [Problem texts](./01-problem-texts/readme.md)  | ✔️ done | —          | the `detail` rule and the tool's message, a failed start's window, the UI's side       |
| 02  | [Reasons](./02-reasons/readme.md)              | ✔️ done | 01/01      | declared problems and `left_out` in the core, counted by the engine, shown by the UI  |
| 03  | [Finish](./03-finish/readme.md)                | 🔨 in progress | all        | the docs, a run by hand, the conclusions                                              |

### Order and parallel work

```
01/01 ──┬── 01/02 ──────────────────────┐
        ├── 01/03 ──────────────────────┤
        └── 02/01 ── 02/02 ── 02/03 ────┴── 03/01
```

- **Wave 1**: 01/01 (the core's `detail` rule; every later step leans on it).
- **Wave 2, in parallel**: 01/02 (`app/tauri` only, and one core text), 01/03 (`app/ui` only) and 02/01 (the core's
  module API). 01/02 and 02/01 both add a key to `crates/core/locales/en.json`: whichever lands second rebases on the
  first.
- **Wave 3, in order**: 02/02 (engine, testing, modules), 02/03 (the UI; after 01/03, both edit
  `app/ui/src/locales/en.json`).
- **Wave 4**: 03/01.

Which files each plan touches, so parallel work stays apart. Plan 012 (merged first) touched the files marked *012*;
this plan builds on them, never against them.

| Plan | Owns                                                                  | Shared files it touches                                                                                                                                                                                                                                      |
| ---- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 01   | `app/tauri/src/start_failure.rs`, `app/ui/src/product/problem-text/`                              | `crates/core/src/error.rs` *012*, `crates/core/src/{text,texts}.rs`, `crates/core/src/cleanup/outcome.rs`, `crates/core/locales/en.json` *012*, `crates/cleaner/src/remove/tool.rs`, `app/tauri/src/lib.rs` *012*, `eslint.config.ts`, `app/ui/src/product/{result-summary,module-section}/`, `app/ui/src/locales/en.json` *012* |
| 02   | `crates/core/src/left_out.rs`                                         | `crates/core/src/{lib,module,event,texts}.rs`, `crates/core/locales/en.json` *012*, `crates/core/tests/typescript.rs`, `crates/engine/src/{session,testing}.rs`, `crates/testing/src/{run_module,contract}.rs`, `modules/{demo,trial}/`, `app/ui/src/ipc/bindings.generated.ts` (regenerated), `app/ui/src/state/reduce.ts`, `app/ui/src/screens/module/category-section.tsx`, `app/ui/src/locales/en.json` *012* |
| 03   | —                                                                     | `docs/` (architecture, testing, deferred, plans README)                                                                                                                                                                                                      |

Plan 012 also added `EngineError::StorageMap` and `CommandError::UnknownVolume`; this plan changes neither, and its
tests run over every `CommandError` through `one_of_each`, so they cover `UnknownVolume` too. `bindings.generated.ts`,
`Cargo.lock` and `package-lock.json` are never merged by hand: regenerate them after a merge and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- No serialized `Refusal` holds its error's `Display` form; no core or module template names `{detail}`; a tool's
  failure reads as a plain sentence with its next step, and the tool's line appears only behind "Details".
- A module that reports a failure or a left-out with a key it did not declare fails its contract test; every declared
  key without a text fails it too.
- `npm run dev`: the demo module's left-out line shows under its category with its count, size and reason; a scan
  restored from the cache after a restart shows none until the next scan (agent's call 4: left-outs are event-only).
- A development build started with `FRESHER_SANDBOX=relative` shows the failed-start window with the plain sentence and
  the refusal under it, then ends; nothing was written.
- The sandbox autotests (`npm run sandbox:test`) pass unchanged: no case file or golden report changed.

## Ideas this touches

- [Best practices](../../ideas/best-practices.md): "reasons for every skip" (built here for modules), "why it matched,
  and how sure" and the read-only report ([deferred](../../deferred/match-explanations.md),
  [deferred](../../deferred/bug-report-file.md)).
- The [storage explainer](../../ideas/storage-explainer.md): its "not measured" reasons are plan 012's; this plan
  adds nothing to them.

## Conclusions

**Merged into `main` on 2026-11-17 at the user's request, before the run by hand** (03/01): the plan stays 🔨 in
progress until the user has run its checklist; the results are written then, on `main`.

Built in three plans on `feat/011-diagnostics`. No raw technical error reaches the user: a detail is an arg no sentence
names, a failed start has its own window, and a module's failure and left-outs are declared texts the contract checks.
What changed from the design:

- Design call 4 stands, reworded: left-outs are event-only; the restore runs during setup, before the UI subscribes,
  and its events are not replayed, so after a restart no left-out line shows until the next scan. An earlier
  "correction" that said the lines come back was wrong and is withdrawn.
- An ignored category's left-outs are not counted by the engine, and the module page drops a left-out-only category
  once it is ignored; such a category's row has neither a box nor a disclosure.
- The detail's disclosure is one stable "Details" button whose `aria-expanded` carries the state.
- The type is `LeftOutItem`, because `LeftOut` is plan 008's.
- The UI lint is a name-based tripwire, not a guarantee.
- A foreign-category left-out is dropped with a once-only warning and refused by the contract.
- `trial.error.failed` had no next step; it has one now.

Left: the run by hand (03/01), the webkit/chromium follow-ups (a chromium keyboard test on Dialog/long-body fails
without this plan's changes too), and three minor UI findings of 01/03 and 02/03 (a stable disclosure label,
`aria-controls`, `en.json` key order).

### Agent's calls taken during the build (to check)

1. Implementers adapt to the code as it is after plans 007 and 012 were merged. Cost if wrong: the reviewer flags drift.
2. The failed-start dialog is non-blocking, the main window is destroyed and an exit without a code is held until the
   dialog is dismissed. Cost if wrong: a stuck dialog keeps a half-dead app alive; Cmd-Q waits for it.
3. The UI lint is name-based and called a tripwire. Cost if wrong: an aliased or destructured read slips through.
4. The new type is `LeftOutItem`. Cost if wrong: none beyond a name.
5. A foreign-category left-out is dropped (warned once) and refused by the contract. Cost if wrong: none; such a count
   has nowhere to render.
6. The core e2e goldens were regenerated for `leftOut`. Cost if wrong: they had been red since 02/01.
7. `trial.error.failed` got a next step in 03/01. Cost if wrong: a wording to change.

