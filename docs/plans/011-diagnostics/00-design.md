# 011 — Problems and reasons: design

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, cleaner, testing, modules, app, app/ui

Expanded on 2026-11-17 from the one-file draft of 2026-11-15, in the light form the user asked for, and approved the
same day by delegation: the user asked the agent to take the recommended answer to every open question and record it
("Decisions"). The agent's calls wait for the user's check.

## Intent

The user's task (the draft of 2026-11-15): one way for a problem to travel from the core to the user, one way to log,
and a way to see why the app did what it did, without writing the user's paths to disk and without sending anything
anywhere.

The user's decision of 2026-11-17, translated: "Take 011 now, in a light form, before the first feature modules: only
what the modules will need from day one. One, a single path for a problem from the core to the user: typed errors
become plain-English text with a next step where there is one, and no raw technical error is ever shown. Two, the
'why': why an item was skipped, left out or not scanned, reusing what plans 008, 012 and 005 already have, with one
way for a module to report a skip or a failure with its reason. Logs on disk, the bug-report file, explanations of why
something matched beyond what exists, and any diagnostics screen are deferred."

## Goals

1. **No raw technical error reaches the user.** Every problem the user sees is a text of a dictionary, in plain
   English, with a next step where there is one; this is checked by tests, not by review alone.
2. **One way for a module to say why.** A module reports that its part failed, or that it recognized an item and left
   it out, in exactly one way each, with a text it declared; the contract test checks the texts.
3. **The user sees why.** What a module left out is counted per category and reason and shown under the category; a
   failed start says so in a window instead of the app vanishing.
4. **Nothing new is written to disk and nothing leaves the Mac** ([`../../restrictions/privacy.md`](../../restrictions/privacy.md)).

## Non-goals (deferred, each with its file in [`../../deferred/`](../../deferred/README.md))

- [A log file of a session](../../deferred/session-log-file.md), opt-in, and the scan's phase timings.
- [The bug-report file](../../deferred/bug-report-file.md) the user makes and sends, with the notice after an
  unexpected exit.
- [Why an item matched](../../deferred/match-explanations.md), beyond the finding's reason text that exists today.
- [A diagnostics corner](../../deferred/diagnostics-screen.md) of settings: versions, permissions, scan statistics,
  recent problems, the problems of past cleanups.
- Crash upload, telemetry, remote logging: forbidden, never planned.

## What already exists

The audit of `main` at `362cd47` and of plan 012's branch (merged before this plan is built).

| What                         | Where                                                                                     | State                                                                                                                                                              |
| ---------------------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| A refused command            | `CommandError` and `Refusal` (`crates/core/src/error.rs`); every Tauri command maps to it | one shape: a `code` tag, the variant's fields, and `text()` → `core.error.<code>`; `CODES` lists every code; `texts.rs` checks each has a text in `locales/en.json` |
| Errors inside the engine     | `EngineError` (`crates/engine/src/error.rs`)                                              | every one becomes `CommandError::Internal` and is logged; a test checks every variant                                                                              |
| A module's failure           | `ModuleError::Failed { reason: Text }` → `ModuleOutcome::Failed` in `Event::ModuleFinished` | the module's own text; a panic becomes `core.module.panicked`; the contract test checks a failure's text only when a run of the tests fails                       |
| A module that cannot run     | `ModuleOutcome::Unavailable { missing }`                                                  | the capability texts say what to allow                                                                                                                             |
| What the scan did not read   | `Coverage` (`not_entered`, `unreadable` by `Unreadable`); 012: `NotScanned`, `RemainderReason` | counts and capabilities, no paths; each reason has a text                                                                                                  |
| Why the cleaner skipped      | `SkipReason` / `SkipCode` (008), `Failure`, `BlockedReason`                               | each with a text; the journal keeps codes and statuses                                                                                                             |
| What the user ignored        | the ignore list (005): `SkipReason::Ignored`, `IgnoredCount` on the overview              | shown as counts                                                                                                                                                    |
| Why an item is offered       | `NewFinding`'s `reason` text                                                              | one sentence per rule or module                                                                                                                                    |
| The UI's side                | `CommandFailed`, `Failure` (`refused` / `unreachable`) in `app/ui/src/ipc/index.ts`      | the UI shows `refusal.text` or `ui.core.unreachable`; a missing key shows the key, so a gap is visible                                                            |
| Logs                         | `tracing` in the engine and the app                                                       | no file; no subscriber writes to disk                                                                                                                              |

**Gaps the audit found**, the whole of this plan:

1. `Failure::ToolFailed { error }` puts the first line of a tool's standard error into the sentence the user reads
   (`"The tool could not remove it: {error}"`): the one raw technical error that reaches the user today.
2. A failed start (`compose::start`, the data folder, the identity check) returns an error from Tauri's setup: the
   app ends without a word.
