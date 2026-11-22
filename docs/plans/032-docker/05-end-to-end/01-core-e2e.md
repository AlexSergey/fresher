# 032/05/01 — The core e2e

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** e2e

Read first: [`../00-design.md`](../00-design.md) sections 1, 3, 6 and 9, `docs/testing.md` ("Kinds of tests", the core
e2e), and as the shape to follow `crates/e2e/tests/app_caches.rs` with `crates/e2e/tests/app_caches_home/mod.rs` and
`crates/e2e/tests/support/home_engine.rs` (an engine over a fixture home with fakes: installed and running apps, the
command runner, the cleaner's fakes).

The engine end to end with `docker` registered beside the other modules: the user-facing guarantees of decision 1.

## Files

- `crates/e2e/tests/docker.rs` and `crates/e2e/tests/docker_home/mod.rs` (new) — a fixture home holding OrbStack's
  socket (stated `kind = "socket"`) and its disk file (sparse, `written`), Docker Desktop's stale socket, and a real
  cache another module offers (for example a `dev-caches` npm cache) so a cleanup has something to do; the engine over
  it with `docker` and that other module registered, OrbStack installed (`FakeInstalledApps`) and running
  (`FakeRunningApps`), and a `FakeCommandRunner` answering exactly the read with the design's four lines.
- `crates/e2e/Cargo.toml` — `fresher-module-docker` as a dev-dependency (as the other modules).

## Tests (negative first)

- `mod negative`:
  - `nothing_of_docker_s_is_a_finding_a_total_or_a_category` — after the scan: no finding of module `docker`, the
    overview's `totals` hold no `docker.` category, `found` equals the sum of the other module's findings.
  - `a_cleanup_of_everything_holds_nothing_of_docker_s` — `prepare_cleanup` with every finding and every category:
    the plan's items are the other module's only; after the confirmed cleanup, OrbStack's disk file and socket are
    untouched (sizes and kind), and `FakeCommandRunner::calls()` is still exactly the one read.
  - `a_closed_runtime_runs_nothing` — the same home with OrbStack not running: `calls()` empty; the insight's summary
    key is `docker.summary.stopped`.
  - `a_restored_scan_runs_nothing` — scan, then a new engine over the same data folder restoring at launch: the
    restore's `ModuleFinished` for `docker` carries the `restored` insight and `calls()` of the new engine's runner is
    empty.
  - `a_cancelled_read_ends_the_module_cancelled` — the runner answers the read with `Cancelled`: `docker`'s outcome is
    `Cancelled`, the scan goes on, the other module completes.
- `mod positive`:
  - `the_only_call_is_the_read_with_the_runtime_s_context` — `calls()` is exactly `["<app path>/Contents/MacOS/xbin/
    docker --context orbstack system df --format json"]`.
  - `the_module_finished_event_carries_the_insight` — the `orbstack` insight: two figures (`on-disk`, `inside`:
    OrbStack hides the maximum), four parts in order build cache, images, containers, volumes,
    with their danger levels, the command lines `docker --context orbstack builder prune --all`, `… image prune --all`,
    `… volume prune` and none for containers (0 B), and the notes `gives-back` and `never-runs`.
  - `docker_desktop_s_stale_socket_adds_no_insight` — exactly one insight (OrbStack's).

## Done when

- `cargo nextest run -p fresher-e2e --test docker`; clippy; fmt.
- One commit: `test(e2e): docker describes and never offers, end to end`.

## Conclusions

Not written yet.
