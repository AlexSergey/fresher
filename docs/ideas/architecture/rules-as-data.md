# Rules as data

**Since:** 2026-11-15

## Declarative rules

- Cleanup rules live in data files (TOML), versioned with the code; a rule ships only with its safety boundary written
  down and its fixture tests (rule 5 of [`../../rules.md`](../../rules.md)).
- A rule states: the category it produces, where to look, how to recognize the item, what must NOT match, its danger
  level, how the item comes back, and the action (move to Trash, or a tool's own command).

## Project artifacts

- A project is found by a **marker file**; each project type lists its artifact folders:
  `Cargo.toml` → `target`; `package.json` → `node_modules`, framework caches; Gradle → `build`, `.gradle`;
  `Package.swift`, `Podfile`, `pubspec.yaml`, …
- **Conditions** prevent false positives: `target` counts only next to `Cargo.toml` (or another build file that uses
  that name); `coverage` only if it holds a coverage report.
- `CACHEDIR.TAG` (a standard marker file) says "this folder is a cache" for any tool that writes it.
- A project's last activity is the newest modification time across its own files, the artifact folders excluded
  ([`../stale-project-cleanup.md`](../stale-project-cleanup.md)).

## Protected paths

- Checked on **physical paths**: resolve symlinks and aliases first, then compare; a symlink or alias must not lead
  around a protected root.
- Protect the **ancestors** of protected items too: deleting a parent folder deletes the protected child.
- Protect security-agent and device-management caches (touching them can trip tamper alarms) and an **explicit list**
  of critical system bundles, rather than every `com.apple.*` id.
- Items in use by a live owner (a running database, a container runtime, a notebook server) are skipped and reported.
- **Fail closed**: if the protection list cannot be read, nothing is cleaned.
- Write a threat model for the cleaner: symlinks and aliases, parents of protected leaves, live owners, time between
  check and removal.
