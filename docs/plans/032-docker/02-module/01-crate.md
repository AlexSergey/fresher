# 032/02/01 — The crate, the table and the texts

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, docs

Read first: `CLAUDE.md`, `docs/rules.md`, [`../00-design.md`](../00-design.md) sections 2 and 5,
`docs/architecture/modules.md`, and as the shape to follow `modules/app-caches` (`Cargo.toml`, `src/places.rs`,
`src/lib.rs`, `tests/contract.rs`). 01/01 and 01/02 are built (`Insight`, `FindingSink::describe`,
`ModuleRun.insights`).

The module's frame: its descriptor, its table and its interests, every text. Nothing is decided yet: `on_match` and
`finish` return `Ok(())` until 02/02.

## Files

- `modules/docker/Cargo.toml` (new) — `fresher-module-docker`, description "The Docker module of Fresher: what Docker
  keeps on the Mac and the commands that free it; it removes nothing."; the workspace's `version`, `edition`,
  `rust-version`, `license`, `publish`; `[lints] workspace = true`; dependencies `fresher-core`, `serde`, `thiserror`,
  `toml`; dev-dependency `fresher-testing` (`serde_json` and `proptest` in 02/02, when first used). The workspace's
  members are the glob `modules/*`: no root edit.
- `modules/docker/runtimes.toml` (new) — the two rows of the design's section 2, exactly, with a head comment on each
  field: `id` (kebab case, also the insight's identity), `name` (the runtime's own product name, shown as the card's
  title through `docker.title`), `app` (the bundle id: whether it is installed and running is
  asked by it), `cli` (the runtime's own Docker CLI, relative to its app bundle), `context` (the Docker context the
  runtime creates; every command, read or shown, names it), `socket` and `disk` (a known folder and a plain relative
  path), `maximum` (`shown` when the disk file's length is the size limit the user set, `hidden` otherwise).
- `modules/docker/src/runtimes.rs` (new) — `Runtimes::shipped()` (`include_str!("../runtimes.toml")`) and
  `Runtimes::parse(text) -> Result<Runtimes, RuntimesError>`, read with `deny_unknown_fields` and kebab-case names;
  a `Runtime` (`id: String`, `name: String`, `app: BundleId`, `cli: PathBuf`, `context: String`, `socket: Place`, `disk: Place`,
  `maximum: Maximum`) with `Place { folder: KnownFolder, path: PathBuf }` and `enum Maximum { Shown, Hidden }`.
  Refusals, one variant each: `Format` (not TOML, an unknown field), `Empty` (no row), `BadId` (not kebab case),
  `DuplicateId`, `BadName` (empty, longer than 40 characters, or a control character), `BadApp` (not a bundle id), `BadCli` (empty, absolute, or a component that is not a plain name: no
  `.`, `..`, `*`), `BadContext` (empty or not lowercase ASCII letters, digits and `-`), `BadPlace` (a path empty,
  absolute, holding `*`, `.` or `..`, or a known folder that is a walked place: `user-temp`, `shared-temp`),
  `DuplicatePlace` (one place named twice, socket or disk, across rows). `Runtimes::interests()` builds, in the table's
  order, two `Interest`s per row: `Pattern::Under(socket.folder, socket.path)` then `Pattern::Under(disk.folder,
  disk.path)`, each `Claim::Item`; `Runtimes::of_interest(index) -> Option<(&Runtime, Watched)>` with `enum Watched {
  Socket, Disk }` (index `2 × row` is the socket, `2 × row + 1` the disk).
- `modules/docker/src/lib.rs` (new) — `#![forbid(unsafe_code)]`; a module doc saying what it describes and what it
  never does (it removes nothing and offers nothing; it never starts a runtime; it runs only the runtime's own CLI with
  `--context <context> system df --format json`, only while its app runs and its socket is present, never in a
  restored scan); `DockerError` (`Id`, `Descriptor`, `Interest`, `Runtimes`), `RuntimesError` re-exported;
  `DockerModule::new() -> Result<Self, DockerError>`: `ModuleId` `docker`, `ModuleGroup::Development`, no category,
  no setting, the dictionary, no problems; `interests()` from `Runtimes`; `on_match` and `finish` return `Ok(())`.
- `modules/docker/locales/en.json` (new) — every text of the design's section 5, keys exactly as listed there:
  `docker.enabled.label`, `.description`; `docker.title` ("{runtime}"); `docker.runtime.<id>.gives-back` for both
  rows; the seven
  `docker.summary.<state>` (`answered`, `stopped`, `restored`, `unknown`, `no-answer`, `unreadable`, `left`); the three
  `docker.figure.<name>` (`maximum`, `on-disk`, `inside`); for each part (`build-cache`, `images`, `containers`,
  `volumes`) `.label`, `.detail` (plural `.detail.one` / `.detail.other` for the three with `{count}`), `.none`,
  `.effect`; `docker.note.disk-unseen`, `docker.note.never-runs`. Placeholders: `{runtime}` in the title and every
  summary,
  `{inside}` and `{unused}` in `answered`, `{count}` in the plural details. Plain English (rule 7).
- `modules/docker/tests/contract.rs` (new) — `assert_module_contract(&module, &Catalogue::Required(docs))` (no category,
  so no catalogue file is due); extended in 02/03 with the runs.
- `docs/dependencies.md` — the new crate in the used-by columns of `serde`, `thiserror`, `toml` and `fresher-testing`'s
  rows, as for `fresher-module-app-caches`.

## Produces

`DockerModule::new()`, `Runtimes` with its interests and `of_interest`, every text, for 02/02 and 02/03.

## Tests (negative first)

- `runtimes.rs`, `mod negative`: one test per `RuntimesError` variant, each with a table made for it (for `BadCli`:
  `/usr/bin/docker`, `../docker`, `Contents/*/docker`; for `BadPlace`: `*`, `..`, an absolute path, `user-temp`);
  `mod positive`: `the_shipped_table_has_docker_desktop_then_orbstack` (pin every field of both rows),
  `each_row_watches_its_socket_then_its_disk` (interests and `of_interest`), `every_interest_is_an_item_claim`.
- A test in `lib.rs` (or `src/tests.rs`): `every_row_has_its_gives_back_text` (for each row of the shipped table,
  `docker.runtime.<id>.gives-back` is in the dictionary, and no `docker.runtime.*` key names a row the table lacks); the descriptor: group Development, no
  category, no setting, no problem.
- `tests/contract.rs` passes.

## Done when

- `cargo nextest run -p fresher-module-docker`; clippy; fmt; the crate guard and the dependency guard (no other module,
  `#![forbid(unsafe_code)]`); `npm run check:machete`; `cargo deny check`.
- One commit: `feat(modules): the docker module's crate, runtimes table and texts`.

## Conclusions

Not written yet.
