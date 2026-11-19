# 020/01/01 — The shared day setting

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** core, modules, docs

`jetbrains`, `app-leftovers` and `temp` each carry the same "unchanged for N days" setting (about 30 lines and its
tests, copied); `downloads` would be the fourth. The deferred item "a shared day setting" (its file is
removed by this step) is due: one type in the core, the three copies removed.
No stored setting changes: the keys, ranges and defaults stay. The design: [`../00-design.md`](../00-design.md),
section 1.1 (decision 8).

## Files

- `crates/core/src/settings/days.rs` (new) — `pub struct DaySetting { name: &'static str, min: u32, max: u32, default:
  u32 }` (`Clone, Copy, Debug, PartialEq, Eq`), documented as in the design:
  - `pub const fn new(name: &'static str, min: u32, max: u32, default: u32) -> Self`;
  - `pub fn descriptor(&self, module: ModuleId) -> Result<SettingDescriptor, SettingError>`: `SettingKey::new(format!
    ("{module}.{name}"))?`, `SettingKind::Number { min, max, unit: Some(Unit::Days) }` (as `i64`), default
    `SettingValue::Number(default)`, through `SettingDescriptor::new` (which refuses a default outside the range with
    `SettingError::BadDefault`);
  - `pub fn days(&self, settings: &ModuleSettings) -> u32`: `settings.number(name)`, the default when `None`, clamped
    to `min..=max`, converted with the default as the fallback — the body of today's three readers.
- `crates/core/src/settings.rs` — `mod days;` and `pub use days::DaySetting;`; `crates/core/src/lib.rs` — `DaySetting`
  in the `settings` re-export.
- `modules/jetbrains/src/lib.rs` — `const UNINSTALLED_AFTER: DaySetting = DaySetting::new("uninstalled-after-days", 7,
  3650, 30);` (with the comment of the old `MIN`: never fewer than a week); `UNINSTALLED_AFTER.descriptor(id)?` and
  `UNINSTALLED_AFTER.days(cx.settings())` where `settings::…` was called; `mod settings;` and
  `modules/jetbrains/src/settings.rs` removed.
- `modules/app-leftovers/src/lib.rs` — the same, its own `UNINSTALLED_AFTER`; `src/settings.rs` removed.
- `modules/temp/src/lib.rs` — `const OLDER_THAN: DaySetting = DaySetting::new("older-than-days", 7, 30, 7);` (the
  comment: the check can only become stricter); `src/settings.rs` removed.
- Each module's error keeps `Setting(#[from] SettingError)` (the descriptor still returns it).
- `docs/architecture/modules.md` — "The crate" or the settings paragraph: a module with an "unchanged for N days"
  setting declares a `DaySetting` constant and uses its `descriptor` and `days`.
- `docs/deferred/shared-day-setting.md` removed; its row in `docs/deferred/README.md` removed.

## Produces

`fresher_core::DaySetting`, for 03/01 (`downloads.older-than-days`, 7 to 3650, 30).

## Checks

- `days.rs`, negatives first:
  - `mod negative`: `a_default_outside_the_range_is_refused` (`DaySetting::new("x-days", 7, 30, 31)` →
    `SettingError::BadDefault`), `a_name_that_makes_no_key_is_refused` (`"Bad_Name"` → `SettingError::InvalidKey`),
    `a_value_that_is_not_resolved_is_the_default` (`ModuleSettings::empty`),
    `a_stored_value_out_of_range_or_not_a_number_is_the_default` (resolved through `SettingsSchema` from
    `{"<module>.x-days": 0}` and `"x"`);
  - `mod positive`: `the_descriptor_is_a_number_of_days_in_its_range` (key, kind with `Unit::Days`, default),
    `a_stored_value_in_range_is_read` (7, 14, 30, and 3650 for a wide setting).
  The schema test helper builds a one-setting module descriptor in the test, as `settings/schema.rs`' tests do.
- The three modules' tests pass unchanged where they store the setting (`modules/temp/src/tests.rs`,
  `modules/jetbrains/src/tests.rs`, their `tests/fixtures.rs` and `tests/contract.rs`); `jetbrains`' test that lists
  its settings (`["jetbrains.uninstalled-after-days"]`) passes; `fixtures/golden/temp-flow.json` is unchanged.
- `cargo nextest run -p fresher-core -p fresher-module-jetbrains -p fresher-module-app-leftovers -p
  fresher-module-temp -p fresher-e2e`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check`;
  `npm run check:machete` (no module keeps an unused dependency, such as `serde_json` used only by a removed test).

## Conclusions

The type is `fresher_core::DaySetting`; the three copies are removed and the stored keys, ranges and defaults are unchanged. `downloads` uses it in 03/01.
