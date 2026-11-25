# 013 — Design

**Status:** ✅ approved (the parts in "Decisions") · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, ci, scripts, app, app/ui, docs

## Intent

The user's task, goals and non-goals: [`readme.md`](./readme.md). This page holds the design the plans argue from.

## Design

- **Signing and notarization (later, plan 01)**: a Developer ID signature, the hardened runtime with no extra
  entitlements, notarized and stapled, so the app opens offline without a warning. Until then (0.1.0) the build is
  unsigned and the release notes say how to open it (System Settings → Privacy & Security → "Open Anyway").
- **A stable identity**: the same bundle id, `io.github.alexsergey.fresher`, in every release — macOS ties granted
  permissions to it (and, once signed, to the signing identity). Development builds use a separate id (plan
  [007](../007-safe-development/readme.md)).
- **The package (plan 02)**: a disk image with the app, for Apple silicon and Intel in one universal build; macOS 13
  and later, the release checklist runs the app on the oldest and the newest supported version.
- **Updates without the network**: the app never checks. About shows the version and where new versions are
  published, as plain text.
- **One official place to download**, named in the README when the first release is published
  ([`deferred/readme-at-release.md`](../../deferred/readme-at-release.md)); every release lists its checksums.
- **Versions and the changelog (plan 03, later)**: semantic versions from 0.1.0; the changelog from the Conventional
  Commits ([`deferred/release-versioning.md`](../../deferred/release-versioning.md)).
- **The release workflow (plan 04)**: a tag starts a CI job that builds with locked dependencies and attaches the
  image and checksums to a draft release; a person publishes it after the checklist. Migrations are tested per
  release on data folders written by earlier versions.

### The licence notices (plan 03/01)

- **What**: every licence text of the Rust crates the app crate (`app/tauri`) is built from — normal and build
  dependencies, not dev-dependencies, macOS targets only, not our own workspace crates — and of the UI's npm
  production packages (`package-lock.json` entries that are not `dev`). Parts with the same licence text are grouped;
  each group lists its parts by name and version.
- **Rust**: `cargo-about` (a machine tool, like cargo-deny) with `about.toml`, run offline (`--frozen`): the texts come
  from the crates in the local registry, so every machine with the same `Cargo.lock` writes the same file. Its
  `accepted` licences mirror `deny.toml`'s allowlist and MPL-2.0 exceptions; the script fails when they differ.
- **npm**: read from `node_modules`, no network: the package's declared SPDX licence must be satisfied by the
  allowlist (read from `deny.toml`; the MPL-2.0 exceptions are crates and do not apply), and the licence file for the
  chosen licence must be in the package's folder.
- **MPL-2.0**: the five crates of Tauri are listed apart, each with the address of its source on crates.io, as
  [`restrictions/dependencies.md`](../../restrictions/dependencies.md) requires.
- **Committed, with a freshness check**: `npm run notices:generate` writes
  `app/ui/src/screens/settings/notices.generated.json` (formatted by Prettier); `npm run check:notices`, in pre-push
  and CI's `lint` job, fails when the file differs or a licence is refused. A dependency change therefore carries its
  notices in the same commit; the file is reviewable, and building the app needs no licence tool.
- **Shown in the app**: Settings → About gets "Show the open-source licences", which opens a dialog: a line on what it
  is, the MPL-2.0 parts with their source addresses as selectable plain text (the app opens nothing), then one folded
  row per licence (name and how many parts) that opens to the parts and the full text. The file is loaded with a
  dynamic import when the dialog is first opened — a chunk of the app's own bundle, never the network — so the window
  does not parse it at start. The dialog is presentational (`product/notices-dialog`), takes the notices by props, and
  its gallery story uses a fixed sample, so its picture does not change with every dependency update.

## Open questions

None left for 0.1.0. Signing (an Apple Developer account and whose name signs) waits for the user.

## Decisions

See [`readme.md`](./readme.md), "Decisions" (the user, 2026-11-25).

## Agent's calls (to check)

See [`readme.md`](./readme.md), "Agent's calls (to check)". For the notices also:

1. **cargo-about over a hand-written reader of `cargo metadata`**: it resolves licence expressions and finds licence
   files the way the Rust ecosystem expects; it is already installed and pinned (0.9.3) like the other cargo tools.
2. **Build dependencies are listed too** (proc macros such as `cssparser-macros` shape the shipped code); listing a few
   more crates than strictly ship costs nothing.
3. **The source addresses are crates.io's version pages**, the exact source the build used.
4. **The notices are UI data, not IPC**: the text never changes at run time, so the core does not serve it.
