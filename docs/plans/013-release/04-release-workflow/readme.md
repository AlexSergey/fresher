# 04 — Release workflow

**Status:** 🔨 in progress (built, awaiting the first tag and the user's look) · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** ci, repo, docs

Written and approved in chat on 2026-11-25 ("the release CI — go", "the release notes — go"; see the global plan's
"Decisions"). Design: [`../00-design.md`](../00-design.md), "The release workflow". One file: the plan is one step.

## What

- **`.github/workflows/release.yml`**, on a pushed tag `v*.*.*`, or by hand (`workflow_dispatch`) as a dry run.
  Conventions of `ci.yml`: actions pinned to full SHAs, `persist-credentials: false`, the shared setup action, locked
  installs; `permissions: {}` at the top, each job asks for its own.
  - **`build`** (macOS, `contents: read`): the whole history; the setup with its caches off (a new `cache` input of
    `.github/actions/setup`), so a release is built only from the tagged tree and the locked downloads; on a tag, the
    tagged commit must be on `main`; `node scripts/release-version.ts check <version>` (the tag's version, or
    `package.json`'s in a dry run); `npm run test:scripts`; `npm run build:release` (plan 02, with its check);
    `SHA256SUMS` of the image; the image, the checksums and the release notes kept as the `release` artifact.
  - **`release`** (Linux, tag pushes only; `contents: write`, `id-token: write`, `attestations: write`): the files
    checked against `SHA256SUMS`, the image's build provenance attested (`actions/attest-build-provenance`), and a
    **draft** release made with `gh release create --draft --verify-tag`, the image and `SHA256SUMS` attached and
    `docs/releases/<version>.md` as its text, followed by the version's changelog section under "What's new in
    <version>" (the user, 2026-11-25: "in most repositories the version's changelog goes into the release; let's do
    the same"). A person publishes it (agent's call 4).
- **The release checklist**, [`docs/release.md`](../../../release.md): version, changelog, notes, check,
  dry run, push, tag, check the draft (checksums, attestation, the oldest and newest macOS, Intel), publish; a broken
  draft is deleted with its tag; from the second release on, the new version opens the previous one's data folder
  (the migration check of the design; 0.1.0 has no earlier release).
- **The release notes of 0.1.0**, [`docs/releases/0.1.0.md`](../../../releases/0.1.0.md): what Fresher is and that
  0.1.0 is for testers, the supported Macs, install, the first launch (Open Anyway, never the quarantine attribute),
  Full Disk Access and why it is lost at each update until the app is signed, how to check the download, the
  agreement's five lines (the texts `core.agreement.line.*`, in the app's order), where to report, the changelog.
  Links are absolute: the notes are the text of a GitHub release.
- **`.github/SECURITY.md`**: the supported versions are 0.1.x and `main`.

## Checks

- `actionlint` (with shellcheck) passes on every workflow; Prettier passes on the YAML.
- What only a pushed tag can show: the macOS runner's universal build and image, the attestation, the draft. A dry
  run (Actions → Release → Run workflow) shows the build part without a release.

## Conclusions

Built on 2026-11-25 and checked locally as far as a workflow can be: actionlint, the version check with 0.1.0, the
release build and its check on the developer's Mac (plan 02). The first tag is the user's.
