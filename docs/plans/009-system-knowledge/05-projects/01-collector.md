# 009/05/01 — Projects from the walk

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, engine

## Files

- `crates/core/src/ids.rs` — `ModuleId::system()`, the reserved owner of the engine's own interests.
- `crates/engine/src/projects.rs` (new) — the interests (one `MarkerFile` per marker, one `FolderName` per artifact
  name; see the conclusions) and the collector: markers of one folder merged, projects inside another's artifact folder dropped,
  sorted by path.
- `crates/engine/src/session.rs` — the interests added under `ModuleId::system()`, their matches to the collector,
  the projects set on the scan's `SystemFacts` before `finish`.

## Checks

- Negative: a package inside `node_modules` is not a project; no module receives a match of the system's interests;
  during the walk `projects()` is `None`.
- Positive: a fixture with a Node and a Rust project and a mixed one gives three projects with their kinds, artifacts
  and last activity, in `finish`.

## Conclusions

- Built as written, with one change found while building: a `MarkerFile` match reports sibling names **beside** the
  matched folder (from its parent's listing), not inside it, so the artifacts cannot come as siblings. The engine adds
  one `FolderName` interest per artifact name (eight names) beside the marker interests (fourteen), all `Claim::Item`,
  and the collector keeps an artifact folder only when its parent is a project of a kind that makes it.
- A project is dropped when it lies inside another project's artifact folder (`tool/target/package/tool-1.0`) or
  inside any `node_modules` (installed packages, even without a project around them). A folder named like a marker
  (`package.json/`) is not a marker. Markers of one folder merge into one project, kinds in `ProjectKind::ALL` order.
- `ModuleId::system()` is the core's own namespace, which `ModuleId::new` already refuses to a module; the session's
  interests are built by `all_interests`, the matches of `ModuleId::system()` go to the collector and never to a
  module, and the projects are set on the scan's `SystemFacts` right after the walk, before any `finish`.
- The added interests change the fingerprint of every scan cache: the first scan after this change is full.
- Checks: `projects.rs` in the engine (the collector) and `crates/e2e/tests/projects.rs` (a fixture home through the
  engine and the reference walk: two projects in `finish`, none during the walk, the module gets only its own
  matches).
