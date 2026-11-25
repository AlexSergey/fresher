# 013 — Release and distribution

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, ci, scripts, app, app/ui, docs

Drafted on 2026-11-15 as one file; written as a global plan on 2026-11-25 from the user's answers of that day (see
"Decisions"). Approved by the user for the licence notices, the version 0.1.0, the unsigned first release and the app
id; the other plans are written when they are due. The design: [`00-design.md`](./00-design.md).

## Intent

How a build becomes something a person can download, trust and run: versioned, with the licences of its dependencies
inside, with no network access of its own — and, later, signed and notarized. Deferred from plan
[001](../001-repository-setup/00-design.md) (agent's call 11).

The user's task of 2026-11-25 (translated): "The first release is 0.1.0, for testers. It ships unsigned: no Apple
Developer account for now, signing comes later. The app id stays. Do the licence notices now: ship the licences of
every Rust and npm dependency in the app."

## Goals

1. A 0.1.0 download for testers that runs on every supported Mac; the release notes say how to open an unsigned app.
2. A release can be verified: built from a tagged commit with locked dependencies, with published checksums.
3. The app still makes no network request — including for updates.
4. The app carries the licence notices of everything it is built from, and a licence outside the allowlist fails the
   checks, for npm as for Rust.
5. Later: a signed and notarized build that opens without a warning, and grants that survive an update.

## Non-goals

- The Mac App Store: the app cannot be sandboxed ([plan 006](../006-permissions/readme.md)).
- Automatic updates.
- A signed 0.1.0: signing and notarization wait for the user's Apple Developer account.

## Plans

| #   | Plan                                                        | Status         | What                                                                                     |
| --- | ----------------------------------------------------------- | -------------- | ---------------------------------------------------------------------------------------- |
| 01  | Identity and signing                                        | ⏸ hold         | the local development identity, release signing, notarization: waits for the account     |
| 02  | Package                                                     | 📝 draft       | the universal build, the disk image, the supported-systems check                         |
| 03  | [Versions and notices](./03-versions-and-notices/readme.md) | 🔨 in progress | the licence notices in About (built); versioning and the changelog (later)               |
| 04  | Release workflow                                            | 📝 draft       | the tag-driven CI job, checksums, the checklist, the release notes, the migration test   |

Plans 02 and 04 and the rest of 03 get their files when the first release is prepared; 01 when the user has the
account. Depends on plan 001/05 (CI).

## Ideas and deferred items this touches

- "One official download location; signed and notarized builds" from [best practices](../../ideas/best-practices.md).
- Closed by 03/01: the deferred licence notices and the known issue "No licence check of npm dependencies yet" (both
  removed on 2026-11-25).
- Open: [release versioning](../../deferred/release-versioning.md) (the numbering is decided, the automation is not);
  [the README at the first release](../../deferred/readme-at-release.md).
- The release page carries the agreement's words from `core.agreement.*` (plan
  [039](../039-agreement-and-wording/readme.md)).

## Decisions

The user, 2026-11-25 (translated):

1. **The first release is 0.1.0**, for testers, not 1.0.0. 1.0.0 comes after real testing.
2. **0.1.0 ships unsigned**: no Apple Developer account for now; signing and notarization come later (plan 01 is on
   hold). The release notes tell users how to open it: try to open Fresher once, then System Settings → Privacy &
   Security → "Open Anyway" next to the line about Fresher, and confirm. The README is left as it is for now (the
   user postponed it); only this plan records the instruction until plan 04 writes the release notes.
3. **The app id stays `io.github.alexsergey.fresher`**: no own domain. Open question 3 is closed; no trademark search
   for 0.1.0.
4. **Do the licence notices now**: ship the licences of every Rust and npm dependency in the app (plan 03/01).

Open question 1 (an Apple Developer account) is answered "not now"; open question 2 (whose name signs the app) is
postponed with signing.

## Agent's calls (to check)

1. **No update check of any kind**; the user downloads new versions themselves.
2. **A universal build** for Apple silicon and Intel.
3. **macOS 13 stays the minimum.**
4. **CI builds; a person publishes.** Once signing exists, CI signs and notarizes too.
5. **A local signing identity for development builds**, with plan 01.
6. **Target version 0.1.0** for the first, unsigned release for testers (was 1.0.0 for a signed public one).
7. **The notices are committed, not made at build time**, with a freshness check in pre-push and CI (03/01,
   [`00-design.md`](./00-design.md), "The licence notices").

## Conclusions

Not written yet.
