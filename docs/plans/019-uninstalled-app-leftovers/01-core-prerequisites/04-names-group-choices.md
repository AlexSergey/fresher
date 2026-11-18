# 019/01/04 — A removed app's name, the group Apps, the choices

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 ·
**Modules:** core, engine, testing, modules, app/ui, docs

Three small core additions the module needs, with no walk in them: a finding may name its app for when the app is no
longer installed (a deferred item, due here); the group Apps; and a module's
selection choices, each a set of its own categories with two texts. The design: [`../00-design.md`](../00-design.md),
section 1.4 (agent's calls 2 and 12). Three commits, one per part.

## Files

A removed app's name:

- `crates/core/src/finding.rs` — `NewFinding` gains `app_name: Option<String>` (serialized beside `app` with the
  struct's serde naming, skipped when `None`); `#[must_use] pub fn for_app_named(self, app: BundleId, name: impl
  Into<String>) -> Self`; `pub fn app_name(&self) -> Option<&str>`. `for_app` leaves the name `None`.
- `crates/engine/src/storage/attribution.rs` — `AppPlaces::app(&self, id: &BundleId, name: Option<&str>)`: the
  installed app's `AppRef` when one has the id (its own name wins); otherwise the given name, or the id when none,
  `installed: false`. `crates/engine/src/storage/meaning.rs` passes `new.app_name()`.
- `modules/jetbrains/src/lib.rs` — `decide_uninstalled` gives `.for_app_named(app.clone(), product.name())`; the
  findings of an installed product keep `for_app`. `fixtures/golden/jetbrains.json` regenerated (`UPDATE_GOLDEN=1`)
  and reviewed: only the uninstalled findings gain the name.
- `docs/deferred/removed-app-name-in-map.md` removed and its row in `docs/deferred/README.md` removed.

The group Apps:

- `crates/core/src/group.rs` — `ModuleGroup::Apps` ("What apps leave behind: the files of apps you removed."), `as_str`
  `apps`; `ALL` in the screen's order `[System, Apps, Development, Debug]`. The negative test that reads `"apps"` as an
  unknown group reads `"downloads"` instead.
- `crates/core/locales/en.json` — `core.group.apps.name` "Apps", `core.group.apps.description` "What apps leave
  behind, such as the files of apps you removed."
- `app/ui/src/product/module-groups.ts` — `GROUP_ORDER` gains `'apps'` after `'system'`; its test.

The choices:

- `crates/core/src/module.rs` — `pub struct ModuleChoice` (`name: String`, `categories: Vec<CategoryId>`, with
  `new(name, categories)` and accessors); `ModuleDescriptor::with_choices(self, choices: Vec<ModuleChoice>) ->
  Result<Self, DescriptorError>` refused with `DescriptorError::ChoiceName` (not kebab case, or listed twice),
  `EmptyChoice` (no category) and `ChoiceCategory` (a category not the module's); `choices(&self)`. The texts
  `<module>.choice.<name>.label` and `.description` are among the keys the descriptor implies (so `MissingText` names a
  missing one).
- `crates/core/src/command.rs` — `ModuleInfo` gains `choices: Vec<ChoiceInfo>`, `pub struct ChoiceInfo { pub name:
  String, pub categories: Vec<CategoryId> }` (`Serialize`, `TS`), empty for a module without choices.
- `crates/testing/src/contract.rs` — `assert_module_contract` checks the choices' texts (through the implied keys) and
  `assert_texts_used` does not require a choice's texts to be used by a run (they are the page's, not a finding's).
- `app/ui/src/ipc/bindings.generated.ts` — regenerated (`cargo test -p fresher-app --test bindings` or the repo's
  command), never edited by hand.

## Produces

`NewFinding::{for_app_named, app_name}`, `ModuleGroup::Apps`, `ModuleChoice`, `ModuleDescriptor::{with_choices,
choices}`, `ModuleInfo.choices`, for 02/01 (the module's group and choices), 03/03 (the page's choices) and 03/02 (the
map).

## Checks

- `finding.rs`: `for_app` has no name; `for_app_named` keeps both; serialized without the name when `None`.
- `attribution_tests.rs`, negatives first: an installed app's name wins over a given one; no name given: the id; then
  a removed app with a name: its name, `installed: false`. `meaning` tests: a named removed app's line.
- `jetbrains`: a removed IDE's finding names the product (`DataGrip`), its golden reviewed.
- `group.rs`: `ALL` sorted, `Debug` last, `Apps` after `System`; `apps` read back; every group's texts exist (the
  core's text test).
- `module.rs`, negatives first: a choice named `Keep`, `keep_settings`, twice; an empty choice; a choice of another
  module's category; then two valid choices kept in order. `command.rs`: `ModuleInfo::new` carries them.
- `contract.rs`: a module with a choice but no `.label` fails `MissingText`.
- `npx vitest run src/product/module-groups.test.ts`; `npm run typecheck` (or the repo's TypeScript check) with the
  new bindings.
- `cargo nextest run -p fresher-core -p fresher-engine -p fresher-testing -p fresher-module-jetbrains -p fresher-app`;
  clippy; fmt.

## Conclusions

Built (b830b13, 738c393, 813d440), one commit per part: `NewFinding::for_app_named(id, name)` (the wire field is
`appName`, skipped when none) and the attribution that shows the name of a removed app, with `jetbrains` naming its
product and its golden gaining `appName` on the three uninstalled findings; the deferred item is removed. `ModuleGroup::Apps`
(order System, Apps, Development, Debug) with its core texts. `ModuleChoice`, `ModuleDescriptor::with_choices`,
`DescriptorError::{ChoiceName, EmptyChoice, ChoiceCategory}`, `ModuleInfo.choices` (`ChoiceInfo`), the choice texts
checked by the contract and the registry, the bindings regenerated and every TypeScript `ModuleInfo` literal given
`choices: []` (preflight F6). `docs/architecture/modules.md` describes it.

Deviations and calls:

- The first commit holds a snake_case `app_name` in the golden; the camelCase rename and the bindings landed in the
  second. Only the final state is consistent; history was not rewritten.
- A choice's texts count as used by `descriptor_keys`, so the contract does not require them to be read by a run.
- Three negative tests that read "apps" as an unknown group now read "downloads".

Tests: finding, attribution (an installed app wins, no name falls back to the id, a removed app with a name), the
meaning line, group order and read-back, the descriptor's choice refusals (empty, a foreign category, a repeated or
malformed name, order kept), the command's `ModuleInfo.choices`, the contract (a missing label fails with
`MissingText`), the vitest group order. `cargo nextest run --workspace`: 2626 passed; tsc, eslint and vitest clean; no
e2e (no screen changed).

Parked minors: a doc comment in `contract.rs` is about 158 characters wide; there is no test for "no name given: the
id" and no jetbrains test named for the product's name (the golden covers it).
