# 002/02/01 — Registry and event bus

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-15 · **Modules:** engine

## Files

- `crates/engine/Cargo.toml` — `fresher-engine`, `[lints] workspace = true`, dependencies `fresher-core`, `thiserror`,
  `tracing`. It falls under the `crates/*` glob: the root `Cargo.toml` does not change.
- `crates/engine/src/lib.rs` — the crate's API: `Registry`, `EventBus`, later `Engine`.
- `crates/engine/src/registry.rs` — `Registry::register(Box<dyn Module>)`: refuses a duplicate module id, a descriptor
  that does not validate, a dictionary that does not parse inside the module's namespace; collects categories, the
  settings schema and the merged dictionary; `Registry::modules()` gives `ModuleInfo`s in registration order.
- `crates/engine/src/bus.rs` — `EventBus::subscribe() -> Receiver<Event>`, `publish(&Event)`: sends a clone to every
  live subscriber, drops the ones whose receiver is gone, never blocks.
- `crates/engine/src/error.rs` — `EngineError` (internal), its mapping to `CommandError`.

## Produces

`Registry` and `EventBus` for 02/02–02/04 and for the app (05/02).

## Checks

- Negative: `refuses_a_second_module_with_the_same_id`, `refuses_dictionary_keys_outside_the_module_namespace`,
  `refuses_a_module_whose_dictionary_is_not_flat_json`, `a_dropped_subscriber_does_not_stop_the_others`.
- Positive: `every_subscriber_receives_every_event_in_order`, `merges_the_dictionaries_of_two_modules`,
  `lists_modules_in_registration_order`.
- `cargo nextest run -p fresher-engine` and Clippy pass.

## Notes from the implementation

- `error.rs` and `EngineError` come with step 02: before the state machine nothing inside the engine can fail, and an
  unused type fails Clippy.
- A descriptor is validated by `ModuleDescriptor::new` already; the registry also builds the settings schema with the
  new module (`RegistryError::Settings`) and parses its dictionary in its namespace (`RegistryError::Dictionary`). A
  refused module leaves the registry as it was.
- `Registry::schema()` returns `Result<SettingsSchema, SchemaError>`: the registry keeps the schema it checked, and an
  empty registry builds the core's one on demand, so there is no panicking path. `Registry::categories()` lists every
  category in registration order.
- The merged dictionary holds the modules' texts only; the core's own texts (plan 04) are merged in by plan 05.
- `crates/engine/src/testing.rs` (`#[cfg(test)]`) starts here with `TestModule`, a module a test builds.
