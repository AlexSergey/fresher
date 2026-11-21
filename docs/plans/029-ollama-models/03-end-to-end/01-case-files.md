# 029/03/01 — The case files

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the Ollama store ships its case files, negatives first, with their
golden reports and left-out expectations. Three files, since a store-wide left-out blocks every case of the file that
holds it (the store is one folder of one home). Manifests use plan 024's `text`. The design:
[`../00-design.md`](../00-design.md), sections 7 and 10.

## Files

- `fixtures/sandboxes/local-ai-models-ollama.toml` (new) — `feature = "local-ai-models-ollama"`, `module =
  "local-ai-models"`; a head comment: the store's layout, that manifests are `text` and layers small files with
  `fill = "data"`, why the store-wide cases are in their own files. With these exact names:
  - negative, in this order:
    - "a layer no manifest names is never claimed" — `home/.ollama/models/blobs/sha256-<a>` alone;
    - "a manifest at the wrong depth names nothing" — `home/.ollama/models/manifests/registry.ollama.ai/library/odd`
      (a valid manifest text);
    - "a layer another model uses stays when its model goes" — the base and the derived model sharing two layers: the
      derived model's finding has only its own layers among its parts (checked in the golden), and after the cleanup
      the shared layers and the base's manifest are in place;
    - "a model with a layer of the wrong size is kept" — left out `local-ai-models.left-out.ollama-incomplete` 1 in
      `local-ai-models.ollama`;
  - positive:
    - "two tags with the same files go as one model" — `llama3/latest` and `llama3/8b`; one finding, medium,
      `in-trash`: both manifests and the model's layers in `home/.Trash`;
    - "a model with no sharer goes with its layers, at Medium" — `mistral/7b`; medium, `in-trash`; `[case.map]`
      `explained` for its layers;
    - "a model in another namespace is titled with it" — `registry.ollama.ai/someone/tiny/q4`; medium, `in-trash`.
- `fixtures/sandboxes/local-ai-models-ollama-unreadable.toml` (new) — negative "an unreadable manifest keeps every
  Ollama model" — one manifest `text = "{"`, a valid model beside it; left out
  `local-ai-models.left-out.ollama-unreadable` 1; nothing offered.
- `fixtures/sandboxes/local-ai-models-ollama-download.toml` (new) — negative "a download in progress keeps every Ollama
  model" — `blobs/sha256-<c>-partial` beside a valid model; left out `local-ai-models.left-out.ollama-download` 1.
- `fixtures/sandboxes/local-ai-models.toml` — the case "Ollama's models are not claimed in this version" removed, and
  the head comment's mention of Ollama; its golden regenerated and reviewed (only that case's lines change).
- `fixtures/golden/sandbox-local-ai-models-ollama.json`, `-ollama-unreadable.json`, `-ollama-download.json` (new) —
  `UPDATE_GOLDEN=1`, reviewed: the findings with their parts, the left-out counts, the map, the cleanup's outcome, the
  tree after (the shared layers and the base's manifest untouched).
- `app/tauri/tests/sandboxes.rs` — the three files in `case_files!`, sorted by file name.
- `docs/testing.md` — the three case files in the list; plan 023's line on Ollama's store updated.

The three files share one store folder (`home/.ollama/models`), so in `npm run sandbox -- all` a store-wide case
would block the positive file's models, and two files writing the same manifest path are refused. The two store-wide
files are therefore worlds of their own: their head comment says so, as `downloads-denied.toml`'s does, and the step
uses whatever the sandbox already offers for such a world (or, if it offers nothing, leaves them out of `all` with a
line in `docs/testing.md`), written in the conclusions. Model names differ between the files. What the code does for
each case is written down when the file is made; a case that differs from the design is fixed in the module, not in
the case file.

## Produces

The case files, for 04/02.

## Checks

- `npm run sandbox:test` passes for the three files and every other case file.
- Once per kind, by hand, written in this step's conclusions: a danger changed (`medium` → `low`) and a left-out's
  items changed (`1` → `2` on the incomplete case) each make the autotest fail; then reverted.
- `npm run sandbox -- local-ai-models-ollama` writes the sandbox; `EXPECTED.md` lists every case, negatives first.
- `cargo nextest run -p fresher-sandbox -p fresher-app`.

## Conclusions

Built: `local-ai-models-ollama.toml` (seven cases, negatives first), `-unreadable` and `-download` (one negative each)
and their goldens; the three case files are registered in `app/tauri/tests/sandboxes.rs`. Changes from the brief: the
shared-layer case is a positive case on the derived model's manifest (F6: a layer another model names makes the model
smaller, it is still offered); the "manifest at the wrong depth" sits one level below a tag's folder, since a plain file
above tag level now leaves the whole store out; the two tags finding's subject is the `8b` manifest (name order); the
two store-wide files write the lone layer of the base file, so `npm run sandbox -- all` leaves them out whole and names
them (F6b). Mutation checks, each failing as it should and reverted: a danger of medium changed to low ("the danger Low
is below the category's Medium") and the left-out items of the incomplete case changed from 1 to 2 ("expected 2 items,
got 1"). `npm run sandbox:test`: 59 of 59 at the time. The by-hand walk is the user's. Review: approved.
