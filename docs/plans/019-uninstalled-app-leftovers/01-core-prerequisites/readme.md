# 019/01 — Core prerequisites

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, scanner, macos, system, engine, testing, modules, app, app/ui, docs

What the code lacks before a module can offer a removed app's data safely: a `*` that matches only bundle-id names,
what the walk saw of each interest's place, a check that Spotlight can see every mounted volume, a removed app's name
in the storage map, the group Apps and a module's selection choices. The design:
[`../00-design.md`](../00-design.md), section 1.

## Steps

| #   | Step                                                                     | Status      | Shared files                                                                                                  |
| --- | ------------------------------------------------------------------------ | ----------- | ------------------------------------------------------------------------------------------------------------- |
| 01  | [Interests that match only bundle-id names](./01-bundle-id-names.md)     | ✅ approved | `crates/core/src/{interest,system/ids,lib}.rs`, `crates/scanner/src/matcher/`, `crates/scanner/benches/scan.rs`, `crates/scanner/tests/reference.rs`, `crates/testing/src/naive_walk.rs`, `docs/architecture/modules.md` |
| 02  | [What the walk saw of each place](./02-places-seen.md)                   | ✅ approved | `crates/core/src/{walk,module,lib}.rs`, `crates/scanner/src/{scanner.rs,places_seen.rs}`, `crates/engine/src/session.rs`, `crates/testing/src/{naive_walk,run_module}.rs`, `docs/architecture/modules.md` |
| 03  | [Spotlight's reach](./03-spotlight-reach.md)                             | ✅ approved | `crates/core/src/system/`, `crates/macos/src/{volume,lib}.rs`, `crates/system/src/{apps,spotlight,lib}.rs`, `crates/testing/src/system_fakes.rs`, `app/tauri/src/{compose,sandbox}.rs`, `modules/jetbrains/{locales/en.json,src/tests.rs}`, `docs/known-issues/` |
| 04  | [A removed app's name, the group Apps, the choices](./04-names-group-choices.md) | ✅ approved | `crates/core/src/{finding,group,module,command,lib}.rs`, `crates/core/locales/en.json`, `crates/engine/src/storage/`, `crates/testing/src/contract.rs`, `modules/jetbrains/src/lib.rs`, `fixtures/golden/jetbrains.json`, `app/ui/src/{ipc/bindings.generated.ts,product/module-groups.ts}`, `docs/deferred/` |

01, 03 and 04 in parallel, first; 02 after 01 (both touch `naive_walk.rs` and the scanner). Each step writes its own
part of `docs/architecture/modules.md`; 04/01 checks them together.

## Conclusions

Built as 01/01 to 01/04 say: a `*` that matches only bundle-id names, what the walk saw of each place, a check that
Spotlight sees every mounted volume, a removed app's name, the group Apps and a module's choices. What changed from the
design: a link at a place reads `Absent` (01/02), the home's volume is asked by `statfs` (01/03). Known issues taken:
a link at a place, an unindexed volume, nested apps and the privacy list, the reference walk's avoided paths. The scan
bench of 01/01 is to be retaken on a quiet Mac (04/02).
