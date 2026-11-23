# 013 — Release and distribution

**Status:** 📝 draft · **Version:** 1.0.0 · **Date:** 2026-11-15 · **Modules:** repo, ci, app

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

How a build becomes something a person can download, trust and run: signed, notarized, versioned, with the licences of
its dependencies inside — and with no network access of its own. Deferred from plan
[001](../001-repository-setup/00-design.md) (agent's call 11); drafted now because signing and the minimum macOS version
constrain plans [003](../003-base-ui/readme.md) and [006](../006-permissions/readme.md).

## Goals

1. One download, opened without security warnings, that runs on every supported Mac.
2. A release can be verified: built from a tagged commit with locked dependencies, with published checksums.
3. The app still makes no network request — including for updates.
4. Permissions granted by the user survive an update.

## Non-goals

- The Mac App Store: the app cannot be sandboxed ([plan 006](../006-permissions/readme.md)).
- Automatic updates.

## Design

- **Signing and notarization**: a Developer ID signature, the hardened runtime with no extra entitlements, notarized and
  stapled, so the app opens offline without a warning.
- **A stable identity**: the same bundle id and signing identity in every release — macOS ties granted permissions to
  them. Development builds use a separate id (plan [007](../007-safe-development/readme.md)) and a local signing
  identity, so their grants also survive a rebuild.
- **The package**: a disk image with the app, for Apple silicon and Intel in one universal build.
- **Supported systems**: macOS 13 and later; the release checklist runs the app on the oldest and the newest supported
  version.
- **Updates without the network**: the app never checks. The About section shows the version and where new versions are
  published; opening that page is the user's click, in their browser.
- **One official place to download**, named in the README; every release lists its checksums.
- **Versions and the changelog**: semantic versions; the changelog is generated from the Conventional Commits.
- **Licence notices**: generated at build time from the Rust and npm dependency trees and shown in About — this closes
  [`deferred/licence-notices.md`](../../deferred/licence-notices.md).
- **The release workflow**: a tag starts a CI job that builds with locked dependencies, signs, notarizes, attaches the
  image and checksums to a draft release; a person publishes it after the checklist.
- **Migrations are tested per release**: the app is started on data folders written by earlier versions (the file
  versions of plan [002](../002-base-architecture/readme.md)).

## Open questions

1. **An Apple Developer account** (a paid yearly membership) is required for a Developer ID signature and notarization.
   Without it a build runs only after the user overrides a security warning. This costs money and belongs to the user's
   decision; everything else in this plan can be prepared before it.
2. **Whose name signs the app**: an individual's or an organisation's — it is shown to users by macOS.
3. **The name and the app id before the first release**: a trademark search for "Fresher" (USPTO, EUIPO) and a domain
   the project owns (for example `getfresher.app`, free on 2026-11-15); the app id moves from
   `io.github.alexsergey.fresher` to that domain before any user has data under it.

## Plans (after approval)

| #   | Plan                    | What                                                                  |
| --- | ----------------------- | --------------------------------------------------------------------- |
| 01  | Identity and signing    | bundle ids, the local development identity, release signing, notarization |
| 02  | Package                 | the universal build, the disk image, the supported-systems check      |
| 03  | Versions and notices    | versioning, the changelog, the licence notices in About               |
| 04  | Release workflow        | the tag-driven CI job, checksums, the checklist, the migration test   |

The local development identity (part of 01) is useful as soon as plan 006 is built; the rest waits for the first
release. Depends on plan 001/05 (CI).

## Ideas this touches

"One official download location; signed and notarized builds" from [best practices](../../ideas/best-practices.md); the
deferred [licence notices](../../deferred/licence-notices.md). The release page carries the agreement's words from
`core.agreement.*` (plan [039](../039-agreement-and-wording/readme.md)).

## Decisions

None yet.

## Agent's calls (to check)

1. **No update check of any kind**; the user downloads new versions themselves.
2. **A universal build** for Apple silicon and Intel.
3. **macOS 13 stays the minimum.**
4. **CI signs and notarizes; a person publishes.**
5. **A local signing identity for development builds.**
6. **Target version 1.0.0** for the first signed public release.

## Conclusions

Not written yet.
