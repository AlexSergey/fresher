# 043 — Build flags, and High and Largest back on: design

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, app/tauri, app/ui, sandbox,
scripts, e2e, ci, docs

The design the plans argue from. Written from the approved draft of 2026-11-26 and a read of the code on
`feat/043-build-flags` (d50221fc). Each step reads its code again before it changes it.

## Intent

The user's task (translated, 2026-11-26, after the first tester's feedback,
[`../../deferred/tester-feedback-0.1.0.md`](../../deferred/tester-feedback-0.1.0.md)): "A more advanced system of
feature flags, set before the build: look at how it is today and improve it, as its own plan. Turn Large and the High
level back on." The draft was approved the same day with every recommended decision and agent's call taken.

## Goals

1. One place, read before the build, says what a build ships with: `flags.toml` at the repository root, one table per
   flag with its value for a release and for a development build and a line saying what it does.
2. Each flag is a typed value (an enum, never a string at the use site), given to the engine and reported to the UI
   by `get_app_info`; the UI shows or hides a screen only from it.
3. A build may override a flag only from its own command line (`FRESHER_FLAGS=largest=hidden`), never from a file the
   user can write or a setting: a flag is the build's, not the user's.
4. The first flags: `high` (`offered` / `withheld`) and `largest` (`shown` / `hidden`), apart: Largest no longer
   follows High. 0.2.0 ships both on.
5. A guard, `npm run check:flags` in the pre-push checks and CI, holds the file's shape and that every flag is read
   somewhere; the release's text names the flags it was built with.
6. The e2e core mock and the sandbox take the flags like the app, so a test names the flags it runs with.

## Non-goals

- Runtime switches, a user setting, or a debug-panel switch (the panel is
  [deferred](../../deferred/dev-debug-panel.md); it may later switch a flag in a debug build only).
