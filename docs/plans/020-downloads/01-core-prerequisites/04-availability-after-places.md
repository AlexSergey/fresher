# 020/01/04 — A module that needs a guarded folder runs in the scan that tries it

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** engine, docs

`session::run` decides each module's availability (`decide_access`) before it decides the guarded places
(`decide_places`), where an opted-in place is tried for the first time. So in the first scan after the user switches
Downloads on, the folder is walked but `downloads` is `Unavailable` (the probe still said `NotAsked`) and offers
nothing until a later scan after the window's focus refreshed the access. No module needed a guarded folder before
this plan. The design: [`../00-design.md`](../00-design.md), section 1.4 (agent's call 8).

## Files

- `crates/engine/src/coverage.rs` — `Entry` gains `pub(crate) tried: Vec<(Capability, Access)>`: every answer of
  `try_place` in `decide`, once per capability (the vector `decide` keeps today becomes the field). `before_restore`
  leaves it empty.
- `crates/engine/src/access.rs` — `AccessView::learn(&self, tried: &[(Capability, Access)]) -> bool`: for each pair,
  the system state of that capability takes the answer (added when the probe did not list it); `true` when a state
  changed. Documented: the probe may remember its tries itself (`MacAccess` does), the engine does not rely on it.
- `crates/engine/src/session.rs` — in `run`: the tool checks first (as `decide_access` does today, with its
  `AccessChanged`), then `decide_places` with that status, then `shared.access.learn(&entry.tried)` and, when it
  changed something, `Event::AccessChanged { status }`; only then `unavailable` and `limited` with the learnt status,
  and the unavailable modules' outcomes. `decide_access` is split accordingly (`check_tools`, then `decide_modules`
  after the places); the order of the events a scan publishes is written in the function's doc: `ScanStarted`, maybe
  `AccessChanged` (tools), maybe `AccessChanged` (places), the unavailable modules' `ModuleFinished`, then the walk.
- `docs/architecture/permissions.md` — "When macOS may prompt": a module whose category needs a guarded folder runs
  in the scan that first tries the folder (the engine learns the answer before it decides the modules).

## Produces

A module with `Need::required(Capability::Folder(_))` that runs in the first scan after an opt-in, for 03 and 04/02
(flow 2).

## Checks

- `crates/engine/src/coverage.rs` and `access.rs` tests, negatives first:
  - `mod negative`: `a_place_that_is_not_opted_in_teaches_nothing` (no try, `learn` of an empty list answers false,
    no `AccessChanged`), `a_restore_learns_nothing_even_opted_in`, `a_denied_try_keeps_the_module_unavailable` (a
    module needing Downloads, opted in, `try_place` answering `Denied`: `ModuleOutcome::Unavailable { missing:
    [Downloads] }`, the coverage naming Downloads not entered);
  - `mod positive`: `a_module_needing_an_opted_in_folder_runs_in_the_scan_that_first_tries_it` (`FakeAccess` with
    Downloads `NotAsked`, guarded at the fixture home's `Downloads`, answering `Granted`; the switch on: the module's
    `on_match` is called for an entry inside, its outcome `Completed`, and one `AccessChanged` with Downloads
    `Granted` comes before its `ModuleFinished`), `learn_records_an_answer_the_probe_did_not_list`,
    `a_second_scan_tries_nothing_again` (the learnt `Granted` allows the place without a try).
- The existing engine tests pass unchanged (`coverage.rs`, `access.rs`, `left_out_tests.rs`,
  `ignore_module_tests.rs`): the events of a scan with nothing tried are the same as before.
- `cargo nextest run -p fresher-engine -p fresher-e2e`; clippy; fmt.

## Conclusions

Not written yet.
