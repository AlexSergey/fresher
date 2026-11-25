# 03/02 — Versions and the changelog

**Status:** 🔨 in progress (built, awaiting the user's look) · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, ci, scripts, docs

Written and approved in chat on 2026-11-25 ("fill in the changelog"; see the global plan's "Decisions"). Resolves the
deferred "Release versioning" (removed).

## What

- **One version everywhere**, 0.1.0: the workspace's `Cargo.toml`, `package.json`, `app/ui/package.json`,
  `app/tauri/tauri.conf.json`, and the workspace's own entries of `package-lock.json` (the root, `packages[""]`,
  `packages["app/ui"]`) and `Cargo.lock` (every package without a `source`).
- **`scripts/release-version.ts`**, no new dependency:
  - `npm run check:version` fails when a place differs from the workspace's `Cargo.toml`; in the pre-push hook and
    CI's `lint` job.
  - `npm run check:version -- X.Y.Z` also wants that version everywhere, a `## [X.Y.Z] - YYYY-MM-DD` section in the
    changelog and `docs/releases/X.Y.Z.md`; the release workflow runs it with the tag's version.
  - `npm run release:version -- X.Y.Z` writes the version into the manifests (only the top-level `"version"` of a
    JSON file, only `version` of `[workspace.package]`), then refreshes the locks offline: `npm install
    --package-lock-only --ignore-scripts --offline` and `cargo update --workspace --offline`, which change only the
    workspace's own entries; it ends with the check.
  - Only `MAJOR.MINOR.PATCH` is accepted, as the tags are `v*.*.*`.
- **The changelog**, [`docs/changelog.md`](../../../changelog.md): documents live in `docs/`
  ([`restrictions/repository.md`](../../../restrictions/repository.md)), and no rule allows a root `CHANGELOG.md`.
  Newest first, an "Unreleased" section, one section per version with its date, links to the tags at the bottom;
  written by hand for the people who use the app, grouped by what they care about. The 0.1.0 entry is drawn from the
  goal, the categories, the plans' readmes and the history of `main`, and says only what is built (High withheld, the
  Largest screen hidden, "Remove everything" hidden).

## Files

`scripts/release-version.ts` and its test, `package.json` (`check:version`, `release:version`, `pre-push`),
`.github/workflows/ci.yml` (`lint`), `docs/changelog.md`, `docs/quality.md`, `docs/workflow.md` ("Releases"), the
deferred index.

## Checks

- `node --test scripts/release-version.test.ts`: a malformed version, a place with another version (every kind of
  place), a release without its changelog section or notes, a lock left behind are refused; the same version
  passes, and a new version is written everywhere with the two lock commands.
- On the repository: `npm run release:version -- 0.1.1` changed exactly the four manifests and the 26 workspace
  entries of `Cargo.lock` and three of `package-lock.json`, offline; `-- 0.1.0` restored them byte for byte.

## Conclusions

Built on 2026-11-25. No release tool was added: the deferred item's candidate would have generated the changelog from
the commits, whose words are the developers' ("feat(app): plan 042 — a plain interface"), not the users'; the bump
is one small tested script with no dependency.
