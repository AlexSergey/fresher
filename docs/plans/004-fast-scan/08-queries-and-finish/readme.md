# 004/08 — Queries and finish

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner, engine, docs, app

The read side of the tree — the largest files and search by name — and the engine's API over it for plans 003, 006
and 012; then the budget run that measures every bar on the final code and on a real home folder, the docs, the
version, and the conclusions. The design: [`../00-design.md`](../00-design.md), sections 11 and 13.

Depends on 07. In order.

## Steps

| #   | Step                                                   | Status         | Shared files                              |
| --- | ------------------------------------------------------ | -------------- | ----------------------------------------- |
| 01  | [Largest files and search](./01-largest-and-search.md) | ✔️ done        | `crates/scanner/benches/scan.rs`          |
| 02  | [The engine's queries](./02-engine-queries.md)         | ✔️ done        | `crates/engine/`                          |
| 03  | [The budget run](./03-budget-run.md)                   | 🔨 in progress  | `docs/plans/004-fast-scan/`               |
| 04  | [Docs and version](./04-docs-and-version.md)           | ✔️ done        | all docs, `CLAUDE.md`, the version files  |

## Conclusions

The plan stays 🔨 in progress until the user's run by hand of 03 is filled in and looked at; the rest is done.

- **The read side** (01): `TreeState::largest_files` (one pass in path order into a bounded heap; a hard link counted
  at another path skipped) and `TreeState::search` (case ignored, a lowercase copy of the names built by the first
  search and counted in the state's heap bytes; the roots and links not searched). No index on disk, no new crate.
- **The engine's queries** (02): `disk_roots`, `disk_node`, `disk_children`, `largest_files`, `search_names`,
  `unreadable_places` over the last committed state, read outside the engine's lock (a query never blocks a scan);
  `NoScan` before any commit, `StaleNode` for an id of another state, every limit capped at 500. No IPC command yet
  (design call 32): plans 003 and 012 add them with their screens.
- **The budget run** (03): every bar of the design measured on the final code, on the developer's machine under other
  load; the table is in the [global plan's conclusions](../readme.md#measurements). Every deterministic bar passes in
  `npm run pre-push`; the timing bars pass (the warm full scan projected from `SMALL`, as `LARGE` cannot be warm on
  this machine). Two decisions taken as recommended (2026-11-16, to check): one thread per core (at most 16), and the
  tree-heap bar kept on its test, the 104 B a node of a tree of many matched files recorded. Added for the run: the
  `scan/restore` bench and the footprint test of ten scans (`crates/e2e/tests/footprint.rs`).
- **Docs and version** (04): the architecture, testing and quality docs as built, the ideas files, version 0.3.0.
- **Calls taken** (each in its step): the scan roots and symbolic links are not searched, ties go to the earlier path
  (01); `disk_roots` answers the roots' nodes, an unknown id of the last state is `StaleNode` (02); the `NoWalk` test
  stubs and the reference walk's `holds_valid_tag` keep their names (04).
- **For the user**: the run by hand of 03 (its checklist, also in the global plan's conclusions).
