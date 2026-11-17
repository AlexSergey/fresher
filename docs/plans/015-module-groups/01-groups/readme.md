# 015/01 — Groups

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
engine, modules, app/ui

Every module names its group; the UI shows modules under their group's name. The design:
[`../00-design.md`](../00-design.md), sections 1 and 2.

## Steps

| #   | Step                                                       | Status                                  | Shared files                                                                                 |
| --- | ---------------------------------------------------------- | --------------------------------------- | -------------------------------------------------------------------------------------------- |
| 01  | [The group in the descriptor](./01-group-in-descriptor.md) | ✔️ done | `crates/core/src/{lib,module,command,texts}.rs`, `crates/core/locales/en.json`, `modules/{demo,trial}/` |
| 02  | [Modules under their group](./02-grouped-screens.md)       | ✔️ done | `app/ui/src/screens/`, `app/ui/src/locales/en.json`                                          |

In order.

## Conclusions

Groups are a required part of the descriptor and every screen that lists modules shows them by group. No change from
the design. Parked minors went to the final wave of the build (tests of the settings grouping, a dead branch, a
fallback name for an unregistered group).
