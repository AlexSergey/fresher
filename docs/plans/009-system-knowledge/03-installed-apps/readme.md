# 009/03 — Installed apps

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** system, app

`MacInstalledApps`: every app bundle in the apps folders with its bundle id, team id, version and helpers, read from
its own `Info.plist` and code signature. The design: [`../00-design.md`](../00-design.md), section 3.

## Steps

| #   | Step                                            | Status      | Shared files                 |
| --- | ----------------------------------------------- | ----------- | ---------------------------- |
| 01  | [The `Info.plist` reader](./01-plist-reader.md) | ✔️ done     | `crates/system/src/lib.rs`   |
| 02  | [The team id reader](./02-signature-reader.md)  | ✔️ done     | `crates/system/src/lib.rs`   |
| 03  | [The apps](./03-installed-apps.md)              | ✔️ done     | `app/tauri/src/compose.rs`   |

01 and 02 in parallel (pure readers), then 03.

## Conclusions

Built as written; 01 and 02 were written in parallel by two subagents in scratch crates with the workspace's lints,
then moved in and committed with 03, their first user. Both readers were checked against real files by hand (the
system's own property list writer; five third-party apps' team ids), and only made-up bytes are kept in the tests.