- Modules as flags: `ModuleSet` stays (YAGNI until a half-built module must stay out of a release).
- Moving `HighLevel` out of the core or changing what `Withheld` withholds (plan 042's rule stays as it is).
- Largest's folders section: plan [044](../044-largest-folders/readme.md).

## What already exists (read on d50221fc)

| What | Where |
| --- | --- |
| The High switch | `HighLevel::{Offered, Withheld}` (`crates/core/src/danger.rs`), a required argument of `Engine::new` / `Engine::with_cleanup`; `Engine::high_level()` |
| The app's value | `app/tauri/src/compose.rs::normal` sets `high: HighLevel::Withheld` in `Shared` (every build); `assemble` passes `Shared.high` to the engine |
| Sandbox mode | `SandboxMode { high, .. }` (`app/tauri/src/sandbox.rs`): `Withheld` from `new` and `from_env`; `offering_high()` for the autotest only |
| The case file | `high = "withheld"` (`crates/sandbox/src/case.rs`); the harness offers High unless a file says so (`app/tauri/tests/sandboxes/harness.rs`); `EXPECTED.md` writes "Not shown in the app: High is withheld in this release." under a High positive (`crates/sandbox/src/expected.rs`) |
| The app's info | `AppInfo { version, build, high, home, now }` (`crates/core/src/app_info.rs`); `commands.rs::app_info` reads `engine.high_level()`; its test asserts `high` |
| The UI | `frame-sidebar.tsx` shows Largest while `useAppInfo()?.high === 'offered'`; `state/navigation.ts::screenToShow(.., high)` sends Largest back to Scan otherwise; `app-frame.tsx` passes `info?.high` |
| The e2e mock | `CoreScript.high?: HighLevel` (`app/ui/e2e/core-mock.ts`, `withHigh`); `support/withheld.ts` mirrors the engine's rule and sets `high: 'withheld'`; `visual.spec.ts` photographs `finished-scan-withheld.png` |
| The build script | `app/tauri/build.rs`: the command allowlist for `tauri_build`; `[build-dependencies]` holds `tauri-build` only |
| The bindings | `app/tauri/tests/bindings.rs`; `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings` rewrites `app/ui/src/ipc/bindings.generated.ts` |
| Release text | `scripts/release-version.ts notes <v>` prints `docs/releases/<v>.md` then the changelog section (`releaseBody`); `.github/workflows/release.yml` writes it to `release/notes.md` |
| Guards | `scripts/check-*.ts` with a `*.test.ts` beside each (`node --test`), named in `package.json` and its `pre-push`, and in `ci.yml`'s `lint` job |
| `toml` | `toml = "=1.2.1"` in the workspace (`Cargo.toml`), used by the modules, rules, sandbox and testing |

## Design

### 1. `flags.toml` (plan 01, step 02)

At the repository root, beside `Cargo.toml`. One table per flag, its keys exactly these five:

```toml
# What a build ships with (plan 043). Read by app/tauri/build.rs before the build; nothing reads it at runtime.
# A build may override a value on its command line: FRESHER_FLAGS=largest=hidden,high=withheld.
# `type` names the enum of fresher-core the value becomes; each value is a variant in kebab-case.

[high]
about = "Whether items at High, which may hold the only copy of the user's data, are offered."
type = "HighLevel"
values = ["offered", "withheld"]
release = "withheld"
development = "withheld"

[largest]
about = "Whether the sidebar shows the Largest screen."
type = "LargestScreen"
values = ["shown", "hidden"]
release = "hidden"
development = "hidden"
```

Plans 01 and 02 keep today's behaviour (`withheld`, `hidden`); plan 03 turns both on. Development follows release
(the draft's decision), so the walk sees what ships.

### 2. The core's types (plan 01, step 01)

`HighLevel` stays in `crates/core/src/danger.rs`. A new `crates/core/src/flags.rs`:

```rust
/// What a build ships with: set before the build from `flags.toml` by the app's build script.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BuildFlags {
    pub high: HighLevel,
    pub largest: LargestScreen,
}

/// Whether the sidebar shows the Largest screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum LargestScreen { Shown, Hidden }
```

The struct lists every flag; nothing else in the core reads `largest` (the engine has no use for it: a pick is High
and already guarded by `HighLevel`). `AppInfo.high` becomes `AppInfo.flags: BuildFlags` (step 03).

### 3. The build script (plan 01, step 02)

- `app/tauri/build/flags.rs` (new): a pure function `resolve(file: &str, profile: Profile, overrides: Option<&str>) ->
  Result<Vec<Resolved>, FlagsError>` and `render(&[Resolved]) -> String`. `Profile::{Release, Development}`;
  `Resolved { name, type_name, value }` (flags come in the order of their names); `FlagsError` (a hand-written enum with `Display`, no `thiserror` in the build
  script) names the flag and the key.
- `app/tauri/build.rs` gains `#[path = "build/flags.rs"] mod flags;` and, before `tauri_build`:
  - `cargo:rerun-if-changed=../../flags.toml`, `cargo:rerun-if-changed=build/flags.rs`,
    `cargo:rerun-if-env-changed=FRESHER_FLAGS`;
  - the profile from `CARGO_CFG_DEBUG_ASSERTIONS` (set: `Development`), the same test as `cfg!(debug_assertions)` in
    `compose.rs::own_build`;
  - writes `render(..)` to `$OUT_DIR/flags.rs`; any `FlagsError` fails the build with its message.
- Refused at build time, each with its own `FlagsError` variant: a file that is not TOML; a table missing a key or
  holding another one; a name outside `[a-z][a-z0-9]*` or a `type` that is not a PascalCase identifier; an empty
  `about`; fewer than two `values`, a repeated one, or one that is not kebab-case; a
  `release` or `development` not in `values`; in `FRESHER_FLAGS` (comma-separated `name=value`, spaces around items
  allowed, empty means none) an unknown name, a value not in that flag's `values`, a repeated name or an item without
  `=`.
- The generated file (no hand edits; it lives only in `OUT_DIR`):

  ```rust
  // Generated by app/tauri/build.rs from flags.toml and FRESHER_FLAGS. Do not edit.
  pub(crate) const FLAGS: fresher_core::BuildFlags = fresher_core::BuildFlags {
      high: fresher_core::HighLevel::Withheld,
      largest: fresher_core::LargestScreen::Hidden,
  };
  ```

  A value becomes its variant by turning kebab-case into PascalCase. The compiler holds the rest: a table without a
  field of `BuildFlags`, a field without a table, a `type` that is not the field's type, or a value with no variant
  does not compile.
- `app/tauri/src/flags.rs` (new): `include!(concat!(env!("OUT_DIR"), "/flags.rs"));` with a doc comment pointing at
  `flags.toml`.
- `toml.workspace = true` in `[build-dependencies]` and `[dev-dependencies]` of `fresher-app` (the parser's tests run
  in the lib's tests: `#[cfg(test)] #[path = "../build/flags.rs"] mod build_flags;` in `lib.rs`).

### 4. The composition (plan 01, steps 02 and 03)

- `compose.rs::normal` sets `high: FLAGS.high` (the comment names `flags.toml`); `SandboxMode::new` and `from_env`
  start from `FLAGS.high`; `offering_high()` stays the autotest's way to offer.
- `Shared.high` becomes `Shared.flags: BuildFlags`; `assemble` passes `flags.high` to the engine. `SandboxMode`
  carries `flags` (its `high` overridden by `offering_high()`).
- `Start` gains `flags: BuildFlags`; `lib.rs` manages it; `commands.rs::app_info(build, flags, engine)` reports
  `flags: BuildFlags { high: engine.high_level(), ..*flags }` so the reported `high` is always the engine's.
- `testing.rs`'s shell does not manage `BuildFlags`: Tauri's `manage` keeps the first value, the shell routes no IPC, and the `app_info` tests manage their own flags as they do for `Build`.

### 5. The UI (plan 01 step 03, plan 02 step 02)

- 01/03 renames mechanically: `info.high` → `info.flags.high` wherever the UI reads it (behaviour unchanged).
- 02/02: `frame-sidebar.tsx` shows Largest while `flags.largest === 'shown'`; `screenToShow(.., largest:
  LargestScreen | null)` sends Largest back to Scan while it is hidden or not known; `app-frame.tsx` passes
  `info?.flags.largest`. High no longer decides Largest.
- With `high = withheld` and `largest = shown` (no build ships it), Largest lists rows and the core refuses a pick
  (`HighWithheld`, its toast says why): agent's call 4.

### 6. The e2e mock and the gallery (plan 02, step 01)

- `CoreScript.high` becomes `CoreScript.flags?: Partial<BuildFlags>`, merged into the app info's `flags`; every
  scripted `appInfo` answers `flags: { high: 'offered', largest: 'shown' }` (what the core can produce; 042's call 18).
- `withheld(script)` sets `flags.high = 'withheld'` only. A spec that photographs or walks the 0.1.0 app states both:
  `{ ...withheld(script), flags: { high: 'withheld', largest: 'hidden' } }` (so `finished-scan-withheld.png` keeps its
  baseline).
- The gallery's stories read no app info today (`rg useAppInfo app/ui/src`: screens only); nothing changes there.

### 7. The guard and the release text (plan 02, step 03)

- `scripts/flags.ts` with `check`, beside `scripts/flags.test.ts`. It reads `flags.toml` through a small
  reader of exactly the file's shape (comments, `[name]` headers, `key = "string"`, `values = ["a", "b"]`) and refuses
  any other line: the repository has no TOML package for Node, and a file the guard cannot read fails it, so the two
  readers never disagree in silence.
- `check` (`npm run check:flags`, in `pre-push` after `check:texts` and in `ci.yml`'s `lint` job): every table has the
  five keys and no other; `about` is one sentence ending in a full stop, at most 120 characters; `values` hold at least
  two kebab-case names, none twice; `release` and `development` are among them; and each flag is read: `flags.<name>`
  or `FLAGS.<name>` appears in a source file under `app/tauri/src` or `app/ui/src` (tests, stories and
  `*.generated.ts` excluded).
- `buildSection` builds the section the release's text ends with (`release-version.ts notes` prints it):

  ```
  ## This build

  - `high`: offered — Whether items at High, which may hold the only copy of the user's data, are offered.
  - `largest`: shown — Whether the sidebar shows the Largest screen.
  ```

  from the release column and `FRESHER_FLAGS` (the same rules as the build script). `release-version.ts notes` appends
  it to `releaseBody`, so `.github/workflows/release.yml` needs no new step. The tag's workflow sets no
  `FRESHER_FLAGS`: a release ships what `flags.toml` says on `main` (agent's call 5).

### 8. Turning High and Largest on (plan 03)

- `flags.toml`: `high` `offered`, `largest` `shown`, release and development.
- `EXPECTED.md`'s line under a High positive becomes "Shown in the app only while the build offers High (`flags.toml`,
  `high`)." (the sandbox crate cannot see the app's flags; the line is true for either value): agent's call 6.
- `docs/restrictions/deletion-safety.md`: "High is withheld in the first release" becomes "High is withheld when the
  build says so": the rule holds while the build's `high` is `withheld`; 0.1.0 withheld it, 0.2.0 offers it; the
  value is set only before the build (Why: a file in the home folder must never switch what can remove at High).
- Test cases by hand, `docs/test-cases/high-and-largest.md` (HL-01 to HL-04), and the user's walk.

### 9. Tests

- Core: `BuildFlags` and `LargestScreen` serialize as the UI reads them; `AppInfo` with `flags`.
- The parser (`build/flags.rs`, run in `fresher-app`'s tests): one negative per refusal, then the positives (each
  profile's column, an override, an empty override, the rendered text).
- The composition: the normal engine and sandbox mode report `FLAGS.high`, whatever its value (no test names the
  shipped value, so plan 03 changes no Rust test); `app_info` reports the managed flags with the engine's `high`.
- UI: the sidebar and `screenToShow` for each `largest`, apart from `high`; the e2e specs name their flags.
- The guard: one negative per check, then a valid file and the notes for an override.

## Risks

- **A stale build**: Cargo reruns the script only on the files and variable it names; `build.rs` names
  `flags.toml`, its own module and `FRESHER_FLAGS`. Tested by hand once in 01/02 (touch the file, rebuild, see the
  script run).
- **`cargo machete`** may not see `toml` used only through `#[path]` in tests; if it flags it, the step adds
  `[package.metadata.cargo-machete] ignored = ["toml"]` with a comment.
- **Coverage**: `build/flags.rs` is compiled into the lib's tests and counted; its tests keep it above the bar.

## Open questions

None.

## Decisions

- 2026-11-26: the draft approved, with every recommended decision and agent's call taken: build time, not runtime;
  development follows release; modules are not flags yet; turning High on re-runs the High parts of plan 042's walk
  with a test case for each; the file `flags.toml` at the root and the override `FRESHER_FLAGS` (the user).
- 2026-11-26: Largest's files stay removable at High (the user's answer in plan 044's draft).

## Agent's calls (to check)

1. **`BuildFlags` lives in the core** (`crates/core/src/flags.rs`), beside `HighLevel`, because `AppInfo` carries it
   across the IPC; the build script that fills it belongs to `app/tauri`. `AppInfo.high` is replaced by
   `AppInfo.flags` (one place for every flag), not kept beside it.
2. **The file names each flag's enum (`type`)** and the build script stays generic: the compiler, not a list in
   `build.rs`, refuses a flag the code does not have or a value with no variant.
3. **The parser is tested in `fresher-app`'s tests** through `#[path]`, as build scripts have no tests of their own;
   `toml` becomes a build- and dev-dependency of `fresher-app` (already in the workspace).
4. **`high = withheld` with `largest = shown` is allowed**: the flags are apart, as asked; the Largest screen then
   lists rows and the core refuses a pick. No build ships it; plan 044's folders make it useful.
5. **A tagged release builds from `flags.toml` alone**; `FRESHER_FLAGS` is for local builds and the workflow's dry
   run, so what a release ships is always in git. The release's text ends with "This build" listing each flag.
6. **`EXPECTED.md`'s High line says when it shows** instead of claiming High is withheld; the sandbox crate is not
   given the app's flags.
7. **The guard reads `flags.toml` with its own strict reader** of the file's shape, rather than adding a TOML package
   for Node.

## Conclusions

See [`readme.md`](./readme.md), "Conclusions".
