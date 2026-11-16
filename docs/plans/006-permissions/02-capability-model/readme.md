# 006/02 — Capability model

**Status:** ✔️ done · **Version:** 0.4.0 · **Date:** 2026-11-16 · **Modules:** core, engine, testing

The OS-neutral half of the plan: the capability types and the two ports in `fresher-core`, their fakes in
`fresher-testing`, and the engine's part — what each category may do with the current access, tool checks, the
opt-in settings, the re-check, and the coverage of a scan. Nothing here knows macOS. The design:
[`../00-design.md`](../00-design.md), sections 2, 3, 5 and 6.

Steps 01–03 depend only on plan 002; step 04 needs the walk's exclusions from plan 004/03.

## Steps

| #   | Step                                                     | Status   | Depends on    | Shared files                                                                     |
| --- | -------------------------------------------------------- | -------- | ------------- | -------------------------------------------------------------------------------- |
| 01  | [Core types and ports](./01-core-types-and-ports.md)     | ✔️ done | —             | `crates/core/src/{lib,ports,walk,category,module,event,command,error}.rs`, `crates/core/locales/en.json` |
| 02  | [Fakes](./02-fakes.md)                                   | ✔️ done | 01            | `crates/testing/src/fakes.rs`, `crates/testing/src/lib.rs`                       |
| 03  | [Availability, tools and opt-ins](./03-availability.md)  | ✔️ done | 01, 02        | `crates/engine/src/{engine,registry,lib}.rs`, `crates/core/src/settings/core_schema.rs`, `app/tauri/src/{adapters,compose}.rs` (the new port wired to a stand-in) |
| 04  | [Coverage of a scan](./04-coverage.md)                   | ✔️ done | 03, 004/03    | `crates/engine/src/session.rs`, `crates/core/src/command.rs`                     |

## Conclusions

- The capability model is in place, OS-neutral, one commit per step through the hooks: the core's types and ports
  (`Capability`, `Access`, `AccessStatus`, `Need`, `Availability`, `GuardedPlace`, `AccessProbe::{guarded_places,
  try_place}`, `SystemUi`), their fakes (`FakeAccess`, `FakeSystemUi`), the engine's view of access with the tool
  checks, availability per category, `ModuleOutcome::Unavailable`, the four opt-in switches, and the coverage of a
  scan (`Coverage`, `Event::ScanCoverage`, `LastScan.coverage`).
- A scan never touches a guarded place that is not allowed or opted into: such a place goes to the walk as avoided
  and is named in the coverage; an incomplete coverage ends the scan `Partial`. An unavailable module does not.
- Deviations, each recorded in its step: the fakes have their own file; a needed tool without a check is refused by
  the registry; the opt-ins are read through a small helper of the engine's settings; a capability is tried once per
  scan.
- What the app still lacks is plan 03's: the macOS adapters behind `AccessProbe` and `SystemUi` replace the
  stand-ins listed in `docs/hacks/README.md`.
