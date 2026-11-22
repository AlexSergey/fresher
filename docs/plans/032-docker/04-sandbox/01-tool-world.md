# 032/04/01 — Tool answers

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, core, app, app/ui

Read first: `CLAUDE.md`, [`../00-design.md`](../00-design.md) section 8, `docs/deferred/sandbox-world.md`,
`docs/testing.md` ("Sandboxes", "The case file", "Sandbox mode"), `crates/sandbox/src/{case.rs,world.rs,error.rs}`
(how `[[case.running]]`, `[[case.elsewhere]]` and `access` travel from a case to `world.toml` and back),
`app/tauri/src/sandbox.rs` (`SandboxPorts::real`, `build`), `app/tauri/tests/sandboxes/harness.rs` (`fakes`),
`crates/testing/src/fakes.rs` (`FakeCommandRunner`), `crates/macos/src/privacy/panes.rs` (`OPEN`).

**Safety:** after this step sandbox mode never runs a program of the Mac but `/usr/bin/open` (the system interface:
Show in Finder, System Settings), whatever its case files say. Today it runs real programs (`ProcessCommandRunner`).

## Files

- `crates/sandbox/src/case.rs` — a case may state `[[case.tool]]`:
  - `program` — a path relative to the sandbox, every component a plain name (no `.`, `..`, empty, or absolute path):
    `Applications/OrbStack.app/Contents/MacOS/xbin/docker`;
  - `args` — the exact list of arguments (strings, each without a NUL);
  - `status` — the exit status, 0 by default;
  - `stdout` — the output, inline, at most 64 KiB (`stderr` is always empty).
  Refused (each its `CaseError` naming the case): a program that is absolute or not plain, an argument with a NUL, an
  output over 64 KiB, an unknown field.
- `crates/sandbox/src/world.rs` — `World` gains `tools: Vec<StatedTool>` (`program: PathBuf`, `args: Vec<String>`,
  `status: i32`, `stdout: String`), joined in the order of the files and their cases; `world.toml` writes them as
  `[[tool]]` (`skip_serializing_if` empty, so every existing `world.toml` is unchanged) and reads them back. Two cases
  stating the same program and arguments with a different answer are refused (`SandboxError::ToolTwice { program }`);
  the same answer twice is kept once. `World::declares_tools()`. `World::commands(sandbox: &Path) -> WorldCommands`.
- `crates/sandbox/src/world.rs` (or `world/commands.rs` if `world.rs` grows past its size) — `WorldCommands`:
  implements `CommandRunner`; a call whose program is `sandbox.join(program)` of a stated tool and whose arguments are
  exactly its `args` answers its `status` and `stdout`; a set cancel token answers `Cancelled`; every other call is
  recorded (`program` and arguments joined by spaces) and answers `NotInstalled`. `WorldCommands::unanswered() ->
  Vec<String>`. Shared behind an `Arc` so the harness can read what was recorded.
- `crates/sandbox/src/error.rs` — the new refusals.
- `crates/core/src/app_info.rs` — `Simulated::Tools` ("The answers of the programs modules run: the case files'.").
  `app/ui/src/locales/en.json` — `ui.build-banner.simulated.tools` "tools". `bindings.generated.ts` regenerated.
- `app/tauri/src/sandbox.rs`
  - `SandboxCommands { world: Arc<WorldCommands>, open: Arc<dyn CommandRunner> }` implementing `CommandRunner`: a call
    to exactly `fresher_macos::OPEN` (`/usr/bin/open`) goes to `open`; every other call to `world`.
  - `SandboxPorts::real` takes `commands: Arc::new(SandboxCommands { world: world.commands(sandbox.path()), open:
    ProcessCommandRunner::new(COMMAND_TIMEOUT) })`, with or without stated tools.
  - `build` adds `Simulated::Tools` when `world.declares_tools()`.
  - Doc comments of `SandboxPorts` and `real` say the commands are the world's.
- `app/tauri/tests/sandboxes/harness.rs` — `fakes` takes the world's `WorldCommands` instead of
  `FakeCommandRunner::default()`; after the scan and after the cleanup of every case file, a non-empty `unanswered()`
  fails the file: "a program the case file does not state was run: <calls>". No existing case file runs a program, so
  every existing case file still passes.

## Produces

`[[case.tool]]`, `World::commands`, `WorldCommands`, `SandboxCommands`, `Simulated::Tools`, the autotest's check of
unstated calls, for 04/02 and 04/03.

## Tests (negative first)

- `case.rs`, `mod negative`: `a_tool_with_an_absolute_program_is_refused`, `a_tool_with_a_dotted_program_is_refused`,
  `a_tool_argument_with_a_nul_is_refused`, `a_tool_output_over_64_kib_is_refused`, `an_unknown_tool_field_is_refused`;
  `mod positive`: `a_tool_reads_with_its_defaults` (status 0).
- `world.rs`, `mod negative`: `two_answers_to_one_call_are_refused`, `an_unstated_call_answers_not_installed_and_is_recorded`,
  `a_stated_program_with_other_arguments_is_unanswered`, `a_set_cancel_token_answers_cancelled`,
  `a_world_without_tools_writes_no_tool` (byte-for-byte `world.toml` of a file without tools); `mod positive`:
  `tools_round_trip_through_world_toml`, `a_stated_call_answers_its_status_and_output`,
  `the_same_answer_twice_is_kept_once`.
- `app/tauri/src/sandbox/tests.rs`, `mod negative`: `sandbox_commands_never_reach_a_real_program_but_open` (a call to
  `/bin/echo` is unanswered and recorded, never run); `mod positive`: `open_goes_to_the_real_runner` (with a fake as
  `open`), `the_banner_names_tools_when_the_world_states_them`.
- `app/tauri/tests/sandboxes/harness.rs`: the check of unstated calls is a function of its own
  (`check_no_unstated_calls(file, commands)`), tested directly: negative `an_unstated_call_fails_the_file_naming_it`;
  positive `no_call_passes`. No registered module runs a program before 04/03, so the end-to-end proof is 04/03's
  by-hand mutation check.

## Done when

- `cargo nextest run -p fresher-sandbox -p fresher-app -p fresher-core`; `npm run sandbox:test` (every existing case
  file passes, goldens unchanged); the UI's `npx vitest run` for the banner; clippy; fmt.
- One commit: `feat(sandbox): tool answers; sandbox mode runs no program but open`.

## Conclusions

Not written yet.
