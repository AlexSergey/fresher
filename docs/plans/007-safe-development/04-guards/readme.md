# 007/04 — Guards

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app, repo

The draft's guards that 008 did not build and that stay worth their cost: development builds with their own identity,
the home folder looked up in one place, modules that start no process and read no environment. The design:
[`../00-design.md`](../00-design.md), sections 6 and 9.

Independent of plans 01–03; runs in wave 1.

## Steps

| #   | Step                                                  | Status   | Shared files                                                                         |
| --- | ----------------------------------------------------- | -------- | ------------------------------------------------------------------------------------ |
| 01  | [The development identity](./01-dev-identity.md)      | ✅ approved | `app/tauri/src/lib.rs`, `package.json` (`dev`)                                       |
| 02  | [Lints and the module guard](./02-lints.md)           | ✅ approved | `clippy.toml`, `app/tauri/src/lib.rs`, `scripts/check-crate-deps.ts` and its tests   |

In order (both touch `lib.rs`).

## Conclusions

**Built:** the development build's own identity (identifier, title, data folder) and the lints: one home lookup, no process
started by a module, the guard's exception named in the cleaner's deletion-point test.

**Measured:** `fresher-app` 53 after 04/01; `scripts` 35 and `fresher-app` with `fresher-sandbox` 101 after 04/02.

**Left:** no code of `crates/` is checked by the module guard (modules only, per the step). Starting `npm run dev` by hand
is in 05/02.

**Agent's calls taken during the build (to check):**

- The home lookup in the deletion-point guard is allowed in exactly two files, and only as `let home = <path>home_dir()` (tightened after the re-review). Costs, if wrong: a third legitimate lookup needs the guard edited, on purpose.
