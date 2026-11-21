# 024/01 — What the core lacks

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** core, testing, sandbox, app, docs

The two things the module needs and the core does not have: a module reading a small text file it matched, and test
worlds that can state a file's text and a module's setting. The design: [`../00-design.md`](../00-design.md), "What is
missing" and sections 1 and 2.

## Steps

| #   | Step                                                                  | Status      | Shared files                                                                                                   |
| --- | --------------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------------- |
| 01  | [The small-file reader](./01-small-files.md)                          | 🔨 in progress | every `SystemPorts { .. }` site, `app/tauri/src/compose.rs`, `crates/testing/src/{system_fakes,run_module}.rs`, `docs/architecture/modules.md` |
| 02  | [File text and settings in the test worlds](./02-test-worlds.md)      | 🔨 in progress | `crates/testing/src/tree_description.rs`, `crates/sandbox/src/`, `app/tauri/tests/sandboxes.rs`, `docs/testing.md` |

01 and 02 in parallel; the second to finish rebases on the first (`crates/testing`).

## Conclusions

Both steps are built: the core reads small text files without following a link, and the test worlds carry file text
and settings. The only edge added is `fresher-sandbox` to `fresher-store`. Walk, scanner, cache and cleaner are
unchanged, so no benchmark is due. See the [global conclusions](../readme.md#conclusions).
