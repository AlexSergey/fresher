# 036/01 — Footprint

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, macos, system, engine, testing, app

What an installed app takes and what of it may go: the core's types, texts and categories; the apps folder's
protection rule; the bundle's own facts (size, entry facts, Spotlight's date, a package manager); the engine's list and
page over the last scan; the commands. Nothing is removed in this plan. The design: [`../00-design.md`](../00-design.md),
sections 1 to 6, 11 and 12.

## Steps

| #   | Step                                                            | Status      | Shared files                                                                                    |
| --- | --------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------- |
| 01  | [The core's types, texts and categories](./01-core-types.md)    | 🔨 awaiting the walk | `crates/core/src/{lib,error}.rs`, `crates/core/src/system/apps.rs`, `crates/core/locales/en.json` |
| 02  | [The apps folder in the protection](./02-protection.md)         | 🔨 awaiting the walk | `crates/core/src/cleanup/protection.rs`, `crates/macos/src/protection.rs`                       |
| 03  | [The bundle's facts](./03-bundle-facts.md)                      | 🔨 awaiting the walk | `crates/system/src/{apps,lib}.rs`, `crates/testing/src/{system_fakes,stated_apps}.rs`, `app/tauri/src/{compose,sandbox}.rs` |
| 04  | [The list, the page and the commands](./04-list-and-page.md)    | 🔨 awaiting the walk | `crates/engine/src/{engine,lib,results}.rs`, `crates/engine/src/storage/attribution.rs`, `app/tauri/src/lib.rs`, `bindings.generated.ts` |

01 first; then 02 and 03 in parallel (no shared file); then 04.

## Conclusions

The footprint is built as `01/01` to `01/04` say: the core's types and texts, the apps folder's protection, the bundle's facts and the list with the page. Nothing in the cleaner or the modules changed. Reviews found, and the steps fixed, an incomplete installed list, a second measure, links and case variants of a bundle's name.
