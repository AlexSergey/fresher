# 029/02/03 — Fixtures and the contract

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, fixtures

The module run on fixture homes through `run_module` (real files, the real `DiskFiles` reader of plan 024), its golden
report, and the contract with four categories. The design: [`../00-design.md`](../00-design.md), section 10.

## Files

- `fixtures/homes/local-ai-models-ollama-not-claimed.toml` (new) — an Ollama folder without `manifests/`; a store
  holding only `manifests/`; a manifest at depth three; a hidden namespace; a layer no manifest names; a name off the
  blob shape. Nothing claimed, nothing left out.
- `fixtures/homes/local-ai-models-ollama-kept.toml` (new) — a store with a partial download (store left out
  `ollama-download`). Two more homes, one per store-wide row: `-ollama-unreadable` (one manifest is `text = "not
  json"`) and `-ollama-long` (a manifest of 4097 bytes): left out `ollama-unreadable`. An incomplete model (a layer of
  the wrong size) beside an offered one goes in `-offered`.
- `fixtures/homes/local-ai-models-ollama-offered.toml` (new) — `llama3:latest` and `llama3:8b` with the same files
  (one finding, both tags); `mistral:7b` with no sharer; a base model and a model made from it sharing two layers (the
  shared reasons); a model in another namespace and one on `hf.co` (titles); the incomplete model (left out). Manifests
  are `text` (design section 6's form), layers files with `size` and `fill`, the manifests' sizes matching.
- `fixtures/golden/local-ai-models-ollama.json` (new) — `UPDATE_GOLDEN=1`, reviewed: every finding's parts, freed
  size, title and reason.
- `modules/local-ai-models/tests/fixtures.rs` — the runs, negatives first; plan 023's homes run unchanged and their
  golden is unchanged (their Ollama entries have no `manifests/`).
- `modules/local-ai-models/tests/contract.rs` — `Catalogue::Required` with the four categories; every Ollama text and
  left-out reason used by a run; every part of every removal a match.

## Produces

The module proven on real files, for 03.

## Checks

- `cargo nextest run -p fresher-module-local-ai-models --test fixtures --test contract`; the golden diff reviewed; the
  module's coverage at or above 90 %.

## Conclusions

Built with 02/02. Seven homes: `-not-claimed`, `-no-blobs`, `-kept` (download), `-unreadable`, `-hidden`, `-long`
(a 4,097-byte manifest), `-offered`; one golden `local-ai-models-ollama.json`. Beyond the brief: the not-claimed home
was split in two (an Ollama folder with `blobs/` and no `manifests/` is now plan 023's home, `-no-blobs` is the other
way round); `-offered` has the incomplete model `broken:1` share a layer with llama3, asserted through real files;
the contract test runs every Ollama home. Module coverage after the fixes: 99.26 % of lines.
