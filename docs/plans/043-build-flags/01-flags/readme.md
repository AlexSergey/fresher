# 043/01 — Flags in the core and the shell

**Status:** ✅ approved · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, app/tauri, sandbox, app/ui
(bindings and mocks only), docs

The flags' types in the core, `flags.toml` and the build script that turns it into constants, the composition that
reads `high` from them, and `get_app_info` reporting them. Behaviour is unchanged: the file keeps today's values
(`withheld`, `hidden`). The design: [`../00-design.md`](../00-design.md), sections 1 to 4; agent's calls 1 to 3.

## Steps

| #   | Step                                                        | Status      | Shared files |
| --- | ----------------------------------------------------------- | ----------- | ------------ |
| 01  | [The flags' types in the core](./01-core-types.md)          | ✔️ done     | `crates/core/src/{flags.rs,lib.rs}` |
| 02  | [`flags.toml` and the build script](./02-build-script.md)   | ✔️ done     | `flags.toml`, `app/tauri/{build.rs,build/flags.rs,Cargo.toml}`, `app/tauri/src/{flags.rs,lib.rs,compose.rs,sandbox.rs}`, `Cargo.lock`, `docs/dependencies.md`, `docs/architecture/README.md` |
| 03  | [`get_app_info` reports the flags](./03-app-info.md)         | ✔️ done     | `crates/core/src/app_info.rs`, `app/tauri/src/{compose.rs,sandbox.rs,commands.rs,lib.rs,testing.rs}`, `app/tauri/tests/bindings.rs`, `bindings.generated.ts`, the UI's readers of `high`, its mocks and fixtures |

01, then 02, then 03.

## Conclusions

To be written when step 03 is done.
