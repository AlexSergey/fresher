# 002/01 — Core contracts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** core, repo

Every type, trait and port the rest of plan 002 and the later plans build on, in `fresher-core`: no IO, fully tested.
The design: [`../00-design.md`](../00-design.md), "Core types", "A module", "Ports", "Events and commands", "Errors",
"Settings and the app's own data".

Runs first and alone; its steps in order (each builds on the previous one). After it, `crates/core/` is frozen for the
plans of wave 2 (only plan 04 adds the core's dictionary file).

## Steps

| #   | Step                                                            | Status      | Shared files                                                                    |
| --- | --------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------- |
| 01  | [Groundwork and domain types](./01-groundwork-and-domain-types.md) | ✔️ done     | root `Cargo.toml`, `Cargo.lock`, `.commitlintrc.cjs`, `docs/quality.md`         |
| 02  | [The module trait and ports](./02-module-trait-and-ports.md)     | ✔️ done     | —                                                                               |
| 03  | [Settings schema and file versions](./03-settings-schema.md)     | ✔️ done     | —                                                                               |
| 04  | [Events, commands and errors](./04-events-commands-errors.md)    | ✔️ done     | —                                                                               |
| 05  | [The dictionary type](./05-dictionary-type.md)                   | ✔️ done     | —                                                                               |

## Conclusions

Built (2026-11-15): `fresher-core` holds every contract of plan 002, with no IO; 101 tests, 99.5 % of lines covered
(`cargo llvm-cov nextest -p fresher-core`).

- **Domain types** (01): `ModuleId`, `CategoryId`, `FindingId`, `FindingKey`, `ScanId`, `ToolId`, `Timestamp`;
  `DangerLevel`; `ByteSize`, `Sizes`, `Completeness`; `Subject`; `Removal` with the validated `ToolCommand`;
  `TextKey`, `Text`, `TextArg`; `CategoryDescriptor` with `CategoryText`; `NewFinding`, `Finding`.
- **The module contract and ports** (02): `Module`, `ModuleDescriptor`, `Locales`, `ModuleContext`, `FindingSink`,
  `ModuleError`, `DescriptorError`; `Interest`, `Pattern`, `Claim`, `InterestSet`, `Match`, `EntryKind`; `DiskWalk`,
  `WalkProgress`, `WalkSummary`, `Unreadable`, `WalkError`; `CancelToken`; `AccessProbe`, `AccessStatus`, `Access`;
  `Clock`, `Trash`, `CommandRunner` now `Send + Sync`.
- **Settings** (03): `SettingKey`, `SettingKind`, `SettingValue`, `Unit`, `SettingDescriptor`, `SettingProblem`,
  `SettingsSchema`, `ResolvedSettings`, `ModuleSettings`; `SettingsStore`, `StoredSettings`, `FormatVersion`,
  `VersionPolicy`, `StoreError`.
- **Events, commands, errors** (04): `Event`, `ModuleOutcome`, `ScanOutcome`; `Operation`, `EngineStatus`, `Overview`,
  `LastScan`, `CategoryTotal`, `ModuleInfo`, `FindingsQuery`, `FindingsPage`, `SettingsView`, `SettingUpdate`,
  `Locale`; `CommandError` with `code()`, `CODES` and `text()`.
- **The dictionary** (05): `Dictionary`, `DictionaryError`.

Deviations from the steps (each written in its step file, "Notes from the implementation"):

- The port's error `CommandError` is renamed `CommandRunError`: the name goes to the error the UI sees.
- `fresher-testing`'s fakes hold their state in a `Mutex`, because the ports are now `Send + Sync`.
- Settings are split into `settings.rs`, `settings/schema.rs` and `settings/core_schema.rs`; `ModuleDescriptor::new`
  takes the module's settings and refuses one outside its namespace, a duplicate and `<module>.enabled`.
- Added types the steps did not name: `IdError`, `TextKeyError`, `RemovalError`, `InterestError`, `CategoryText`,
  `ToolCommand`, `Unreadable`, `SettingError`, `SchemaError`, `LastScan`; `DictionaryError` has `InvalidKey`, and
  `NotFlat` names the key.
- Enum tags are kebab-case everywhere; enums with newtype variants are adjacently tagged (`kind` + `value`).
- An integration test (`crates/core/tests/typescript.rs`) builds the TypeScript declaration of every exported type.
- `rust-toolchain.toml` and `ts-rs` 12.0.1 needed no change: its default feature `serde-compat` is the one the plan
  names; Rust 1.99.1 builds it.
