# 038/01/01 — The demo header and the scenery kind

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox

A case file may now be a **demo world**: a `[demo]` header with the world's own clock, and cases of a third kind,
`scenery`, whose content is shown, never checked. The generator writes a demo world with its own `now` and takes a
case file by path. Sizes, the volume and `shows` are read here but used from 01/02 and 01/03. The design:
[`../00-design.md`](../00-design.md), section 1.

Read first: `crates/sandbox/src/{lib,case,generate,expected,world,main}.rs`, `docs/testing.md` ("The case file"),
`fixtures/sandboxes/trial.toml`. Nothing outside `crates/sandbox` changes in this step.

## Files

- `crates/sandbox/src/demo.rs` (new) — `pub struct Demo { now: SystemTime, sizes: DemoSizes, shows: Vec<CategoryId>,
  volume: Option<DemoVolume> }` with getters; `pub enum DemoSizes { Sparse, AsDeclared }` (TOML `"sparse"` /
  `"as-declared"`, default `Sparse`); `pub struct DemoVolume { name: String, capacity: u64, used: u64, free: u64 }`
  with getters. Read from a `RawDemo` (`deny_unknown_fields`, kebab-case): `now: u64` (seconds since the Unix epoch,
  required, > 0), `sizes` (optional), `shows: Vec<String>` (required, non-empty, each a valid category id
  `<module>.<name>` via the same check `case.rs` uses, no duplicate), `volume` (optional table: `name` non-empty,
  `capacity`, `used`, `free`, with `used + free <= capacity` and `capacity > 0`).
- `crates/sandbox/src/case.rs`:
  - `CaseKind::Scenery` (`"scenery"`), documented: "a part of a demo world: what it shows is not checked".
  - `RawCaseFile` gains `demo: Option<RawDemo>`; `CaseFile` gains `demo: Option<Demo>` and `pub fn demo(&self) ->
    Option<&Demo>`.
  - Refusals, one new `CaseError` variant each, with the case's or file's name in the message: `DemoWithModule` (a
    demo file names a `module`), `CheckedCaseInDemo { case }` (a negative or positive case in a demo file),
    `SceneryOutsideDemo { case }` (a scenery case in a file without `[demo]`), `SceneryWithExpectation { case }` (a
    scenery case with `expect`, `map` or `left-out`), `Demo { reason }` (each refusal of `demo.rs` above, the reason
    naming the field). The negatives-before-positives order check ignores scenery (a demo file has no other kind).
  - Every `match` on `CaseKind` and every comparison with it in the crate handles `Scenery` explicitly (grep
    `CaseKind` and `.kind()`); a scenery case writes its tree, its apps, its running apps and its world facts exactly
    as a negative case does.
- `crates/sandbox/src/expected.rs` — `EXPECTED.md` of a demo world: a head line "A demo world: what it shows is not
  checked." and one line per scenery case with its name; the checklist of a normal case file is unchanged (its golden
  tests stay green).
- `crates/sandbox/src/generate.rs` — no signature change: the caller passes the demo's `now`. A doc line on
  `generate`: a demo world is generated at its own `now`.
- `crates/sandbox/src/world.rs` — no change in this step (sizes and volume in 01/02).
- `crates/sandbox/src/lib.rs` — `mod demo;` and `pub use demo::{Demo, DemoSizes, DemoVolume};`; the crate doc names
  demo worlds in one sentence.
- `crates/sandbox/src/main.rs`:
  - `generate <feature|all|path> [--into <dir>]`: a target containing `/` or ending in `.toml` is a case file read by
    its path (relative to the current folder), its default `--into` `sandbox/<file stem>/`; a bare name stays
    `fixtures/sandboxes/<feature>.toml`. `all` never reads `fixtures/demos/`.
  - The generation moment is the file's `demo().now()` when it has a demo header, else `SystemTime::now()` as today.
  - `USAGE` updated: `usage: fresher-sandbox generate <feature|all|path.toml> [--into <dir>]`.

## Produces

`CaseFile::demo()`, `Demo`, `DemoSizes`, `DemoVolume`, `CaseKind::Scenery`; the generator by path, for 01/02, 01/03
and 02/01.

## Checks

- `case.rs` / `demo.rs` tests, negatives first (`mod negative`, then `mod positive`):
  - each new refusal with a minimal text: a demo file with `module`; a negative case in a demo file; a scenery case
    without `[demo]`; a scenery case with `expect`, with `map`, with `left-out`; `now = 0`; `shows = []`; a bad
    category id in `shows`; a duplicate in `shows`; an unknown key in `[demo]`; a volume with `used + free >
    capacity`; a volume with an empty name.
  - then: a demo file with two scenery cases parses, `demo()` gives its `now`, `sizes` (`AsDeclared` and the default
    `Sparse`), `shows` and `volume`; a normal case file has `demo() == None`.
- `generate.rs` tests: a demo world generated into a temporary folder writes every scenery case's files with ages
  counted from the demo's `now` (a file of `age = "40d"` has a modification time of `now - 40 days`), and its
  `world.toml` `now` equals the demo's.
- `expected.rs`: the demo checklist's text for a two-case world (exact string).
- `main.rs` tests: `parse_args` keeps a path target as given; `case_files` reads a target with `/` or `.toml` by
  path, and refuses a missing one naming its path.
- `cargo nextest run -p fresher-sandbox` and `-p fresher-app --test sandboxes` (the autotest still passes: no case
  file changed); clippy; fmt.

## Conclusions

Built as briefed: `crates/sandbox/src/demo.rs` (`Demo`, `DemoSizes`, `DemoVolume`, refusals), the `scenery` case kind
and its four refusals in `case.rs`, the demo world's section of `EXPECTED.md`, generation at the world's own `now`, and
the generator taking a path (`/` or `.toml`; default folder `sandbox/<file stem>/`). 192 sandbox tests, 92 sandbox
autotests, none of the existing goldens changed.

- `DemoWithModule` and `Demo` errors carry the file's feature so every message names the file.
- A scenery case also refuses `[[case.insight]]` (it states nothing checked), decided in review.
