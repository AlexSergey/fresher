# Stale project cleanup

**Since:** 2026-11-15 · **Plan:** [021](../plans/021-stale-projects/readme.md) (draft)

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
