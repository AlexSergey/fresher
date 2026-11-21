# 023/01/03 — Fixtures, golden and registration

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, fixtures, app

The module on fixture trees through `run_module` (the reference walk), its golden report, the contract test over the
fixture runs, and its registration in every build of the app. The design: [`../00-design.md`](../00-design.md),
section 7.

## Files

- `fixtures/homes/local-ai-models-not-claimed.toml` (new) — under the fixture home: `.cache/huggingface/hub/` with
  `spaces--o--demo/{blobs/b,snapshots/r/app.py}`, `.locks/models--o--n/x.lock`, `version.txt`,
  `models--o--empty/refs/main` (no `blobs`, no `snapshots`), `models--a--b--c/{blobs/b,snapshots/r/f}`, and a link
  `models--o--linked` to a folder outside the hub; `.lmstudio/models/pub/loose.gguf` (a file at a model's depth),
  `.lmstudio/models/.hidden/m/w.gguf`; `.ollama/models/manifests/registry.ollama.ai/library/llama3/latest` and
  `.ollama/models/blobs/sha256-0001`; `Code/app/models/llama.gguf` (a model file in a project). Nothing is claimed.
- `fixtures/homes/local-ai-models-kept.toml` (new) — `models--o--locked/` with a blob carrying `flags = ["user-lock"]`
  (`not-yours`), `models--o--socket/` with a socket inside (`special`); `.lmstudio/models/pub/open-model/w.gguf` run
  with LM Studio among the running apps (`app-open`) and once with the running apps failing (`no-system`).
- `fixtures/homes/local-ai-models-offered.toml` (new) — `models--meta--tiny/` with `blobs/aa`, `blobs/bb`,
  `refs/main`, `snapshots/r1/config.json` → `../../blobs/aa` and `snapshots/r1/model.safetensors` → `../../blobs/bb`
  (links inside the folder, counted as links); `models--gpt2/` without an owner; `datasets--o--words/`;
  `.lmstudio/models/pub/model-a/model.gguf`; `.cache/lm-studio/models/pub/model-b/model.gguf`. Small files, real
  sizes.
- `fixtures/golden/local-ai-models.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings (titles, reasons, sizes,
  dangers), the left-outs.
- `modules/local-ai-models/tests/fixtures.rs` (new) and `tests/support/mod.rs` — the three homes through `run_module`,
  as `modules/dev-caches/tests/fixtures.rs`; the golden comparison.
- `modules/local-ai-models/tests/contract.rs` — the fixture runs added, so every text and every declared left-out is
  used.
- `app/tauri/Cargo.toml` — the crate as a dependency; `app/tauri/src/compose.rs` — `LocalAiModels(#[from] …)` in the
  compose error and the module in `modules()`, beside `dev-caches`; `app/tauri/src/commands.rs` — the module list, if
  it names modules.

## Produces

The registered module, for 02.

## Checks

- `tests/fixtures.rs`, negatives first: `mod negative` `nothing_in_the_not_claimed_home_is_offered_or_left_out`,
  `the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason`; `mod positive`
  `the_offered_home_offers_every_model_in_its_category`, `a_model_s_size_counts_its_blobs_once_and_its_links_as_links`,
  `the_golden_report_matches`.
- `tests/contract.rs` passes with the fixture runs.
- The app's composition tests (the module registered, its id taken once); `cargo nextest run -p
  fresher-module-local-ai-models -p fresher-app`; coverage of the crate at or above 90 %.

## Conclusions

Built as written: the three fixture homes (`-not-claimed`, `-kept`, `-offered`), the golden report, the contract
test with `Catalogue::Required`, and the registration in `app/tauri`. Fixtures that need a date carry `changed`, so the
reports are deterministic. The golden's allocated and reclaimable sizes are 0 (a fixture tree has no disk blocks), as
the other modules' goldens. The row without a time is covered by the decision's unit tests, not by a fixture.
