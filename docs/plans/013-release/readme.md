# 013 — Release and distribution

**Status:** 🔨 in progress (built, awaiting the user's look) · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, ci, scripts, app, app/ui, docs

Drafted on 2026-11-15 as one file; written as a global plan on 2026-11-25 from the user's answers of that day (see
"Decisions"). Approved by the user for the licence notices, the version 0.1.0, the unsigned first release and the app
id; later that day for the changelog, the package, the release workflow and the release notes, written as one short
file per plan. Plan 01 waits for the Apple Developer account. The design: [`00-design.md`](./00-design.md).

Branch: `feat/013-release-workflow`, from `main` at 9aefa750 (the licence notices of 03/01 are already on `main`).

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
| 02  | [Package](./02-package/readme.md)                           | 🔨 built, awaiting the user's look | the universal build, the disk image, their check                     |
| 03  | [Versions and notices](./03-versions-and-notices/readme.md) | 🔨 built, awaiting the user's look | the licence notices in About; one version everywhere; the changelog  |
| 04  | [Release workflow](./04-release-workflow/readme.md)         | 🔨 built, awaiting the first tag   | the tag-driven CI job, checksums, provenance, the draft, the checklist, the release notes |

Plan 01 gets its files when the user has the account. Depends on plan 001/05 (CI).

## Ideas and deferred items this touches

- "One official download location; signed and notarized builds" from [best practices](../../ideas/best-practices.md).
- Closed by 03/01: the deferred licence notices and the known issue "No licence check of npm dependencies yet" (both
  removed on 2026-11-25).
- Closed by 03/02: the deferred "Release versioning" (removed on 2026-11-25).
- Open: [the README at the first release](../../deferred/readme-at-release.md), done by the README's own work; its
  `SECURITY.md` line was done here.
- The release page carries the agreement's words from `core.agreement.*` (plan
  [039](../039-agreement-and-wording/readme.md)).

## Decisions

The user, 2026-11-25 (translated):

1. **The first release is 0.1.0**, for testers, not 1.0.0. 1.0.0 comes after real testing.
2. **0.1.0 ships unsigned**: no Apple Developer account for now; signing and notarization come later (plan 01 is on
   hold). The release notes tell users how to open it: try to open Fresher once, then System Settings → Privacy &
   Security → "Open Anyway" next to the line about Fresher, and confirm. Later the same day the user asked for the
   instruction in the README too, as a temporary "First launch on macOS" section under Install; the rest of the
   README stays postponed. Removing the quarantine attribute is never suggested.
3. **The app id stays `io.github.alexsergey.fresher`**: no own domain. Open question 3 is closed; no trademark search
   for 0.1.0.
4. **Do the licence notices now**: ship the licences of every Rust and npm dependency in the app (plan 03/01).

Open question 1 (an Apple Developer account) is answered "not now"; open question 2 (whose name signs the app) is
postponed with signing.

The user, later on 2026-11-25 (translated), approving the remaining plans in chat:

5. **"Fill in the changelog"**: the changelog and the 0.1.0 entry (plan 03/02).
6. **"The release CI — go"**: the package and the release workflow (plans 02 and 04).
7. **"The release notes — go"**: the release notes of 0.1.0 (plan 04).

## Agent's calls (to check)

1. **No update check of any kind**; the user downloads new versions themselves.
2. **A universal build** for Apple silicon and Intel.
3. **macOS 13 stays the minimum.**
4. **CI builds; a person publishes.** Once signing exists, CI signs and notarizes too.
5. **A local signing identity for development builds**, with plan 01.
6. **Target version 0.1.0** for the first, unsigned release for testers (was 1.0.0 for a signed public one).
7. **The notices are committed, not made at build time**, with a freshness check in pre-push and CI (03/01,
   [`00-design.md`](./00-design.md), "The licence notices").
8. **No release tool**: a small tested script checks and sets the version (03/02); no new dependency.
9. **The changelog is written by hand for the people who use the app**, not generated from the commits, whose words
   are the developers'; it lives at `docs/changelog.md`, as every document lives in `docs/`.
10. **The release is built without caches** (a `cache` input of the setup action), from the tagged tree and the locked
    downloads only, and only from a commit on `main`; the dry run is the same build by hand, with no release.
11. **The build and the release are two jobs**: the macOS build holds only a read token; the Linux job that attests and
    makes the draft gets the write permissions and never runs the build's code.
12. **The draft is not marked as a pre-release**: GitHub's "latest release", which the README's download link and badge
    will name, skips pre-releases. "For testers" is said in the notes and the version; the person who publishes may
    still tick it.
13. **`npm run build:release` sets `CI=true`**, so Tauri's image script never drives Finder; the image has no custom
    window layout or background.
14. **The release notes ask to remove Fresher from the Full Disk Access list after an update** and turn it on when the
    app asks: an ad hoc signature ties the grant to that build, so the old entry no longer matches.
15. **`npm run check:version`** runs in pre-push and CI's `lint`, so the places never drift between releases.

## Conclusions

Built on 2026-11-25 on `feat/013-release-workflow`; awaiting the user's look, then the first tag.

- **Versions** (03/02): 0.1.0 in all nine kinds of place (26 crates in `Cargo.lock`); `npm run check:version` in
  pre-push and CI; `npm run release:version -- X.Y.Z` round-trips offline (0.1.1 and back, byte for byte).
- **The changelog** (03/02): `docs/changelog.md`, "Unreleased" and the 0.1.0 entry in seven groups, only what is on
  `main`: High findings, Largest and "Remove everything" are said to be held back.
- **The package** (02): the universal app (21.9 MiB, `x86_64 arm64`, ad hoc with the hardened runtime) and its image
  (9.6 MiB) built on the developer's Mac in under five minutes of compiling; `check:bundle -- --release` passed on
  both.
- **The release workflow** (04): `release.yml`, actionlint clean; the release checklist in `workflow.md`. Not run
  yet: only a pushed tag (or a dry run by hand) shows the runner's build, the attestation and the draft.
- **The release notes** (04): `docs/releases/0.1.0.md`, with the agreement's five lines and the first-launch steps
  (Open Anyway; the quarantine attribute is never mentioned).

For the user's look: the changelog's and the notes' wording, agent's calls 8 to 15, and the first dry run of the
workflow before the tag.
