# 032/02/03 — Fixtures, golden and the contract

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, fixtures

Read first: [`../00-design.md`](../00-design.md) sections 3, 4 and 9, `docs/testing.md` ("Fixture homes"),
`modules/app-caches/tests/{fixtures.rs,contract.rs,support/mod.rs}` (the shape), 02/02's module, and
`crates/testing/src/run_module.rs` (`Ports`: `apps`, `running`, `commands`, `restored`, `avoid`).

The module on fixture trees through `run_module` (the reference walk), with the installed and running apps and the
command answers as `Ports` give them; its golden report; the contract over the runs. The app's registration is made
in 04/03, not here.

## Files

- `fixtures/homes/docker-silent.toml` (new) — nothing to say: a stale socket at `.docker/run/docker.sock` (a file
  stated `kind = "socket"`), a file `VMs/Docker.raw` at no known place, a folder named
  `Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw` (a folder, not a file). Run with no app installed:
  no insight, no call.
- `fixtures/homes/docker-stopped.toml` (new) — `Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw` sparse
  of 64 GiB (`size`), and the OrbStack socket `.orbstack/run/docker.sock` stated `kind = "socket"`. Runs: Docker
  Desktop installed and closed (insight `stopped` with `maximum` and `on-disk`); OrbStack installed and open but its
  socket removed from the tree's run (a second home `docker-stopped-no-socket.toml` if a run cannot drop a file;
  insight `stopped`, no figure); the same home restored (`Ports::restored = true`, insight `restored`, no call);
  the running apps failing (`unknown`, no call). No call in any run.
- `fixtures/homes/docker-answered.toml` (new) — OrbStack's socket stated `kind = "socket"` and its disk
  `Library/Group Containers/HUAQ24HBR6.dev.orbstack/data/data.img` sparse of 8 GiB. Runs with OrbStack installed and
  open and `FakeCommandRunner` answering the read (`<app path>/Contents/MacOS/xbin/docker --context orbstack system df
  --format json`) with the design's four lines, Containers at `0B`: insight `answered`, figures `on-disk` and
  `inside` (no `maximum`: hidden), four parts, three commands; then answering with status 1 (`no-answer`), with
  `garbage` (`unreadable`), and with the disk's place avoided (`Ports::avoid` holding `Library/Group Containers`:
  the note `disk-unseen`, no disk figure).
- `fixtures/homes/docker-left.toml` (new) — `Docker.raw` as in `docker-stopped.toml`, no app installed: insight `left`
  with `maximum` and `on-disk`, no `gives-back` note, no call.
- `fixtures/golden/docker.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the insights of the answered run (titles,
  summaries, figures, parts with their command lines, notes). No finding, no left-out.
- `modules/docker/tests/fixtures.rs` and `tests/support/mod.rs` (new) — the homes through `run_module` with the
  installed apps (`installed_app("dev.kdrag0n.MacVirt", "OrbStack", "2.2.5")`, `installed_app("com.docker.docker",
  "Docker", "4.0")`), the running apps and the answers; the golden comparison. The expected command is built from the
  fake app's path, never written as an absolute path.
- `modules/docker/tests/contract.rs` — `assert_texts_used` over every run above (so every state's summary, every
  figure, part and note text is used) and `check_removals_matched` empty (the module has no removal).

## Produces

The module's fixtures, golden and contract, for 04 and 05.

## Tests (negative first)

- `tests/fixtures.rs`, `mod negative`: `nothing_is_said_or_run_without_an_installed_runtime`,
  `a_stale_socket_alone_runs_nothing`, `a_folder_at_the_disk_file_s_place_is_not_a_disk_file`,
  `a_closed_runtime_runs_nothing`, `an_open_runtime_without_its_socket_runs_nothing`, `a_restored_scan_runs_nothing`,
  `no_run_pushes_a_finding_or_a_left_out`; `mod positive`: `an_open_runtime_is_asked_once_with_its_context`,
  `the_answer_gives_four_parts_and_a_command_for_each_unused_one`, `a_left_disk_file_is_described`,
  `an_unseen_disk_place_says_how_to_see_it`, `the_golden_report_matches`.
- `tests/contract.rs` passes with the runs.

## Done when

- `cargo nextest run -p fresher-module-docker`; coverage of the crate at or above 90 %; clippy; fmt.
- One commit: `test(modules): docker's fixture homes, golden and contract`.

## Conclusions

Not written yet.
