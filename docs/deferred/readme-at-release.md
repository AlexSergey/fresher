# The README at the first release

**Agreed:** 2026-11-22, by the user's delegation (plan [040](../plans/040-readme/readme.md)) ·
**Due at:** when plan 013 publishes the first release

## What is postponed

The README and its neighbours say "pre-release", because there is no release to point at. At the first release:

- **The Install block** becomes the download link to the latest release, the package manager's one line, the lowest
  macOS version and the first-launch note.
- **The `status: pre-release` badge** becomes the release badge (`https://img.shields.io/github/v/release/AlexSergey/fresher`).
- **The social image** (`assets/social-preview.svg`, rendered to the PNG) is rendered again if the head changed.
- **"Supported versions" of `.github/SECURITY.md`** names the release.

## Why

Until a release exists a download link or a version badge would be false, and the README states only what is true
([`../plans/040-readme/00-design.md`](../plans/040-readme/00-design.md)).
