# 009/05 — Projects

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine

Projects found during the one shared walk and handed to every module's `finish`. The design:
[`../00-design.md`](../00-design.md), section 5.

## Steps

| #   | Step                                        | Status      | Shared files                   |
| --- | ------------------------------------------- | ----------- | ------------------------------ |
| 01  | [Projects from the walk](./01-collector.md) | ✔️ done     | `crates/engine/src/session.rs` |

## Conclusions

Built with one change: the artifacts are found by their own `FolderName` interests, since a marker's sibling names
are read beside the matched folder. The projects reach every module's `finish`; a project's last activity is its
folder's own time ([deferred](../../../deferred/project-last-activity.md)).
