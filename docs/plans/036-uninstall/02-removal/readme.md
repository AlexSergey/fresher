# 036/02 — Removal

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** core, macos, system, engine, app

The two choices as one confirmed item: the uninstall plan, the context that lets it reach the apps folder (and keeps
every other plan out of it), the findings and apps that leave with what went, and the App Management permission
learned from the first uninstall. The cleaner does not change. The design: [`../00-design.md`](../00-design.md),
sections 6 to 8.

## Steps

| #   | Step                                                  | Status      | Shared files                                                                                          |
| --- | ----------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [The uninstall plan](./01-uninstall-plan.md)          | 🔨 awaiting the walk | `crates/engine/src/cleanup/{context,operation,pending}.rs`, `crates/engine/src/{results,engine}.rs`, `app/tauri/src/{apps_commands,lib}.rs` |
| 02  | [App Management](./02-app-management.md)              | 🔨 awaiting the walk | `crates/core/src/access.rs`, `crates/core/locales/en.json`, `crates/macos/src/privacy/panes.rs`, `crates/system/src/{access,system_ui}.rs`, `crates/engine/src/{access,engine}.rs` |

01, then 02.

## Conclusions

The uninstall plan and the engine's second wall, and App Management learned passively. The cleaner is unchanged; the two judgements (prepare and confirm) are the engine's.
