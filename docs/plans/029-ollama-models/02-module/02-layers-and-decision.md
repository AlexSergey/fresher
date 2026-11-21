# 029/02/02 — The layers and the decision

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, docs

Which layers each model uses and frees, the store's and the model's rows, and the report: one finding per model with
`Removal::MoveTogether`, or the store's or the model's left-out. The design: [`../00-design.md`](../00-design.md),
sections 7 to 9.

## Files

- `modules/local-ai-models/src/ollama/layers.rs` (new) — pure, on the parsed manifests and the gathered blobs:
  - `pub(crate) struct Model { tags: Vec<OllamaTag>, manifests: Vec<PathBuf>, digests: BTreeSet<Digest> }`, built by
    grouping manifests with the same digest set (tags in name order);
  - `pub(crate) struct Layout { models: Vec<Model>, named_by: BTreeMap<Digest, Vec<usize>> }` and
    `pub(crate) fn layout(manifests: Vec<(OllamaTag, PathBuf, Manifest)>) -> Layout`;
  - `Layout::parts(model) -> (unshared digests)`, `sharers(model) -> Vec<usize>` (other models naming one of its
    layers, by name), in name order.
- `modules/local-ai-models/src/ollama/decide.rs` (new) — on plain values, never a port:
  - `pub(crate) enum StoreVerdict { NotClaimed, LeftOut(Reason), Open(Layout) }` and `pub(crate) fn store(…)`: the
    design's rows 1 to 4 in order (any manifest or blob or manifests' folder with `owned_by_user` false; any
    `BlobName::Partial`; any manifest not read);
  - `pub(crate) enum ModelVerdict { NotClaimed, LeftOut(Reason), Offered(Offer) }`, `Offer { parts: Parts, freed:
    Sizes, listed: ByteSize, sharers: Vec<String>, date: Option<Timestamp> }`, and `pub(crate) fn model(…)`: rows 5 to
    7 (a named layer absent, not a regular file, or of another logical size; more than `MAX_PARTS` parts).
  - `Reason` (plan 023's) gains `OllamaUnreadable`, `OllamaDownload`, `OllamaIncomplete`.
- `modules/local-ai-models/src/report.rs` — the Ollama finding (design section 8): identity, `Subject::Path` of the
  first manifest, `Removal::MoveTogether`, the freed sizes, the title, the reason by the number of sharers; the store's
  left-out with the store's sizes, a model's with its parts' sizes.
- `modules/local-ai-models/src/lib.rs` — `finish`: once the other stores are decided, reads every gathered manifest
  through `cx.system().small_file(parent, tag)`, checking `cx.cancel()` between reads; decides the store, then each
  model in identity order; the module's docs name Ollama and the files it reads.
- `modules/local-ai-models/src/categories.rs` — `ollama`, Medium.
- `modules/local-ai-models/locales/en.json` — the texts of the design's section 9.
- `docs/categories/local-ai-models-ollama.md` (new) — the category as built: the store, the rows, the texts, the
  danger and why; `docs/categories/README.md` — its row.
- `modules/local-ai-models/src/tests.rs` — the module run on hand-built matches with a fake `SmallFiles` (plan 024):
  every row through `finish`.

## Produces

Ollama's findings and left-outs, for 03 and the end-to-end plan.

## Checks

- `ollama/layers.rs`, negative first: `a_layer_another_model_names_is_never_a_part`,
  `an_incomplete_models_layers_still_count_as_named`; positive: `tags_with_the_same_files_are_one_model`,
  `a_model_frees_its_manifests_and_its_unshared_layers`, `sharers_are_listed_by_name`; properties (proptest):
  `the_parts_of_two_models_never_overlap`, `freed_and_shared_layers_add_up_to_every_named_layer`,
  `the_layout_does_not_depend_on_the_order_of_manifests`.
- `ollama/decide.rs`, one test per row, keep before offer: `no_store_or_no_layout_is_not_claimed`,
  `a_foreign_manifests_folder_leaves_out_the_store`, `a_foreign_layer_leaves_out_the_store`,
  `a_partial_download_leaves_out_the_store`, `one_unreadable_manifest_leaves_out_the_store`,
  `a_missing_layer_leaves_out_the_model`, `a_layer_of_another_size_leaves_out_the_model`,
  `a_linked_layer_leaves_out_the_model`, `too_many_parts_is_not_claimed`; positive: `a_model_is_offered_with_its_freed_sizes`;
  property: `a_restored_scan_decides_the_same`.
- `tests.rs`: the finding's identity, subject, parts, sizes, title and each reason form; the left-outs' sizes.
- `cargo nextest run -p fresher-module-local-ai-models` (module coverage at or above 90 %); clippy; fmt.

## Conclusions

Built together with 02/03 in one commit (preflight ruling F4/F5), then fixed in two rounds from the reviews. The
decision has seven rows (design section 8); beyond the design: a model with fewer than two parts (every layer shared
with another model) is not claimed, since `Parts` refuses one part; a model with no known time is not claimed; "not
the user's" covers a system or user lock and a restricted entry, as plan 023 does; manifests are read only once the
store holds `manifests` and `blobs`, and only after the rows that can leave the store out without them. Fail closed
beyond the design, from the reviews: an entry of the manifests' tree at host, namespace or model level that is not a
folder of a plain name (a link, a file, a socket, a hidden or non-UTF-8 folder) leaves the whole store out as
`ollama-unreadable`, because it could hide a manifest and make a shared layer look unshared; a hidden regular file
(`.DS_Store`) is ignored. A count of manifests read that differs from the count found also leaves the store out. The
reason with a count has plural forms (`ollama-shared-more.one/.other`, the arg named `count`), and names the sharer
by its first tag's title. Plan 023's not-claimed home and sandbox case lost only their manifest, so their meaning
(Ollama present, not claimed) holds and the 023 golden did not change. Review: needed fixes (the link and then the
hidden folder), fixed in two rounds and re-reviewed.
