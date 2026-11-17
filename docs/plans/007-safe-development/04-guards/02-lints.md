# 007/04/02 — Lints and the module guard

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app, repo

## Files

- `clippy.toml` — `disallowed-methods` gains `std::env::home_dir` and Tauri's `PathResolver::home_dir`, with the
  reason "the home folder is looked up once, in the app's setup; everything else gets its roots injected".
- `app/tauri/src/lib.rs` — the one use, under `#[expect(clippy::disallowed_methods, reason = "...")]`; the
  generator's binary (01/03) has the other, for the real protection list.
- `scripts/check-crate-deps.ts` and its test — the module guard: a file of `modules/*/src` that names `std::process`
  or `std::env` fails; a module runs a tool only through `CommandRunner` and gets its places from the context.
- `docs/quality.md` — the two checks in the list of guards.

## Produces

The draft's lints, narrowed to what protects files: a test or a module that forgets to inject a root cannot reach the
real home, and a module cannot start a process the cleaner's checks do not see.

## Checks

- Negative: the guard's tests — a planted `std::process::Command` and a planted `std::env::var` in a module's source
  fail; a `home_dir` call outside the setup fails Clippy (checked once by hand, written in the conclusions).
- Positive: the workspace passes `npm run lint` and `npm run check:crate-deps`.

## Conclusions

The home folder is looked up once (a `disallowed_methods` expectation in the app and the sandbox binary), and the module
guard rejects a module that starts a process. Review finding: the guard reads grouped imports across lines and nested
groups, strips block comments and reports once per module. The final review found the lint broke the cleaner's
deletion-point test (a red pre-push): it now names the two home lookups explicitly, and, after the re-review, accepts only a
plain `let home = <path>home_dir()` statement.
