# 029/03/02 — The core e2e

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** e2e, fixtures, docs

The engine with the real scanner, the module, plan 024's real reader and the real `DiskCleaner` on a sandbox home, as
the flows of `crates/e2e/tests/local_ai_models.rs`. The together removal is proven here on the disk, with the real
Trash of the sandbox and the real journal. The design: [`../00-design.md`](../00-design.md), sections 2, 3 and 10.

## Files

- `fixtures/homes/local-ai-models-ollama-flow.toml` (new) — a store with `llama3:latest`/`llama3:8b` (same files),
  `mistral:7b`, a base model and a model made from it (two shared layers).
- `crates/e2e/tests/local_ai_models_ollama.rs` (new), through the shared harness of the flow homes, negatives first:
  - `mod negative`: `a_model_whose_layer_is_mapped_moves_nothing` (the open-files fake names a layer as mapped: the
    item ends `InUse`, the manifest and every layer in place, the journal's item `Skipped`);
    `a_shared_layer_and_the_sharers_manifest_stay_after_the_cleanup`;
    `a_layer_replaced_after_the_plan_moves_nothing` (a layer rewritten with another file id between the plan and the
    confirmation: `Replaced`, nothing moved);
  - `mod positive`: `the_scan_offers_each_model_with_its_freed_size` (and the reason's listed size);
    `a_confirmed_model_moves_its_manifests_and_unshared_layers_to_the_trash_together` (the journal's begin record
    holds every part, the item's `trashed` every part, the subject first); `put_back_restores_every_file_byte_for_byte`
    (the subject last); `ollama_lists_nothing_of_a_removed_model` (the module's next scan offers the rest unchanged);
    `a_restored_scan_offers_the_same_models`.
- `fixtures/golden/local-ai-models-ollama-flow.json` (new) — the flow's report, reviewed.
- `docs/testing.md` — `local_ai_models_ollama.rs` in the list of core e2e files.

## Produces

The together removal proven through the engine and the cleaner, for 04/02.

## Checks

- `cargo nextest run -p fresher-e2e --test local_ai_models_ollama`; `npm run test:e2e` (the core part).

## Conclusions

Built: eight flows over the real cleaner, negatives first: a mapped layer (in use, nothing of the model moves), a
layer replaced after the plan, shared layers and a twin that stay; positives: a confirmed together move (the begin
record holds every part, the item's `trashed` every part, subject first, `went` equal to the parts), put back byte for
byte over the whole store with `back` ending at 0 (subject last), a removed model not listed on the next scan, and a
restored scan, which after a restart with a manifest moved away reads the manifests again and leaves the store out as
`ollama-unreadable` (a rename stands in for a removal: the lints forbid `remove_file` in tests). The golden is passed
through `Flow::relative`, which sorts lists of paths, so the golden's lists look sorted; the order is asserted on the
raw journal. `FakeOpenFiles` has no mapped or descriptor split, so the mapped case is the in-use case with a comment
(F11). The flow home has the same shape as the module's. Review: needed fixes (vacuous put-back loop, subject-first
only checked as "a manifest"), fixed in round 1 and re-reviewed.
