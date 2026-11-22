# 032/02/02 — The answer, the decision and the insight

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules

Read first: [`../00-design.md`](../00-design.md) sections 2 to 5 (the tables there are the specification),
`docs/architecture/modules.md` ("The system's facts", "What a module sees of the walk"), 02/01's crate, and as shapes
`modules/local-ai-models/src/{gathered.rs,tests.rs}` and `modules/app-caches/src/decide.rs`.

**Safety (rule 1, decision 1):** the module's only call to a program is the read below. It never runs a runtime that
is not up, never in a restored scan, and nothing it reports is a finding.

## Files

- `modules/docker/src/gathered.rs` (new) — the matches of one scan, keyed by its cancel token, as
  `modules/local-ai-models/src/gathered.rs`: per row, whether its socket was seen (only an `EntryKind::Other` match at
  the socket's interest) and its disk file's sizes (only an `EntryKind::File` match at the disk's interest; a link or a
  folder there is not seen).
- `modules/docker/src/answer.rs` (new) — `parse(stdout: &str) -> Result<Usage, Unreadable>`: the design's section 3,
  "The answer's format", exactly. `Usage { build_cache: Kind, images: Kind, containers: Kind, volumes: Kind }`,
  `Kind { size: ByteSize, unused: ByteSize, count: u64, active: u64 }` (`unused` from `Reclaimable`; `unused_count()` =
  `count − active`). Each non-empty line a JSON object (`serde_json`, unknown fields allowed) with string fields
  `Type`, `TotalCount`, `Active`, `Size`, `Reclaimable`; the four types each exactly once, a line of another type ignored whatever its other fields. A size
  `<decimal><unit>` with units `B`, `kB`, `KB`, `MB`, `GB`, `TB`, `PB` (powers of 1000), rounded to whole bytes, an
  optional ` (N%)` after `Reclaimable` dropped. Unreadable (`Unreadable` carries a short reason for the log, never
  shown): empty output, an output over 64 KiB, a line that is not such an object, a type missing or twice, a number or size that does not read,
  `Active > TotalCount`, `Reclaimable > Size`.
- `modules/docker/src/decide.rs` (new) — on plain values, never a port:
  - `enum State { Left, Restored, Unknown, Stopped, NoAnswer, Unreadable, Answered(Usage) }` with `key()` (the
    summary's key: `docker.summary.left`, …);
  - `enum Verdict { Silent, Describe(State), Ask }`;
  - `fn decide(installed: Result<Option<&InstalledApp>, NoSystem>, running: impl FnOnce() -> Result<bool, NoSystem>,
    socket_seen: bool, disk: Disk, restored: bool) -> Verdict` with `enum Disk { Seen(Sizes), Absent, Unseen }`: the
    design's section 3 rows 1 to 7 in order; `running` called only when a row reaches row 5;
  - `fn answered(result: Result<CommandOutput, CommandRunError>) -> Result<State, Cancelled>`: `Cancelled` →
    `Err(Cancelled)`; `NotInstalled`, `Failed` or a status other than 0 → `NoAnswer`; status 0 → `parse`, `Answered`
    or `Unreadable`.
- `modules/docker/src/report.rs` (new) — `fn insight(runtime: &Runtime, state: &State, disk: &Disk) ->
  Result<Insight, InsightError>`: the design's section 4 table exactly — identity the row's id; title `docker.title`
  with `runtime` (`TextArg::Text`, the row's `name`); summary `docker.summary.<state>` with `runtime` and, when
  answered, `inside` and `unused` (`TextArg::Size`: the sums of the four `size` and of the four `unused`); the figures
  in order `maximum`
  (disk seen and `Maximum::Shown`: its logical size), `on-disk` (disk seen: its allocated size), `inside` (answered);
  the four parts when answered, in order build cache (`DangerLevel::Safe`), images (`Low`), containers (`Medium`),
  volumes (`High`), each `size` = `unused`, `detail` `docker.part.<part>.detail` with `count` (`TextArg::Count`, the
  unused count; the build cache's detail has no count) or `.none` when `unused` is 0, `effect`, and a command when
  `unused` > 0: `ShownCommand::new(docker, ["--context", <context>, …])` with `builder prune --all`, `image prune
  --all`, `container prune`, `volume prune`; the notes in order `docker.note.disk-unseen` (disk `Unseen`),
  `docker.runtime.<id>.gives-back` (every state but `Left`), `docker.note.never-runs` (a part has a command).
- `modules/docker/src/lib.rs` — `on_match` gathers; `finish`, for each row in the table's order: checks `cx.cancel()`
  (`ModuleError::Cancelled`); asks `cx.system().installed_apps()` once per scan (only when a row needs it) and finds the
  row's app by bundle id; an app not found counts as not installed only when `installed_apps_complete()` is `Ok`,
  otherwise the row is silent (row 1: a partial listing never gives a false `left`; the cost is that a truly left-behind
  disk file is not described then); decides with `cx.is_restored()`, the gathered socket and disk (`Disk::Unseen` when
  `cx.place_seen(disk interest)` is `Unread` or `NotScanned`, `Absent` when nothing was matched and the place was read
  or is absent); on `Ask` runs `cx.commands().run(<app path>/<cli> as a string, ["--context", <context>, "system",
  "df", "--format", "json"], cx.cancel())`, a program path that is not UTF-8 counting as `NoAnswer`; on `Describe` or
  after the answer, `report::insight(..)` and, when it is `Ok`, `out.describe(insight)`. An `InsightError` can only come from the module's own table
  (its contexts are refused by `Runtimes::parse` when they would need quoting, and its counts are below the core's
  maximums): `finish` skips that row's insight and goes on, never panics, and a test pins that the shipped table
  builds every state without an error. The running apps are asked at most once per scan (memoised in a `OnceCell`) and
  only when a row reaches row 5.
- `modules/docker/src/tests.rs` (new) — the module on hand-built matches with `FakeInstalledApps`,
  `FakeRunningApps` and `FakeCommandRunner` (as `modules/local-ai-models/src/tests.rs` builds its context).
  `modules/docker/src/decide/tests.rs`, `modules/docker/src/answer/tests.rs` — the tests of those files.
- `modules/docker/Cargo.toml` — `serde_json` (dependency), `proptest` (dev-dependency); their used-by rows in
  `docs/dependencies.md`.

## Produces

The module's answers, for 02/03 (fixtures, golden), 04 and 05.

## Tests (negative first)

- `answer/tests.rs`, `mod negative`: `an_empty_answer_is_unreadable`, `a_line_that_is_not_json_is_unreadable`,
  `a_missing_kind_is_unreadable`, `a_kind_twice_is_unreadable`, `a_size_without_a_unit_is_unreadable`,
  `an_unknown_unit_is_unreadable` (`GiB`, `XB`), `more_active_than_total_is_unreadable`,
  `more_unused_than_the_size_is_unreadable`; `mod positive`: `the_documented_answer_reads` (the four lines of the
  design), `units_are_powers_of_1000`, `a_percentage_after_reclaimable_is_dropped`, `an_unknown_kind_is_ignored`,
  `unknown_fields_are_ignored`; property (proptest): `a_size_printed_in_docker_s_units_reads_back_within_rounding`.
- `decide/tests.rs`, `mod negative` (rows that ask nothing, keep before ask):
  `nothing_is_said_when_the_installed_apps_cannot_be_listed`, `nothing_is_said_when_the_app_and_its_disk_are_absent`,
  `a_restored_scan_never_asks`, `it_never_asks_when_the_running_apps_cannot_be_listed`,
  `a_closed_app_is_never_asked`, `an_open_app_without_its_socket_is_never_asked`, `a_stale_socket_alone_is_never_asked`,
  `a_failed_run_is_no_answer`, `a_non_zero_status_is_no_answer`, `a_cancelled_run_is_cancelled`; `mod positive`:
  `a_left_disk_file_is_described_as_left`, `an_open_app_with_its_socket_is_asked`, `a_readable_answer_is_answered`;
  property (proptest): `a_row_stopped_before_row_5_never_asks_the_running_apps`.
- `tests.rs`, `mod negative`: `no_runtime_installed_runs_nothing_and_says_nothing`,
  `a_stopped_runtime_runs_nothing` (`FakeCommandRunner::calls()` empty), `a_restored_scan_runs_nothing`,
  `nothing_is_ever_pushed_as_a_finding` (every run: zero findings, zero left-outs), `a_cancelled_scan_stops_between_rows`,
  `a_link_at_the_disk_file_s_place_is_not_seen`; `mod positive`: `the_only_call_is_the_read_of_its_runtime` (the exact
  program `<app path>/Contents/MacOS/xbin/docker` and arguments), `an_answered_runtime_has_four_parts_with_their_commands`
  (lines `docker --context orbstack builder prune --all`, …), `a_part_with_nothing_unused_has_no_command`,
  `the_maximum_is_shown_only_for_rows_that_show_it`, `an_unread_disk_place_adds_the_note`,
  `two_runtimes_give_two_insights_in_table_order`, `the_shipped_table_builds_every_state_without_an_error` (each row ×
  each `State`, with and without a seen disk).

## Done when

- `cargo nextest run -p fresher-module-docker`; clippy; fmt; `npm run check:machete`; the crate's coverage at or above
  90 % (`cargo llvm-cov` as `docs/testing.md` says).
- One commit: `feat(modules): docker reads the runtime's answer and describes it`.

## Conclusions

Not written yet.
