# Stale project cleanup

**Since:** 2026-11-15 · **Plan:** [021](../plans/021-stale-projects/readme.md) (built)

## Problem

Old projects keep gigabytes of regenerable build output and dependencies: `node_modules`, Cargo's `target/`, and the
like. A project not touched for a year still holds all of it, and nobody remembers it is there.

## Direction

- Scan the user's project folders (configurable in [`user-settings.md`](./user-settings.md)) and recognize a project
  by its manifest: `package.json`, `Cargo.toml`, and later others (`build.gradle`, `Podfile`, `pyproject.toml`, …).
- For each project, find the regenerable parts next to its manifest:
  - Node: `node_modules`, framework build caches (`.next`, `.nuxt`, `.turbo`, `.parcel-cache`, `dist` only when the
    project's config says it is build output);
  - Rust: `target/`;
  - others later: `.venv`, `build/`, `.gradle`, `Pods`, …
- **Staleness by date:** a project whose last activity is older than the threshold (default 1 year, customizable in the
  settings) is offered for cleanup. Only the safe, regenerable parts are offered — never the source code, never the
  project folder itself.
- "Last activity" is the newest modification time of the project's own files (excluding the regenerable folders
  themselves), and the last commit date when it is a git repository.
- The finding names the project, its path, its last activity date and what each part costs to restore (`npm install`,
  `cargo build`).
- Danger level: Low by default (restored by a reinstall or a rebuild, at a cost of time and bandwidth); a project
  touched recently is not offered at all.

Open questions:

- A project with uncommitted changes — skip it, or offer with a warning?
- Monorepos and nested `node_modules`: count once at the project root, list the nested ones inside the finding.
- Should "recently active but huge" projects be shown as information only (no cleanup offered)?

## Built by plan 021

The module `stale-projects` offers the artifact folders of projects nothing has changed in for a year (a setting, 30 to
3650 days): packages (`node_modules`), build output (`target`, `build`, `.gradle`, `.build`, `.dart_tool`), Python
environments (`.venv`, `venv`, Medium), and, for any Rust project, the build cache unused for 7 days. The last
activity is the newest folder time outside the artifact folders and `.git`, or the newest folder time of
`.git/refs/heads`; git is never run. A monorepo is one group; a project that could not be read in full offers
nothing. Known limits: [a file edited in place](../known-issues/core-project-activity-misses-edits-in-place.md)
and [a project cleaned once looks active](../known-issues/core-cleaned-stale-project-looks-active.md).

Decided by the user on 2026-11-19: `dist`, `.next`, `.nuxt`, `.turbo`, `.parcel-cache` and any other framework output
are never offered, not even later (so the "`dist` only when the config says it is build output" idea above is dropped).

Stays an idea:

- other kinds and artifacts (`Pods`, `build/` of other build tools, a project's own caches);
- single stale files inside a live `target/deps`;
- the editors that save in place, if the run by hand (plan 021 step 04/02) finds any;
- a project with uncommitted changes shown with a warning, and "recently active but huge" as information only (the
  module keeps them, with the reason, and never offers them).
