# 019/03/01 — Spotlight in the sandbox and the case files

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. First the sandbox learns what Spotlight answers (a part of
[the deferred sandbox world](../../../deferred/sandbox-world.md)), then the two files. Two commits.

## Files

Spotlight in the sandbox:

- `crates/sandbox/src/case.rs` — a case may declare `[[case.elsewhere]]` (`name`, `bundle-id`: an app Spotlight knows
  outside the apps folders; the bundle id checked as `[[case.app]]`'s) and `spotlight = "off"` (the only value);
  refused: an unknown field, a bad bundle id, an empty name, another `spotlight` value.
- `crates/sandbox/src/world.rs` — `World` gains `spotlight_on: bool` (on unless a case says `off`) and `elsewhere:
  Vec<(String, BundleId)>`, joined from every case as the running apps are; `world.toml` writes `spotlight = "off"` and
  `[[elsewhere]]` only when there are any (a world without them is written as before); `World::app_search(sandbox)
  -> DeclaredSearch`: the index as declared, each elsewhere app answered at `<sandbox>/Volumes/Elsewhere/<name>.app`
  (never written and never read: Spotlight only answers it).
- `crates/sandbox/src/expected.rs` — `EXPECTED.md` names the apps Spotlight knows elsewhere and says when Spotlight is
  off.
- `app/tauri/src/sandbox.rs` — `AppSearch::Declared(world.app_search(&sandbox))` instead of
  `DeclaredSearch::listed()`; the banner's simulated parts name Spotlight when the world declares any of it.
- `app/tauri/tests/sandboxes/harness.rs` — the autotest's ports take the same `DeclaredSearch` from the world.
- `docs/testing.md` — the case-file vocabulary: `[[case.elsewhere]]` and `spotlight = "off"`, and that a file with
  Spotlight off is a world of its own. `docs/deferred/sandbox-world.md` — "Partly taken by plan 019": Spotlight's
  answers; its table row in `docs/deferred/README.md` updated.

The case files:

- `fixtures/sandboxes/app-leftovers.toml` (new) — `feature = "app-leftovers"`, `module = "app-leftovers"`; a head
  comment saying what the file covers, that `Library/Containers` is not opened in sandbox mode (so no container is
  offered here; the fixtures and the core e2e offer one), and why `app-leftovers-spotlight-off.toml` exists; the
  twelve negative cases, then the four positive ones, of the design's section 6, "Sandbox", with these exact names:
  - negative: "a name that is not an app's id is never claimed", "Apple's own files are never claimed", "a tool's
    files with no app behind them are not claimed", "an installed app's files are not claimed", "an open app outside
    the apps folders keeps its files", "files changed lately are kept", "an app Spotlight knows elsewhere keeps its
    files", "a link at a place is never claimed", "a leftover holding a socket is kept", "a leftover holding the
    system's file is kept", "a group container is never claimed", "a container is not opened";
  - positive: "a removed app's caches, logs and saved windows go", "a removed app's settings go, at Medium", "a
    removed app's web data and cookies go with its settings", "a removed app's support folder goes, at High".
  Each case its own ids and paths (no path written twice in the file); **every entry of a leftover states `age` and
  `changed`** (`"40d"`, `"5d"` for a recent one), its folder a `[[case.dir]]` with both and its files directly inside
  (the status change cannot be set back on disk; a whole folder's age is the newest inside it); positives' files
  `fill = "data"`; `[[case.left-out]]` for each kept reason with its category and items; `[case.map]` `your-folders`
  at `home/Library/Application Support/Notes` and the support folder positive in its id's line.
- `fixtures/sandboxes/app-leftovers-spotlight-off.toml` (new) — `feature = "app-leftovers-spotlight-off"`, `module =
  "app-leftovers"`; a head comment (Spotlight off is a world of its own); negative "nothing is offered while Spotlight
  cannot be asked" (`com.example.Unsearched`: a saved state and caches, 40 days, `spotlight = "off"`; left out
  `spotlight-unread` 2 in `app-leftovers.caches`). No path any other case file writes (`all` joins them).
- `fixtures/golden/sandbox-app-leftovers.json`, `sandbox-app-leftovers-spotlight-off.json` (new) — `UPDATE_GOLDEN=1`,
  reviewed: the findings with their app, the left-out counts, the map, the cleanup's outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — both files in `case_files!`, sorted by file name.
- `docs/testing.md` — the two case files in the list of case files.

What the code does for each case is written down when the file is made; a case that differs from the design is fixed
in the module, not in the case file.

## Produces

The module's sandboxes and the sandbox's Spotlight answers, for the runs by hand (04/02) and every later module that
asks Spotlight.

## Checks

- `crates/sandbox`, negatives first: each refusal of `[[case.elsewhere]]` and `spotlight`; a world without them
  written byte for byte as before; then a world with both read back equal, and `app_search` answering the declared
  apps only.
- `npm run sandbox:test` passes both files; it fails, once each by hand, when a positive's danger, a left-out's
  `items`, and a positive's proof (its saved state removed) are changed on purpose (written in the conclusions).
- `npm run sandbox -- app-leftovers` (and the other) writes `sandbox/<feature>/` with `world.toml` (the apps, the
  running and elsewhere apps, the stated entries) and an `EXPECTED.md` that names them; `npm run sandbox -- all`
  writes them beside the other case files without a clash (Spotlight off then holds for every module's look by hand:
  said in `EXPECTED.md`, [known issue](../../../known-issues/repo-sandbox-all-worlds-interact.md)).
- `cargo nextest run -p fresher-sandbox -p fresher-app`; `npm run lint`.

## Conclusions

Built (feb5ea4, b066af5): in the sandbox, `[[case.elsewhere]]` (`name`, `bundle-id`) and `spotlight = "off"` per case
(`CaseError::{ElsewhereBundleId, ElsewhereName}`), `World::{spotlight_on, elsewhere_apps, declares_spotlight}`, the
lines of `world.toml` and `EXPECTED.md` written only when a case declares them, `app/tauri/src/sandbox.rs` composing
`AppSearch::Declared` for both the app and the autotest (F20). The case files `app-leftovers.toml` (twelve negatives, four
positives) and `app-leftovers-spotlight-off.toml`, their goldens (12 findings; left out `running` 2, `used-recently` 4,
`installed-elsewhere` 2, `special` 1, `not-yours` 1; the spotlight-off golden has only `spotlight-unread` 2); the generator
checked by hand for each file and for `all`. `docs/testing.md` and `docs/deferred/sandbox-world.md` (Spotlight's answers
taken) say it.

Deviations and calls:

- `World` exposes `spotlight_on()` and `elsewhere_apps(sandbox)`; the `DeclaredSearch` is built in `app/tauri`. The brief's
  `World::app_search` is not possible: the crate guard allows `fresher-sandbox` only core, macos and testing.
- The banner names a simulated Spotlight: `Build::Sandbox.simulated_spotlight`, regenerated bindings, two UI keys, the
  banner tests, story and e2e. The default mock has it `false`, so no screenshot changed.
- Spotlight off in the `all` sandbox keeps the positives of every module that asks Spotlight; recorded in the known issue
  [`repo-sandbox-all-worlds-interact.md`](../../../known-issues/repo-sandbox-all-worlds-interact.md).
- A sandbox `[[case.app]]` cannot declare helpers; a declared helper is covered by the fixtures of 02/03.

Tests: changing an expectation on purpose fails the autotest, once per kind (a danger, a left-out's items, a proof
removed from a positive, the spotlight-off items, the `spotlight` line removed), each restored. `cargo nextest run -p
fresher-sandbox -p fresher-app`: 246 passed, `every_registered_module_has_a_case_file` included; `npm run sandbox:test`
42 passed; vitest 1009; tsc and eslint clean; the banner e2e passes in both engines.

Parked minors: a new sandbox test sits between a doc comment and its test.
