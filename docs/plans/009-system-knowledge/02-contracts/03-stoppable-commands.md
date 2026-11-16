# 009/02/03 — Commands that can be stopped

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, testing, system, engine, cleaner, modules, app

## Files

- `crates/core/src/ports.rs` — `CommandRunner::run(program, args, cancel)`; `CommandRunError::Cancelled`.
- `crates/system/src/process.rs` — `ProcessCommandRunner`, moved from `app/tauri/src/process.rs`: the token checked
  with the timeout, the program killed when it is set.
- `crates/testing/src/fakes.rs` — `FakeCommandRunner` answers `Cancelled` for a set token.
- Callers: the engine's tool checks (the scan's token), the cleaner's tool removal (a token never set: it runs to its end),
  `modules/trial` (`cx.cancel()`).

## Produces

The cancellable runner modules and the cleaner use.

## Checks

- Negative: a program running when the token is set is stopped within a second and answers `Cancelled`; a token set
  before the start runs nothing; a missing program, a program that cannot start, a program past its time (as before).
- Positive: a program runs with its arguments and its large output (as before).

## Conclusions

- Built as written, with one change found while building: the cleaner's tool removal does **not** take the
  cleanup's token. Stop ends a cleanup after the item in flight (plan 008), and a tool killed halfway would leave its
  removal in an unknown state; it runs to its end or its 60 s timeout, with a token of its own that is never set.
- `CommandRunError::Cancelled { program }`; the engine's tool check that is cancelled changes no tool's state; the
  trial module's listing stopped by the token is a failed listing (the scan is cancelled anyway).
- `ProcessCommandRunner` and `COMMAND_TIMEOUT` moved to `fresher-system` (`git mv`, history kept); a token set before
  the call starts nothing; a token set while the program runs kills it at the next 10 ms poll
  (`a_program_running_when_the_token_is_set_is_stopped`: stopped well within a second).
