# 009/01 — Known folders

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, scanner, rules, testing, system, app, e2e

A module names a place by a known folder, never by a macOS path; the walk resolves it through the `KnownFolders`
port. The design: [`../00-design.md`](../00-design.md), sections 1 and 2.

## Steps

| #   | Step                                                         | Status      | Shared files                                       |
| --- | ------------------------------------------------------------ | ----------- | -------------------------------------------------- |
| 01  | [The port and the pattern](./01-port-and-pattern.md)         | ✔️ done     | `crates/core/src/interest.rs`, every walk test     |
| 02  | [The crate and the macOS folders](./02-crate-and-folders.md) | ✔️ done     | root `Cargo.toml`, `scripts/check-crate-deps.ts`   |

In order.

## Conclusions

Both steps built as written; the known folders replace the home-relative pattern everywhere, and the app's scanner
anchors come from `MacKnownFolders`. The fingerprint names the folder, so every scan cache written before is discarded
once (one full scan).
