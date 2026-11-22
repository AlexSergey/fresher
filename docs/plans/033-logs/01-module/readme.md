# 033/01 — The module

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** modules, testing, docs

`modules/logs` (`fresher-module-logs`): its descriptor, two categories and their texts and catalogue files, its
setting, six interests, the decision per file, the second exception of "Unknown is not junk", its fixtures, golden and
contract. Its registration in the app is made in 02/01, with the case file the app's sandbox test needs. The design:
[`../00-design.md`](../00-design.md), sections 1 to 9.

## Steps

| #   | Step                                                        | Status      | Shared files                                                                         |
| --- | ----------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------ |
| 01  | [The crate, the texts and the categories](./01-crate.md)    | 🔨 in progress | `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md`                     |
| 02  | [The decision](./02-decision.md)                            | 🔨 in progress | `docs/restrictions/deletion-safety.md`                                               |
| 03  | [Fixtures, golden and the contract](./03-fixtures.md)       | 🔨 in progress | `Cargo.lock`, `docs/dependencies.md`                                                 |

In order: 01, then 02, then 03.

## Conclusions

The module is built and reviewed: 66 tests, `modules/logs/src` at 99.64 % of lines. One change from the design, in
01/02's fix round (a top-level file shaped as a bundle id is offered). See [the global readme](../readme.md#conclusions).
