# 03 — Versions and notices

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, ci, scripts, app/ui, docs

The licences of everything the app is built from, shown in About and checked on every push; the version and the
changelog of a release. Design: [`../00-design.md`](../00-design.md), "The licence notices".

## Steps

| #   | Step                                           | Status      | What                                                                    |
| --- | ---------------------------------------------- | ----------- | ----------------------------------------------------------------------- |
| 01  | [The licence notices](./01-licence-notices.md) | 🔨 in progress (awaiting the user's look) | generator, check, the About dialog, pre-push and CI          |
| 02  | [Versions and the changelog](./02-versions-and-changelog.md) | 🔨 in progress (built, awaiting the user's look) | one version in every place, its check and its bump; the changelog |

## Conclusions

Both steps are built (2026-11-25) and wait for the user's look: the licence notices in About, and one version in every
place with `npm run check:version` and `npm run release:version`, and the changelog's 0.1.0 entry. Each step's file
holds its own conclusions.
