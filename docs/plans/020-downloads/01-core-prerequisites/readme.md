# 020/01 — Core prerequisites

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 ·
**Modules:** core, scanner, system, engine, testing, modules, e2e, app, app/ui, docs

What the code lacks before a module can read Downloads safely and usefully: the day setting once in the core (a
deferred item), a `*` that matches only names with a suffix, an installed app's change time with the installer's
receipts and the system's version, a module that needs a guarded folder running in the scan that first tries it, and
the group **Your files**. The design: [`../00-design.md`](../00-design.md), section 1.

## Steps

| #   | Step                                                                                   | Status      | Shared files                                                                                                  |
| --- | -------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------- |
| 01  | [The shared day setting](./01-day-setting.md)                                          | ✔️ done | `crates/core/src/{settings,lib}.rs`, `crates/core/src/settings/days.rs`, `modules/{jetbrains,app-leftovers,temp}/src/`, `docs/architecture/modules.md`, `docs/deferred/` |
| 02  | [Interests that match only names with a suffix](./02-suffixed-names.md)                | ✔️ done | `crates/core/src/{interest,lib}.rs`, `crates/scanner/src/matcher/`, `crates/scanner/benches/scan.rs`, `crates/scanner/tests/{reference,macos_reader}.rs`, `crates/scanner/tests/support/mod.rs`, `crates/rules/src/rules.rs`, `crates/testing/src/naive_walk.rs`, `docs/architecture/modules.md` |
| 03  | [An app's change time and the install records](./03-install-records.md)               | ✔️ done | `crates/core/src/system/`, `crates/core/src/lib.rs`, `crates/system/src/{apps,installs,lib}.rs`, `crates/testing/src/{system_fakes,run_module,lib}.rs`, every `SystemPorts {` and `InstalledApp {` site, `app/tauri/src/compose.rs`, `docs/architecture/` |
| 04  | [A module that needs a guarded folder runs in the scan that tries it](./04-availability-after-places.md) | ✔️ done | `crates/engine/src/{session,coverage,access}.rs`, `docs/architecture/permissions.md`                          |
| 05  | [The group Your files](./05-group-files.md)                                            | ✔️ done | `crates/core/src/group.rs`, `crates/core/locales/en.json`, `app/ui/src/{ipc/bindings.generated.ts,product/module-groups.ts}` |

All five in parallel; 01/03 and 01/01 both touch the three modules' test files: the second merge keeps both sides.
Each step writes its own part of `docs/architecture/`; 05/01 checks them together.

## Conclusions

Not written yet.
