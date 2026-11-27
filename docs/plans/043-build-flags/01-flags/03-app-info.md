# 043/01/03 — `get_app_info` reports the flags

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, app/tauri, app/ui (bindings,
readers of `high`, mocks), docs

`AppInfo.high` becomes `AppInfo.flags: BuildFlags`; the shell carries the flags from the composition to the command.
The UI follows mechanically (`info.high` → `info.flags.high`); Largest still follows High until 02/01. The design:
[`../00-design.md`](../00-design.md), sections 4 and 5; agent's call 1.

## Read first

01/02's commit; `crates/core/src/app_info.rs` and its tests; `app/tauri/src/compose.rs` (`Start`, `start`,
`Shared`, `assemble`); `app/tauri/src/sandbox.rs` (`SandboxMode`, `compose`); `app/tauri/src/lib.rs` (the `manage`
calls); `app/tauri/src/commands.rs` (`app_info` and its tests); `app/tauri/src/testing.rs` (the shell's state);
`app/tauri/tests/bindings.rs`; the UI's readers (`rg "\.high\b|high:" app/ui/src app/ui/e2e -g '!*.generated.ts'`).

## Files

- `crates/core/src/app_info.rs` — `high: HighLevel` becomes `flags: BuildFlags` (doc: what the build ships with; its
  `high` is the engine's); the module doc and the serialization test follow (`"flags":{"high":"withheld","largest":"hidden"}`).
- `app/tauri/src/compose.rs` — `Shared.high` becomes `Shared.flags: BuildFlags` (`normal` sets `FLAGS`); `assemble`
  passes `flags.high`; `Start` gains `flags: BuildFlags`, filled by `start` from the normal or the sandbox
  composition.
- `app/tauri/src/sandbox.rs` — `SandboxMode.high` becomes `flags: BuildFlags` (from `FLAGS`); `offering_high()` sets
  `flags.high = Offered`; `compose` passes `flags` into `Shared`.
- `app/tauri/src/lib.rs` — `app.manage(start.flags)` beside `build`.
- `app/tauri/src/commands.rs` — `app_info(build, flags: State<'_, BuildFlags>, engine)` reports `flags: BuildFlags {
  high: engine.high_level(), ..*flags.inner() }`; its doc names the flags.
- `app/tauri/src/testing.rs` — unchanged: the shell does not manage `BuildFlags` (Tauri's `manage` keeps the first
  value, the shell routes no IPC, and the `app_info` tests manage their own flags as they do for `Build`).
- `app/tauri/tests/bindings.rs` — `BuildFlags` and `LargestScreen` in `declarations()`; regenerate with
  `UPDATE_BINDINGS=1 cargo nextest run -p fresher-app --test bindings` and review the diff.
- `app/ui/src/ipc/bindings.generated.ts` (generated, never by hand).
- The UI, behaviour unchanged: `screens/frame-sidebar.tsx` (`useAppInfo()?.flags.high`), `screens/app-frame.tsx`
  (`info?.flags.high`); every unit-test fixture and story that builds an `AppInfo` (`app-frame.test.tsx`,
  `settings-screen.test.tsx`, `category-screen.test.tsx`, `scan-screen.test.tsx`, `review-screen.test.tsx`,
  `largest-screen.test.tsx`, `navigation.test.tsx`, `own-item-row`'s, `src/test/largest-scan.ts`): `flags: { high:
  …, largest: 'shown' }` in place of `high`.
- `app/ui/e2e/core-mock.ts` — the scripted `appInfo` answers carry `flags: { high: 'offered', largest: 'shown' }`;
  `withHigh` writes `flags.high` (the option is renamed in 02/02); `support/*.ts` that build an app info follow.
- `docs/architecture/README.md` — "The High switch" and "The High switch in the UI" say `AppInfo.flags.high`.

## Interfaces produced

- `AppInfo.flags: BuildFlags` over the IPC (`{ high, largest }`); `State<BuildFlags>` in the shell; `Start.flags`.

## Tests (negatives first)

- `crates/core/src/app_info.rs`: the existing negatives unchanged; positive
  `a_sandbox_build_serializes_with_its_folder` updated to `flags`.
- `app/tauri/src/commands.rs`: negative `app_info_reports_the_engines_high_over_the_managed_one` (managed `{
  Withheld, Hidden }`, an engine that offers: `flags.high` is `Offered`, `largest` stays `Hidden`); positive
  `app_info_is_the_crate_version_and_the_managed_build` asserts `info.flags`.
- `app/tauri/src/compose.rs`: positive `the_start_carries_the_builds_flags` (`start(..)` on temporary folders:
  `start.flags == FLAGS`).
- `app/tauri/src/sandbox.rs`: positive `offering_high_keeps_the_other_flags`.
- The bindings test passes on the regenerated file.

## Checks

- `cargo nextest run -p fresher-core -p fresher-app`, `npm run sandbox:test`
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`
- `npm run lint:ts`, `npm run test:ui`
- `npx playwright test --project chromium e2e/core-mock.spec.ts e2e/flows.spec.ts e2e/apps.spec.ts` (from `app/ui`)
- `rg "\binfo\??\.high\b|appInfo.*high:" app/ui -g '!*.generated.ts'` finds nothing outside `flags`.

Commit: `feat(app): get_app_info reports the build flags`.
