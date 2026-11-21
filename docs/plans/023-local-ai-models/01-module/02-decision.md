# 023/01/02 — The names and the decision

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules

What the module does with each match: read the model's name from its folder, decide by the rows of the design's
section 4 — offered, left out with its reason, or not claimed — and report. Pure decisions on plain values, so every
row is a unit test; the module only gathers and reports. The design: [`../00-design.md`](../00-design.md), sections 3
to 5.

## Files

- `modules/local-ai-models/src/name.rs` (new) — `pub(crate) struct ModelName { kind: Option<String>, title: String }`
  and `pub(crate) fn read(shape: Shape, path: &Path) -> Option<ModelName>`: `HubRepo` splits the last component on
  `--` into `<kind>--<owner>--<name>` (title `owner/name`) or `<kind>--<name>` (title `name`), every part non-empty;
  `PublisherModel` takes the last two components (title `publisher/model`), neither starting with `.`; any other
  shape, or a name that is not valid UTF-8, is `None`. Components, never string slicing of the path.
- `modules/local-ai-models/src/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) enum Reason { NotYours, Special, AppOpen, NoSystem }` with `ALL` and `key()`
    (`local-ai-models.left-out.<name>`);
  - `pub(crate) enum Verdict { NotClaimed, LeftOut(Reason), Offered { category: CategoryId, name: ModelName } }`;
  - `pub(crate) struct NoSystem;`
  - `pub(crate) fn decide(candidate: &Candidate, row: &Store, running: Result<&RunningSet, NoSystem>) -> Verdict`:
    the nine rows in order; `running` read only for a row with an app; the inside's lock names are not read.
  - `pub(crate) fn last_seen(inside: &Inside) -> Option<Timestamp>`: the newer of `newest_change` and
    `newest_file_access`.
- `modules/local-ai-models/src/report.rs` (new) — the text keys made once with the module (as
  `modules/dev-caches`): `Report::tell(row, candidate, verdict, out)` pushes a `LeftOutItem { category, reason, sizes
  }` (the category of the name's kind when it has one, else the row's) or a `NewFinding`: identity `<row id>/<the
  shape's folder names>` (LM Studio `<row id>/<publisher>/<model>`: two publishers can ship the same model name), `Subject::Path` and `Removal::MoveToTrash` of the folder, the match's sizes, title `local-ai-models.title`
  with `name`, reason `local-ai-models.reason.<tool>` with `date` (`last_seen`) and `size` (the logical size).
- `modules/local-ai-models/src/gathered.rs` (new) — the candidates the module matched in one scan (path, kind, row,
  sizes, entry facts, holds, inside), keyed by its cancel token, as `modules/dev-caches/src/gathered.rs`.
- `modules/local-ai-models/src/lib.rs` — `on_match` gathers; `finish` asks `cx.system().running_apps()` once, only
  when a gathered candidate's row names an app (as `dev-caches` waits for `finish` for the running apps), then decides
  and reports every candidate in path order, checking `cx.cancel()` between them (`ModuleError::Cancelled`).
- `modules/local-ai-models/src/tests.rs` (new) — the module run on matches built by hand (as
  `modules/dev-caches/src/tests.rs` builds them) with `FakeRunningApps`.
- `modules/local-ai-models/Cargo.toml` — `proptest` as a dev-dependency.

## Produces

The module's answers, for 01/03 (fixtures and golden) and 02.

## Checks

- `name.rs`, negatives first: `mod negative` `an_empty_part_is_not_a_repo` (`models--`, `models----x`,
  `--owner--name`), `three_separators_are_not_a_repo` (`models--a--b--c`), `the_hub_s_own_entries_are_not_repos`
  (`.locks`, `version.txt`), `a_hidden_publisher_or_model_is_not_a_model`, `a_name_not_in_utf8_is_not_a_model`;
  `mod positive` `a_repo_with_an_owner_reads_owner_and_name`, `a_repo_without_an_owner_reads_its_name`,
  `a_dataset_reads_its_kind`, `an_lm_studio_folder_reads_publisher_and_model`.
- `decide.rs`, negatives first, one test per row, keep before offer:
  `a_file_or_a_link_is_not_claimed`, `a_space_is_not_claimed` (kind not in the row),
  `a_repo_without_blobs_or_snapshots_is_not_claimed`, `a_folder_read_in_part_is_not_claimed`,
  `a_socket_inside_keeps_it_special`, `an_entry_not_yours_keeps_it`, `lm_studio_s_model_is_kept_when_the_apps_cannot_be_listed`,
  `lm_studio_s_model_is_kept_while_it_runs`; then `a_hugging_face_model_is_offered_in_the_models_category`,
  `a_dataset_is_offered_in_the_datasets_category`, `an_lm_studio_model_is_offered_at_medium`,
  `the_date_is_the_newer_of_change_and_access`. Properties (proptest): `a_row_without_an_app_never_asks_the_running_apps`,
  `a_restored_context_decides_the_same`.
- `tests.rs`: `mod negative` `ollama_s_store_is_never_claimed` (no interest matches it: the module's interests name
  no `.ollama` place), `nothing_left_out_is_also_offered`, `a_cancelled_scan_stops_between_candidates`; `mod positive` `two_orders_of_matches_give_the_same_answers`,
  `a_finding_names_the_model_the_date_and_the_size`, `a_finding_moves_the_model_folder_to_the_trash`.
- `cargo nextest run -p fresher-module-local-ai-models`; clippy; fmt; `npm run check:machete`.

## Conclusions

Built as written, with two calls: the decision's verdict holds the category as `&'static str` (the decision stays
pure, on plain values), and an LM Studio finding's identity is `<row id>/<publisher>/<model>`, not the folder name
alone, so two publishers shipping one model name never collide; the design's section 5 says so. Rows 1 to 9 are one
unit test each, keep before offer; the verdict never depends on the order of the matches. A review round added the
cancel-between-candidates test and the row-pick property.
