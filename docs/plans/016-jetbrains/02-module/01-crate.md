# 016/02/01 — The crate, products, names and texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
modules, docs

## Files

- `modules/jetbrains/Cargo.toml` (new) — `fresher-module-jetbrains`; workspace `version`, `edition`, `rust-version`,
  `license`, `publish`; `[lints] workspace = true`; depends on `fresher-core`, `serde`, `toml`, `thiserror` (all in the
  workspace already); dev-dependencies `fresher-testing`, `proptest`. Root `Cargo.toml`: the member.
- `modules/jetbrains/src/lib.rs` — `#![forbid(unsafe_code)]`; `JetBrainsError`; `JetBrainsModule::new()`: the
  descriptor (`jetbrains`, `ModuleGroup::Development`, the three categories at Safe, Safe, Medium, no settings, the
  dictionary) `.with_problems(…)` (the six left-out keys); `interests()` (the three `Under(…, "JetBrains/*")`,
  `Claim::Whole`); `on_match` and `finish` return `Ok(())` until 02/02.
- `modules/jetbrains/products.toml` and `src/products.rs` — the table of the design, section 2, and its checks.
- `modules/jetbrains/src/name.rs` — `Version`, `parse(name, &Products)`, `installed_version(text)` (section 3).
- `modules/jetbrains/locales/en.json` — every text: the three categories' five texts, `jetbrains.enabled.label` and
  `.description`, the finding's title and reason, the six left-out reasons (section 5), in plain English.
- `docs/categories/jetbrains-caches.md`, `jetbrains-logs.md`, `jetbrains-settings.md` (new) — in the catalogue's format:
  danger level, what it is, where it lives, what happens if you remove it, how the app decides (the decision table);
  `docs/categories/README.md` — the three rows, and the sentence "No category is documented yet" replaced.
- `Cargo.lock` (regenerated).

## Produces

`JetBrainsModule::new()`, `Products`, `parse`, `installed_version` for 02/02; the texts for 03/03.

## Checks

- Unit, negatives first (`name.rs`): every refused name of the design, section 3; a three- or five-digit year; a minor
  of `0`, `00` or `03`; a non-UTF-8 name; a prefix of one product followed by another's name. Positive: each product's
  folder name; `PyCharmCE` versus `PyCharm`, `IdeaIC` versus `IntelliJIdea`; `2025.10` after `2025.9`.
- Property (`mod property`): the order of `Version` is the order of `(year, minor)`; parsing the folder name built
  from any product and version gives them back.
- `installed_version`: `2025.3.1`, `2025.3`, `2026.1 EAP` read; `253.1234`, empty, `v2025.3` unknown.
- `products.rs`: a duplicate folder, an id in two products, an invalid id, an empty list refused; the shipped table
  parses.
- `tests/contract.rs`: `assert_module_contract(&module, &Catalogue::Required(docs/categories))` passes.
- `cargo nextest run -p fresher-module-jetbrains`; `cargo clippy -p fresher-module-jetbrains --all-targets -- -D warnings`;
  the crate guard (`#![forbid(unsafe_code)]`, workspace keys) and the dependency guard (no other module).

## Conclusions

Built as written: `modules/jetbrains` (`fresher-module-jetbrains`), the 16 products in `products.toml` (checked
constructors in `products.rs`), folder-name reading in `name.rs` (`Version`, `parse`, `installed_version`), the
descriptor with the three categories and six left-out keys (later eight), and `locales/en.json`. The module's rule
column in the catalogue reads "none (the module's code, `modules/jetbrains`)". 37 tests at the time, three proptests;
the contract test with `Catalogue::Required` passed.
