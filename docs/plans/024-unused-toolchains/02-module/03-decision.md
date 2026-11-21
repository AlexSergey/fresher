# 024/02/03 — The decision

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules

What the module does with its matches: gather them during the walk; in `finish`, read the pointers and pins its
matches showed, resolve them per manager, decide each version by the rows of the design's section 6 and report. The
decision is a pure function on plain values; only `finish` reads files. The design:
[`../00-design.md`](../00-design.md), sections 5 to 7.

## Files

- `modules/unused-toolchains/src/gathered.rs` (new) — the matches of one scan, keyed by its cancel token (as
  `modules/local-ai-models/src/gathered.rs`): the version candidates (path, row, kind, sizes, entry facts, inside),
  the pointer matches (the `.rustup` folder with its holds; each alias file by its path), the pin folders with which
  pin names they hold.
- `modules/unused-toolchains/src/read.rs` (new) — the only code that reads: `fn pointers(row, gathered, system,
  artifacts) -> Pointers` asks `cx.system().small_file` for `settings.toml` (only when the `.rustup` match holds it),
  for each alias file matched, and for each pin of the row whose folder is not inside an artifact folder of
  `cx.system().projects()` (components, never string prefixes), then resolves them with 02/02's parsers: `Missing`
  pins are skipped; `NotAFile`, `TooLong`, `NotText`, `Unreadable` are `Unknown`; a missing or unreadable
  `settings.toml` is an `Unknown` default for rustup, a missing `alias/default` is `Nothing` for nvm. Each file is read
  once per scan (a pin of both managers, `.tool-versions`, is read once and used twice).
- `modules/unused-toolchains/src/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) enum Reason { Special, NotYours, PartialScan, DefaultUnknown, PinUnknown }` with `ALL` and `key()`
    (`unused-toolchains.left-out.<name>`);
  - `pub(crate) enum Verdict { NotClaimed, LeftOut(Reason), Offered }`;
  - `pub(crate) struct Scan { walk_complete: bool, switch_on: bool }`;
  - `pub(crate) fn decide(candidate: &Candidate, row: &Row, version: Option<&Version>, newest: bool, pointers:
    &Pointers, scan: Scan) -> Verdict`: the twelve rows in order; `version` is `Version { index, name }`, the
    version's index among the row's installed versions and its parsed name (`None` off shape); a row whose switch is
    off is decided with empty pointers (rule 3 returns not claimed, no file of the row is read);
  - `pub(crate) fn last_seen(inside: &Inside) -> Option<Timestamp>`: the newer of `newest_change` and
    `newest_file_access`.
- `modules/unused-toolchains/src/report.rs` (new) — the text keys made once with the module: `Report::tell(row,
  candidate, verdict, out)` pushes a `LeftOutItem { category, reason, sizes }` or a `NewFinding`: identity `<row
  id>/<folder name>`, `Subject::Path` and `Removal::MoveToTrash` of the folder, the match's sizes, title
  `unused-toolchains.title.<row>` with `name`, reason `unused-toolchains.reason.<row>` with `date`, `size` and
  `command` (`rustup toolchain install <folder name>`, `nvm install <x.y.z>`).
- `modules/unused-toolchains/src/lib.rs` — `on_match` gathers by the interest's kind; `finish`: nothing when
  `cx.system().projects()` is `None`; per row, the installed versions parsed and the newest found (a linked version
  folder counts as installed, as nvm counts it); the pointers read (`read.rs`), none for a row whose switch is off
  (`settings::is_on`: no file of it is read, and decide returns not claimed); then every candidate decided and
  reported in path order, checking `cx.cancel()` between them (`ModuleError::Cancelled`).
- `modules/unused-toolchains/src/tests.rs` (new) — the module run on matches built by hand (as
  `modules/local-ai-models/src/tests.rs`), with a `FakeSmallFiles` (a map of path to `SmallFile`, counting reads) and
  `ProjectFinder`'s projects set by hand.

## Produces

The module's answers, for 02/04 and 03.

## Checks

- `decide.rs`, negatives first, one test per row, keep before offer: `a_link_or_a_file_is_not_claimed`,
  `a_name_off_the_shape_is_not_claimed`, `a_node_version_with_the_switch_off_is_not_claimed`,
  `a_moving_channel_and_the_newest_node_are_not_claimed`, `a_folder_read_in_part_is_not_claimed`,
  `a_version_the_default_an_override_an_alias_or_a_pin_keeps_is_not_claimed`, `a_socket_inside_keeps_it_special`,
  `an_entry_not_yours_keeps_it`, `a_walk_not_complete_keeps_it`, `an_unknown_default_keeps_it`,
  `an_unknown_pin_keeps_it`; then `an_old_toolchain_nothing_points_at_is_offered_low`,
  `a_dated_nightly_nothing_points_at_is_offered`, `an_old_node_version_is_offered_medium`,
  `the_date_is_the_newer_of_change_and_access`. Properties (proptest): `the_verdict_never_depends_on_the_order_of_matches`,
  `a_kept_version_is_never_offered_or_left_out_whatever_else_is_unknown`, `the_default_and_the_newest_are_never_offered`,
  `the_switch_off_never_claims_a_node_version`.
- `tests.rs`: `mod negative` `nothing_is_claimed_before_the_projects_are_known`,
  `a_pin_inside_node_modules_is_not_read`, `a_linked_pin_blocks_its_manager`, `the_switch_off_reads_no_nvm_file`,
  `nothing_left_out_is_also_offered`, `a_cancelled_scan_stops_between_candidates`; `mod positive`
  `each_file_is_read_once_per_scan`, `two_orders_of_matches_give_the_same_answers`,
  `a_finding_names_the_version_the_date_and_the_command`, `a_finding_moves_the_version_folder_to_the_trash`.
- `cargo nextest run -p fresher-module-unused-toolchains`; clippy; fmt.

## Conclusions

Built as written, with these calls. The decision takes `Version { index, name }` rather than a bare name, because
the pointers' answers are by index. With the Node switch off the row is not skipped: its candidates are decided with
empty pointers and `switch_on: false`, so no nvm file is read and the rule itself leaves them alone. The installed
versions of a row are every candidate of its place, links included: a pin or default naming a linked toolchain keeps
the link (never claimed anyway), and a linked version folder counts as installed when the newest Node version is
found, as nvm's own view does (cost: none seen; a plain file named like a newer version does not, fixed in the final
fix wave with a test that failed first). Each file is read once per scan (a cache by folder and name). Pins inside a
project's artifact folder are not read, by path components. The tests moved from `lib.rs` to `tests.rs`, where they
first did not compile in (102 passed until the shadowing was fixed, then 118). Mutations by hand: dropping the artifact
skip and dropping the read cache each failed a named test.
