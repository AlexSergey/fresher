# Releases

How Fresher is released: what a release is, where it is published, how versions grow, and the steps to cut one. Every
release follows this document. The design and its decisions: plan [013](./plans/013-release/readme.md).

## Principles

- **One official place to download**: the repository's GitHub Releases page. No mirrors, no installers hosted
  elsewhere, no package manager until one is set up on purpose (then this document names it).
- **A person publishes.** CI builds, checks and drafts the release; the user checks the draft and publishes it. Nothing
  reaches users without the user's look (rule 10, [`rules.md`](./rules.md)).
- **No update check, no network.** The app never asks the internet whether a new version exists; users download new
  versions themselves ([`restrictions/privacy.md`](./restrictions/privacy.md)).
- **A release can be verified**: built from a tagged commit on `main` with locked dependencies and no build caches,
  with published SHA-256 checksums and a build provenance attestation.
- **A published release is never replaced.** A broken published release is fixed by the next patch version.

## Versions

- **Semantic versioning**, `MAJOR.MINOR.PATCH`, tags `vMAJOR.MINOR.PATCH`.
- **0.x is for testers.** The first release is 0.1.0; 1.0.0 comes only after real testing on people's Macs (the
  user's decision of 2026-11-25).
- While in 0.x: a release with new features or changed behaviour raises MINOR (0.2.0); a release with fixes only
  raises PATCH (0.1.1).
- **One version everywhere**: the workspace `Cargo.toml`, both `package.json`, `app/tauri/tauri.conf.json` and the
  lock files. `npm run check:version` (pre-push and CI) fails when they differ; `npm run release:version -- X.Y.Z`
  writes a new one into all of them.
- The version is changed only when a release is cut, never by plans.

## What a release holds

- **`Fresher_X.Y.Z_universal.dmg`**: one disk image for Apple silicon and Intel, macOS 13 or later
  (`npm run build:release`, checked by `npm run check:bundle -- --release`).
- **`SHA256SUMS`** of the image, and a **build provenance attestation** (`gh attestation verify`).
- **The release's text**: [`releases/X.Y.Z.md`](./releases/) (what it is, supported Macs, install, first launch, Full
  Disk Access, how to check the download, no warranty, where to report), followed by the version's section of
  [`changelog.md`](./changelog.md) under "What's new in X.Y.Z". `node scripts/release-version.ts notes X.Y.Z` prints
  it.
- The draft is not marked as a pre-release, so GitHub's "latest release" (the README's download link and badge) points
  at it.

## Signing

- **Today: ad hoc.** The app is signed as a whole with the hardened runtime and no entitlements (`signingIdentity:
"-"`), not with an Apple Developer ID and not notarized. macOS blocks the first launch of a downloaded copy; the
  README and the release notes tell users to open it once, then System Settings → Privacy & Security → "Open
  Anyway". **Users are never told to remove the quarantine attribute.** An ad hoc signature ties Full Disk Access to
  one build, so after an update the grant is turned on again; the release notes say so.
- **Later: Developer ID and notarization**, when the user has an Apple Developer account (plan 013/01, on hold): the
  certificate and the notarization key become CI secrets, `signingIdentity` names the Developer ID, the workflow
  notarizes and staples, and the first-launch instructions leave the README and the notes. The app id
  (`io.github.alexsergey.fresher`) does not change.

## The changelog

- [`changelog.md`](./changelog.md) is written for the people who use the app, in plain English, grouped by what they
  notice; it is not a list of commits. Its top section is "Unreleased".
- When a release is cut, the changes since the last tag (Conventional Commits: `git log vPREV..main`) are read and
  written into "Unreleased", which then becomes `## [X.Y.Z] - YYYY-MM-DD` with the day of the tag. Only what is on
  `main` and reachable in the app is listed.

## Cutting a release

1. **Version.** On `main`, `npm run release:version -- X.Y.Z` writes the version into every place and refreshes the
   lock files (nothing to do when the version already is X.Y.Z, as for 0.1.0).
2. **Changelog.** "Unreleased" becomes `## [X.Y.Z] - YYYY-MM-DD` with the day of the tag; the links at the bottom get
   the new version.
3. **Release notes.** `docs/releases/X.Y.Z.md`, from the previous one. `.github/SECURITY.md` names the supported
   versions.
4. **Check and commit.** `npm run check:version -- X.Y.Z` passes; the change is one commit on `main`
   (`chore(repo): release X.Y.Z`) through the hooks.
5. **Test cases by hand.** Every case of [`test-cases/`](./test-cases/README.md) is run on the release's commit and
   its result written in the column of the new version. A failed case is fixed (with an autotest where one can hold
   it) and run again before the tag; a case that fails because of a known issue is marked ⚠️ and the issue named.
6. **Dry run (optional).** Actions → Release → "Run workflow" on `main` builds the same files without a release and
   keeps them as the `release` artifact for 14 days.
7. **Push and wait.** `git push origin main`; CI passes on that commit (a tag is never the first build of its tree).
8. **Tag.** `git tag -a vX.Y.Z -m "Fresher X.Y.Z"` on that commit, then `git push origin vX.Y.Z`. The workflow
   (`.github/workflows/release.yml`) refuses a tag that is not on `main` or that differs from the version in any
   place, builds the image, writes `SHA256SUMS`, attests the provenance and makes the **draft** release.
9. **Check the draft.** On the Releases page: the image, `SHA256SUMS` and the text are there; download the image, run
   `shasum -a 256 -c SHA256SUMS` and `gh attestation verify <image> --repo AlexSergey/fresher`; install it as the
   notes say and open it on the oldest supported macOS (13) and the newest, on Apple silicon and on Intel where a Mac
   is at hand: the first launch, About's version, a scan. From the second release on, open the new version over the
   data folder of the previous one (its settings, ignore list and cleanups are kept).
10. **Publish** the draft. Then the README's release items of
   [`deferred/readme-at-release.md`](./deferred/readme-at-release.md) are due.

A broken draft is deleted with its tag (`git push --delete origin vX.Y.Z`), fixed on `main` and tagged again.