3. A module can only push findings or fail whole. An item it recognized and chose not to offer (the app that owns it
   is running, a tool's object it could not read) is dropped without a trace, so the user cannot learn why.
4. A module's failure text is checked only when a test happens to run that failure; nothing lists the texts a module
   may report.
5. Nothing checks that the serialized refusal never holds the error's `Display` form, and the rule "the `Display` of
   an error is for the log" is written only in a doc comment.

## Design

### 1. One shape for a problem the user sees

A problem reaches the user in one of three forms, all already typed; this plan adds no fourth.

| Form                   | Carried by                                                                   | Text                                            |
| ---------------------- | ---------------------------------------------------------------------------- | ----------------------------------------------- |
| A command refused      | `Refusal { error: CommandError, text: Text }`                                | `core.error.<code>`                             |
| A part that ended badly | `ModuleOutcome::Failed { reason }`, `ScanOutcome::Failed { reason }`, `ItemOutcome::Failed { reason }`, `ItemOutcome::Skipped { reason }` | the module's declared text, or the core's |
| A reason something was not done | `Coverage`, `NotScanned`, `RemainderReason`, `SkipReason`, and the new left-out counts (section 3) | each reason's own key                  |

The rules of the shape, written into [`../../architecture/README.md`](../../architecture/README.md) by the finish:

- **A text is the only thing shown.** The UI renders a `Text` through the dictionary; it never shows an error's code,
  its `Display`, its `Debug` or a JavaScript `Error.message`.
- **The sentence says what happened and what to do.** "What happened" in plain words, what stayed safe when it
  matters ("nothing was removed"), then the next step as the last sentence ("Open Settings, then Permissions…",
  "Scan again…"). A next step is a sentence, not a button (decision 1).
- **A technical detail is a `detail` arg, never part of the sentence.** A text may carry an arg named `detail` (a
  tool's own message); no template names `{detail}`, and the UI shows it only on demand, under the sentence. Checked
  in the core's dictionary and in every module's (01/01, 02/02).
- **The `Display` of an error is for the log.** `tracing` keeps it in memory; it is never serialized to the UI.

### 2. The gaps of errors, closed

- **A tool's failure** (01/01): `Failure::ToolFailed` keeps the tool's first line (200 characters at most, as today)
  as the `detail` arg; the sentence becomes plain, names no output and gives the next step. The cleanup's result
  shows the detail behind "Show the details" (01/03). The journal keeps only the status, as today.
- **A failed start** (01/02): when the setup fails, the app shows a native message with the core's text
  `core.start.failed` ("Fresher could not start. It changed nothing on your Mac. Quit and open it again; if this
  happens again, reinstall Fresher.") and then ends. In a development build the error's `Display` follows as the
  detail, so a sandbox refusal is readable; a release build shows only the sentence.
- **The UI** (01/03): one small component shows a problem's text and, when it has one, its `detail` behind "Show the
  details"; the cleanup's result list and the module's section use it. One lint keeps an error's `.message` and a
  refusal's `error.code` out of `screens/` and `product/`; `CommandFailed`'s message, which is only for the
  developer's console, stays as it is.

### 3. One way for a module to say why

Two ways for a module to report a problem, and no third:

| The module                                         | How                                                        | The user sees                                         |
| -------------------------------------------------- | ---------------------------------------------------------- | ----------------------------------------------------- |
| could not do its part (a tool failed, a bug)       | `Err(ModuleError::Failed { reason })`, as today            | the module's section says so; the other modules finish |
| recognized an item and chose not to offer it       | `out.left_out(LeftOutItem { category, reason, sizes })` (new)  | "2 items left out (1.2 GB): the app that made them is open" under the category |

Everything else already has its way and stays where it is: what the walk could not read is `Coverage` (the module
does nothing); a missing permission or tool is `Unavailable` and the category's needs; a path no rule recognizes is
not claimed and stays in "Your folders" (no "Other", rule 3); the cleaner's checks are `SkipReason`.

- `LeftOutItem { category: CategoryId, reason: TextKey, sizes: Sizes }` holds **no path and no args**: left-outs of one
  category and one reason add up into one line, and nothing about the item is kept. The size is what the user would
  have freed.
- **Declared problems.** `ModuleDescriptor::with_problems(keys)` lists every key a module may report as a failure or
  a left-out reason: in the module's namespace, each once. `check_module_contract` checks each has a text;
  `check_texts_used` checks every failure and left-out of the runs uses a declared key, and counts a declared key as
  used. A module with no problem declares none.
- **The engine counts.** The session's sink adds left-outs per (category, reason): items and sizes. They travel with
  the module's end, `Event::ModuleFinished { left_out: Vec<LeftOutCount> }`, beside its outcome, like a failure today:
  event-only, not in the scan cache (agent's call 4). A left-out whose key the module did not declare is still counted
  and logged once as a bug; the contract test is the guard.
- **The UI** (02/03) keeps the counts per module and shows each under its category: the core's line
  `core.left-out.summary` ("{count} items left out ({size}):", plural forms) followed by the reason's text.

### 4. What the first modules do

The module checklist ([`../../architecture/modules.md`](../../architecture/modules.md)) gains two boxes: every
failure and left-out text is declared and plain English with its next step; an item the module recognizes and does
not offer is reported with `left_out`, never dropped silently. The `demo` module reports one left-out of a made-up
item so the path is seen end to end in a development build; `trial` declares its one failure text. No module changes
what it offers.

### 5. Sandbox case file

None in this plan (rule 8 applies where a plan adds or changes a module or a category): no category is added, and no
module changes what it offers. `trial` only declares its existing failure text (its case files and golden reports
do not change); `demo` has no case file by the rule. The first module that reports a left-out adds the expectation
to the case format with its own case file (agent's call 6).

### 6. Where the code lives

| Crate or folder        | What it gains                                                                                  |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| `fresher-core`         | `LeftOutItem`, `LeftOutCount`, `FindingSink::left_out`, `ModuleDescriptor::with_problems`, the field of `ModuleFinished`, the texts `core.start.failed` and `core.left-out.summary`, the `detail` rule's test, `ToolFailed`'s text |
| `fresher-engine`       | the counting in the session's sink                                                             |
| `fresher-testing`      | `ModuleRun::left_out`, the contract's checks of declared problems and `{detail}`                |
| `fresher-cleaner`      | `ToolFailed` with its detail                                                                   |
| `modules/demo`, `modules/trial` | the declared problems; demo's one left-out                                            |
| `app/tauri`            | the failed start's message                                                                     |
| `app/ui`               | the detail on demand, the lint, the left-out lines                                             |

No new crate and no new third-party dependency: the message uses `tauri-plugin-dialog`, already in the app.

## Open questions

None left: each was answered with its recommended option, by delegation (decisions 1–5).

## Decisions

The user's, of 2026-11-17 (translated): take 011 now in a light form before the first feature modules; the two parts
in the intent; logs on disk, the bug-report file, match explanations and a diagnostics screen deferred; the plan
expanded autonomously with the recommended decisions accepted.

Taken by delegation on 2026-11-17 (the user's instruction: accept the recommended decisions):

1. **A next step is a sentence of the text, not a button.** Options: (a) the last sentence of the text — recommended:
   every error text already works this way, and no module needs a button yet; (b) a typed action with a button
   ("Open Settings", "Scan again") on every refusal. A typed action comes with the first screen that needs it.
2. **A tool's own message is a detail shown on demand.** Options: (a) a `detail` arg the sentence never names, shown
   behind "Show the details" — recommended: the user's rule says no raw error, and the line is the only clue
   to why a tool refused; (b) dropped, logged only — loses the clue while there is no log file; (c) kept in the
   sentence, as today — breaks the rule.
3. **A left-out holds no path and no args.** Options: (a) category, reason key and sizes, counted — recommended:
   bounded, private and enough for "why"; (b) a path per item, listed — a second list of findings, and the paths of
   things the user was not offered.
4. **A failed start shows a native message and ends.** Options: (a) the native message with the core's text, the
   detail in development builds only — recommended; (b) open the window on an error screen — needs a UI without an
   engine; (c) as today, end without a word.
5. **Built after plan 012 is merged into `main`**, on this branch made again from `main` then, since both edit
   `crates/core/src/error.rs` and the two dictionaries.

## Agent's calls (to check)

1. **The draft's "Explain" and "Diagnostics" became deferred items**, and its "Logging" too, as the user asked; the
   error model of the draft is this plan, smaller: the error shapes it asked for already exist.
2. **Declared problems are a list in the descriptor**, not a type of each module: a module keeps using `Text` and
   `TextKey` as today, and the contract test does the checking.
3. **`FindingSink::left_out` is a required method.** The core's `impl FindingSink for Vec<NewFinding>`, used by tests
   that look at findings only, drops left-outs and says so in its doc comment; the testing harness and the engine keep
   them.
4. **Left-outs are event-only, like a module's failure**: a scan restored from the cache shows no left-out lines.
   Keeping them in the scan cache waits for the [diagnostics corner](../../deferred/diagnostics-screen.md).
   **Reworded 2026-11-17:** left-outs are event-only: after a restart the UI knows the restored scan only from the
   overview, so no left-out line shows until the next scan. (The restore runs during setup, before the UI subscribes,
   and its events are not replayed; modules without interests, such as the demo, do not run on a restore at all.)
5. **The rules engine reports no left-out yet.** A rule's `min-age-days` or `must-not-match` stays silent; a rule
   format for "left out because" comes with [match explanations](../../deferred/match-explanations.md).
6. **No case file in this plan** (section 5).
7. **A missing text still shows its key** in the UI: a visible gap the tests prevent, rather than a blank.
8. **No version bump**: the version stays 0.1.0 ([`../README.md`](../README.md), "Versions").

## Conclusions

See [the plan's readme](./readme.md), "Conclusions", for what was built, what changed and the agent's calls.
