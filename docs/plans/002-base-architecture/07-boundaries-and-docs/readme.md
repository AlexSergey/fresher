# 002/07 — Boundaries and docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** repo, docs

The dependency table of the design checked by a repository guard, and the documents brought in line with what plan 002
built: files instead of SQLite, the crates and the `modules/` folder, the module model. The design:
[`../00-design.md`](../00-design.md), "Crates".

Step 01 depends on plan 01 only and runs in wave 2 (it owns `scripts/check-crate-deps.ts`; the other wave-2 plans do
not touch `package.json` or `docs/quality.md`). Step 02 runs last, after every other plan.

## Steps

| #   | Step                                                | Status      | Depends on     | Shared files                                                                  |
| --- | --------------------------------------------------- | ----------- | -------------- | ----------------------------------------------------------------------------- |
| 01  | [The dependency guard](./01-dependency-guard.md)    | ✔️ done     | 002/01         | `package.json`, `docs/quality.md`                                             |
| 02  | [Docs and version](./02-docs-and-version.md)        | ✔️ done     | every other step | `docs/`, `CLAUDE.md`, `Cargo.toml`, `app/tauri/tauri.conf.json`, both `package.json`, `package-lock.json`, `Cargo.lock` |

## Conclusions

- **The guard** (01): `scripts/check-crate-deps.ts` reads `cargo metadata` and fails on an edge outside the design's
  table, on a module depending on another module, and on a crate that is not in the table; 19 script tests;
  `npm run check:crate-deps` runs in the pre-push hook and in CI.
- **The docs** (02): `goal.md` §5, `rules.md` §6, `architecture/README.md` (layers, crates, ports, data; links
  `modules.md`), `restrictions/privacy.md` and `repository.md`, `testing.md`, the `CLAUDE.md` map, the root `README.md`
  and `ideas/architecture/crates.md` describe plain files instead of SQLite and the crates that were built. The crate
  table matches the guard's constant (compared by hand).
- **Version 0.1.0** in `Cargo.toml`, `tauri.conf.json` and both `package.json`; the release bundle reports 0.1.0.
- Deviations: none from the steps; the details are in 02's notes.
