# 024/02 — The module

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, app, docs

`modules/unused-toolchains` (`fresher-module-unused-toolchains`): its descriptor, its table of managers, two
categories and their texts, the Node switch, the pointers' parsers, the decision, its fixtures and golden, and its
registration in every build. The design: [`../00-design.md`](../00-design.md), sections 3 to 8.

## Steps

| #   | Step                                                                   | Status      | Shared files                                                                                   |
| --- | ---------------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------- |
| 01  | [The crate, the table and the texts](./01-crate.md)                    | 🔨 in progress | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`                              |
| 02  | [What points at a version](./02-pointers.md)                           | 🔨 in progress | `modules/unused-toolchains/src/pointers/` only                                                 |
| 03  | [The decision](./03-decision.md)                                       | 🔨 in progress | `modules/unused-toolchains/` only                                                              |
| 04  | [Fixtures, golden and registration](./04-fixtures-and-registration.md) | 🔨 in progress | `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the module lists), `Cargo.lock` |

In order: 01, 02, 03, 04.

## Conclusions

Built: the crate, the pointers, the decision, the fixtures and the registration. Differences from the design are in
each step: `Version { index, name }`, the switch-off row decided with empty pointers, linked folders counted as
installed for the newest (plain files not), no `-off` home, a kept-unknown home, `partial-scan` proven in
`src/tests.rs` only, and the case-file test red between 02/04 and 03/01. See the
[global conclusions](../readme.md#conclusions).
