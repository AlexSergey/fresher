# 023/02/02 — The core e2e

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** e2e, fixtures

The engine with the real scanner, the module and the real `DiskCleaner` on a sandbox home, as the flows of
`crates/e2e/tests/dev_caches.rs`. The design: [`../00-design.md`](../00-design.md), section 7.

## Files

- `fixtures/homes/local-ai-models-flow.toml` (new) — a home with a Hugging Face model (snapshot links into its
  blobs), a dataset, an LM Studio model, a Space, and a project folder holding a link into the model's blobs.
- `crates/e2e/tests/local_ai_models.rs` (new) and `crates/e2e/tests/local_ai_models_home/` (the flow's home and
  helpers, through the shared harness of the sandbox flow homes) — the flows, negatives first:
  - `mod negative`: `a_space_and_the_project_s_link_are_never_offered`; `a_model_with_a_file_open_is_skipped_in_use`
    (the open files fake names a blob: the item ends `InUse`, nothing moved); `the_project_s_link_stays_a_link_after_the_cleanup`
    (the link is still there as a link, its target gone with the model);
  - `mod positive`: `the_scan_offers_each_model_in_its_category_with_its_sizes`;
    `a_confirmed_cleanup_moves_the_model_folder_with_its_links_to_the_trash` (the folder lands whole in the
    sandbox's Trash, the journal has one entry; put back restores it);
    `a_restored_scan_offers_the_same_models` (the cached matches at launch).
- `fixtures/golden/local-ai-models-flow.json` (new) — the flow's report, reviewed.
- `crates/e2e/Cargo.toml` — the module as a dev-dependency.
- `docs/testing.md` — `local_ai_models.rs` in the list of core e2e files.

## Produces

The module proven through the engine and the cleaner, for 03.

## Checks

- `cargo nextest run -p fresher-e2e --test local_ai_models`; `npm run test:e2e` (the core part).

## Conclusions

Built as written: `crates/e2e/tests/local_ai_models.rs` and its flow home. The flows run on the engine's default scan
root (`~`), which proves that hidden folders such as `.cache` and `.lmstudio` are reached with no override (a review
round removed a roots override the first version carried). The flows: the sizes of each category, scan, confirm with
Medium, Trash and put back byte for byte, a model with a blob held open skipped in use, the project's link kept as a
link, and a restore at launch that offers the same models.
