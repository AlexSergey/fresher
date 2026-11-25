# 042/01/03 — The disk's five parts

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core, engine (tests), app/ui (mocks)

Each volume's map carries a summary of five parts, merged in the core, with plain names and one-sentence "i" texts:
what the disk card draws. The design: [`../00-design.md`](../00-design.md), section 2; agent's calls 3 and 11. Mockup:
[`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html) (the legend).

## Read first

`crates/core/src/storage/{map.rs,mod.rs}` (`VolumeMap::new`, `PartKind`, `MapPart`), `crates/engine/src/storage/
{volume.rs,query.rs,volume_tests.rs}`, `crates/core/src/texts.rs` (`core_keys`), `crates/core/locales/en.json`
(`core.storage.*`), `crates/engine/src/results.rs` (`add`, the sum of sizes), `app/ui/src/test/storage-map.ts`,
`app/ui/e2e/core-mock.ts` (its storage maps), `app/ui/src/product/storage-map/core-keys.test.ts`.

## Files

- `crates/core/src/storage/summary.rs` (new, exported through `storage/mod.rs` and `lib.rs`):
  - `pub enum SummaryKind { CanBeFreed, FilesAndApps, MacOs, SystemSpace, Free }` (`Serialize, TS`, kebab-case:
    `can-be-freed`, `files-and-apps`, `mac-os`, `system-space`, `free`), in that order, with `ALL`.
  - `pub struct SummaryPart { pub kind: SummaryKind, pub sizes: Sizes }` (`Serialize, TS`).
  - `SummaryKind::of(&PartKind) -> SummaryKind`: `Explained` → `CanBeFreed`; `YourFolders`, `NotScannedYet`,
    `Remainder { .. }`, `OtherVolume { .. }` → `FilesAndApps`; `MacOs` → `MacOs`; `SwapAndSleep`, `DiskOwn` →
    `SystemSpace`; `Free` → `Free`. An exhaustive `match`, no wildcard, so a new part kind must choose.
  - `name_key()` → `core.storage.summary.<kind>.name`, `description_key()` → `core.storage.summary.<kind>.description`.
  - `pub(crate) fn summarize(parts: &[MapPart]) -> Vec<SummaryPart>`: always five parts in `ALL`'s order, each the sum
    of its parts' sizes (every field, the weakest completeness; a sum past `u64::MAX` stays at the maximum), a kind
    with no part at zero.
- `crates/core/src/storage/map.rs` — `VolumeMap` gains `summary: Vec<SummaryPart>`, set in `new` from `parts`;
  `pub fn summary(&self) -> &[SummaryPart]`; its doc comment: "what the disk card draws; the parts stay for the
  by-meaning view and the place view".
- `crates/core/locales/en.json` and `core_keys` — exactly:
  - `core.storage.summary.can-be-freed.name` "Can be freed"; `.description` "What Fresher found that you can remove,
    sorted below by what removing it costs."
  - `core.storage.summary.files-and-apps.name` "Your files and apps"; `.description` "Your documents, apps and
    everything else Fresher does not offer to remove."
  - `core.storage.summary.mac-os.name` "macOS"; `.description` "The system itself, which macOS looks after; Fresher
    never touches it."
  - `core.storage.summary.system-space.name` "System space"; `.description` "Memory macOS keeps on disk, and space the
    disk keeps for itself."
  - `core.storage.summary.free.name` "Free"; `.description` "Space you can still use for new files."
- `app/ui/src/ipc/bindings.generated.ts` (regenerated); `app/ui/src/test/storage-map.ts` gains a fixture helper
  `withSummary(volume)` that fills `summary` from the fixture's parts by the same table (test code only; a comment
  names `crates/core/src/storage/summary.rs` as the rule's home); every UI fixture and `core-mock.ts` map gains its
  `summary` through it. `product/storage-map/core-keys.test.ts` (or its neighbour) checks the ten new keys exist in
  the core's dictionary.

## Tests (negatives first)

`summary.rs`, `mod negative`: `an_empty_map_summarizes_to_five_zero_parts`; `a_sum_past_the_maximum_stays_at_it`;
`the_weakest_completeness_wins`. `mod positive`: `every_part_kind_goes_to_its_summary_kind` (one of each kind, the
remainder both ways, two other volumes); `the_summary_adds_up_to_the_capacity` (`map.rs`, every kind present);
`serializes_as_kebab_case`; `the_keys_name_each_kind`. `crates/engine/src/storage/volume_tests.rs`: positive
`a_complete_map_carries_its_summary` (the scanned map of an existing test, its five parts' sizes).

## Checks

`CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-core -p fresher-engine -p fresher-app` (bindings), clippy,
fmt; `npm run lint:ts`; `npx vitest run` on the storage-map and scan-hero tests (they build maps);
`npx playwright test e2e/core-mock.spec.ts --project chromium` (the mock's maps).

Commit: `feat(core): the disk's five parts with plain names`.

## Conclusions

- `crates/core/src/storage/summary.rs`: `SummaryKind`, `SummaryPart` and `summarize`, five parts always, built in
  `VolumeMap::new`; the ten texts verbatim. The UI's fixture helper `withSummary` and its key check live in
  `app/ui/src/test/` (ruling C-1).
- The detailed parts stay in the core for its by-meaning and place queries; the UI draws only the summary (ruling
  D-8); `docs/architecture/README.md` says so.
