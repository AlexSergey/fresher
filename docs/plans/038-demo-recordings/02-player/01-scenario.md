# 038/02/01 — The scenario and the `demo` command

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, fixtures

A scenario is `fixtures/demos/scenarios/<name>.toml`: its world, its window, and steps from a closed list. It is
parsed and checked in `crates/sandbox` into typed values that `ts-rs` exports to the UI (02/02). The generator's new
`demo <scenario>` command writes the scenario's world, fresh, into a neutral folder. The design:
[`../00-design.md`](../00-design.md), sections 1 ("Where it is generated") and 2.

Read first: 01/01 (as built: `CaseFile`, `Demo`, the generator by path), `crates/sandbox/src/main.rs`,
`crates/sandbox/src/layout.rs` (`Sandbox::open`, the marker check), `crates/core/src/app_info.rs` (how a core type
derives `ts_rs::TS` with `#[ts(...)]` attributes and serde's tag and case), `app/tauri/tests/bindings.rs`.

## Files

- `crates/sandbox/Cargo.toml` — `ts-rs.workspace = true` under `[dependencies]` (a workspace dependency already used
  by `fresher-core`: no new crate in `Cargo.lock`'s set; `cargo deny` and `cargo machete` stay green).
- `crates/sandbox/src/scenario.rs` (new) — the types sent to the UI, each `#[derive(Clone, Debug, PartialEq, Eq,
  Serialize, ts_rs::TS)]` with a doc comment (it becomes the TypeScript doc), camelCase or kebab-case as the core's
  IPC types are (follow `app_info.rs`):
  - `pub struct Scenario { pub name: String, pub world: String, pub window: DemoWindow, pub steps: Vec<Step> }`;
  - `pub struct DemoWindow { pub width: u32, pub height: u32 }` (logical points);
  - `pub enum Step` tagged by `kind`: `Click { target: Target }`, `Hover { target: Target }`, `Scroll { target: Target,
    by: i32 }`, `Wait { until: Wait }`, `Pause { ms: u32 }`;
  - `pub struct Target { pub role: Role, pub name: NameMatch }`;
  - `pub enum Role` (kebab-case strings): `Button`, `Link`, `Tab`, `Checkbox`, `Switch`, `Heading`, `Region`, `Dialog`,
    `Row`;
  - `pub enum NameMatch` tagged: `Exact { text: String }`, `Start { text: String }`;
  - `pub enum Wait` tagged: `State { state: AppState }`, `Target { target: Target }`;
  - `pub enum AppState` (kebab-case): `ScanFinished`, `CleanupFinished`;
  - `pub struct DemoPlay { pub scenario: Scenario, pub private: Vec<String> }` — what the shell answers the UI (the
    texts the picture must never show);
  - `pub enum DemoOutcome` tagged, `Deserialize` too: `Finished { elapsed_ms: u32 }`, `Failed { step: u32, reason:
    String }` (`step` counted from 1) — what the UI reports.
  - `Scenario::parse(name: &str, text: &str) -> Result<Scenario, ScenarioError>` from raw TOML structs
    (`deny_unknown_fields`): `world` (a plain name: letters, digits, `-`), `window = { width, height }`, `[[step]]`
    tables each with exactly one of `click`, `hover`, `scroll`, `wait`, `pause`. A target table is `{ role, name }` or
    `{ role, name-start }`; `scroll` adds `by`; `wait` is a string (an `AppState`) or a target table; `pause` an
    integer of milliseconds.
  - `ScenarioError` (thiserror), every variant naming the step (counted from 1) where there is one: `Format`
    (TOML or an unknown key), `BadWorld`, `Window` (width outside 900–1600 or height outside 600–1000), `NoSteps`,
    `StepKeys { step }` (zero or more than one key), `Name { step }` (both `name` and `name-start`, neither, or an
    empty text), `ScrollZero { step }`, `Pause { step }` (outside 1–10 000), `TooLong` (the pauses add up to more than
    20 000 ms).
  - `Scenario::read(path: &Path)` — reads the file, its name the file stem; an IO error is its own variant naming the
    path.
- `crates/sandbox/src/lib.rs` — `mod scenario;` and its public types re-exported.
- `crates/sandbox/src/main.rs` — a second command, `demo <scenario> [--into <dir>]`:
  - reads `fixtures/demos/scenarios/<scenario>.toml` (a plain name only), then its world
    `fixtures/demos/<world>.toml` (refused unless it has a `[demo]` header);
  - the folder is new on every run: `--into` (refused when anything is there, a link or a file included), else
    `/private/tmp/fresher-demo/<scenario>-<n>` with the first `n` from 1 not taken (a constant `DEMO_FOLDER =
    "/private/tmp/fresher-demo"`). Nothing is ever removed, topped up or overwritten: only the cleaner deletes
    (docs/restrictions/deletion-safety.md), so there is no second deletion point. Old demo folders stay until macOS
    clears `/private/tmp`; the generated folder keeps its sandbox marker, which tells a demo world;
  - generates the world at its demo `now`; prints one line, `wrote the demo world <world> into <folder>`, the folder
    canonical: the recorder (03/01) and the shell (02/02) take the folder from it;
  - `USAGE` names both commands.
- `fixtures/demos/scenarios/overview.toml` (new, first version) — `world = "developer-mac"`, `window = { width =
  1100, height = 720 }`, steps: a pause of 800, `click` the scan button (its exact accessible name read from
  `app/ui/src/locales/en.json` and the scan screen's component), `wait = "scan-finished"`, a pause of 1500. Finished
  in 04/01.

## Produces

`Scenario`, `Step`, `Target`, `Role`, `NameMatch`, `Wait`, `AppState`, `DemoWindow`, `DemoPlay`, `DemoOutcome`,
`Scenario::read`, and `fresher-sandbox demo`, for 02/02, 02/03 and 03/01.

## Checks

- `scenario.rs`, negatives first: one test per `ScenarioError` variant with a minimal text (an unknown key, a world
  `../x`, a window 800 wide, no steps, a step with `click` and `pause`, an empty step, a target with both names, one
  with neither, an empty name, an unknown role, `by = 0`, `pause = 0`, `pause = 10001`, pauses adding up to 20 001);
  the step number is in the message. Then: the draft's example parses into the exact typed steps; a `wait` string and
  a `wait` target; serialized to JSON, a step reads `{"kind":"click","target":{...}}` (the shape the UI gets).
- `main.rs` tests, negatives first: `demo` with a name that is not plain is refused; `demo --into` a folder that
  exists (holding a file, empty, a link, a file, or a world it wrote before) is refused and left as it is; a scenario
  whose world has no `[demo]` is refused. Then: `demo` into a missing folder writes the world; two runs make
  `<scenario>-1` and `<scenario>-2`, the first left as it was; the first free number is taken. Nothing is deleted.
- A test that every `fixtures/demos/scenarios/*.toml` parses and names a world in `fixtures/demos/`.
- `cargo nextest run -p fresher-sandbox`; clippy; fmt; `cargo deny check`; `cargo machete`.
- `docs/dependencies.md`: `fresher-sandbox` added to the used-by column of `ts-rs`'s row.

## Conclusions

Built as briefed (`crates/sandbox/src/scenario.rs`, `demo_command.rs`, `fixtures/demos/scenarios/overview.toml`),
with one change: the design's fresh folder was a removal, and only the cleaner deletes. Controller's ruling of
2026-11-22: every `demo` run **creates** a new folder `/private/tmp/fresher-demo/<scenario>-<n>` (the first free `n`,
claimed atomically with `create_dir`), an existing `--into` is refused, nothing is removed, topped up or overwritten; old
folders stay until macOS clears `/private/tmp`.

- 232 sandbox tests; `ts-rs` added to `fresher-sandbox` (`Cargo.lock` gains one edge; `dependencies.md` updated).
- `DemoOutcome` also derives `Deserialize` (the recorder reads it).
