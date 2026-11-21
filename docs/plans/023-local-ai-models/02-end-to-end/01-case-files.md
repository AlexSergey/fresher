# 023/02/01 — The case files

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. Two files, since an open app is shared by every case of a file (as
`dev-caches-xcode-open.toml`). The design: [`../00-design.md`](../00-design.md), section 7.

## Files

- `fixtures/sandboxes/local-ai-models.toml` (new) — `feature = "local-ai-models"`, `module = "local-ai-models"`; a
  head comment: what the file covers, that blobs and model files are small with `fill = "data"`, that snapshot links
  are `[[case.symlink]]` with relative targets inside the model folder. With these exact names:
  - negative, in this order:
    - "a Space in the hub is never claimed" — `home/.cache/huggingface/hub/spaces--o--demo/blobs/b` and
      `snapshots/r/app.py` → `../../blobs/b`;
    - "the hub's locks are never claimed" — `home/.cache/huggingface/hub/.locks/models--o--n/x.lock`;
    - "a folder without blobs and snapshots is never claimed" — `home/.cache/huggingface/hub/models--o--empty/refs/main`;
    - "a name off the repository shape is never claimed" — `home/.cache/huggingface/hub/models--a--b--c/blobs/b` and
      its `snapshots/r/f`;
    - "a loose file in LM Studio's models folder is never claimed" — `home/.lmstudio/models/pub/loose.gguf`;
    - "Ollama's models are not claimed in this version" — `home/.ollama/models/manifests/registry.ollama.ai/library/tiny/latest`
      and `home/.ollama/models/blobs/sha256-0001`; `[case.map]` `your-folders` at `home/.ollama/models`;
    - "a model file in a project is never claimed" — `home/Code/app/models/tiny.gguf`;
    - "a model with a socket inside is kept" — `home/.cache/huggingface/hub/models--o--busy/{blobs/b,snapshots/r/f}`
      and a socket `home/.cache/huggingface/hub/models--o--busy/blobs/s` (`kind = "socket"`); left out
      `local-ai-models.left-out.special` 1 in `local-ai-models.hugging-face-models`;
    - "a project's link into the cache stays a link" — `home/Code/linked/weights.safetensors` →
      `../../.cache/huggingface/hub/models--o--shared/blobs/w`, the blob of the model the positive "the model a
      project links to goes, the link stays" writes; after the cleanup the link is still in `home/Code/linked` as a
      link;
  - positive:
    - "a Hugging Face model goes" — `home/.cache/huggingface/hub/models--meta--tiny/` (`blobs/aa`, `blobs/bb`,
      `refs/main`, `snapshots/r1/config.json` → `../../blobs/aa`, `snapshots/r1/model.safetensors` →
      `../../blobs/bb`); `home/.cache/huggingface/hub/models--meta--tiny` in
      `local-ai-models.hugging-face-models`, low, `in-trash`; `[case.map]` `explained`;
    - "a model without an owner goes" — `home/.cache/huggingface/hub/models--gpt2/{blobs/b,snapshots/r/f}`; low,
      `in-trash`;
    - "the model a project links to goes, the link stays" —
      `home/.cache/huggingface/hub/models--o--shared/{blobs/w,snapshots/r/weights.safetensors}` (the snapshot a link
      to `../../blobs/w`); `home/.cache/huggingface/hub/models--o--shared` in `local-ai-models.hugging-face-models`,
      low, `in-trash`;
    - "a Hugging Face dataset goes" — `home/.cache/huggingface/hub/datasets--o--words/{blobs/b,snapshots/r/data.txt}`;
      in `local-ai-models.hugging-face-datasets`, low, `in-trash`;
    - "an LM Studio model goes, at Medium" — `home/.lmstudio/models/pub/model-a/model.gguf`;
      `home/.lmstudio/models/pub/model-a` in `local-ai-models.lm-studio`, medium, `in-trash`;
    - "a model in LM Studio's older place goes" — `home/.cache/lm-studio/models/pub/model-b/model.gguf`; medium,
      `in-trash`.
- `fixtures/sandboxes/local-ai-models-lm-studio-open.toml` (new) — `[[case.running]]` LM Studio with its bundle id
  (as 01/01's table); negative "LM Studio's models are kept while it is open" —
  `home/.lmstudio/models/pub/open-model/model.gguf`; left out `local-ai-models.left-out.app-open` 1 in
  `local-ai-models.lm-studio`; positive "a Hugging Face model still goes while LM Studio is open" —
  `home/.cache/huggingface/hub/models--o--other/{blobs/b,snapshots/r/f}`; low, `in-trash`. File paths apart from
  `local-ai-models.toml`'s.
- `fixtures/golden/sandbox-local-ai-models.json`, `sandbox-local-ai-models-lm-studio-open.json` (new) —
  `UPDATE_GOLDEN=1`, reviewed: the findings, the left-out counts, the map, the cleanup's outcome, the tree after (the
  project's link still a link).
- `app/tauri/tests/sandboxes.rs` — `local_ai_models => "local-ai-models"` and
  `local_ai_models_lm_studio_open => "local-ai-models-lm-studio-open"` in `case_files!`, sorted by file name.
- `docs/testing.md` — the two case files in the list of case files.

The negative "a project's link into the cache stays a link" owns only `home/Code/linked`; its target model is the
positive "the model a project links to goes, the link stays" (one world: a folder is written by one case only). What
the code does for each case is written down when the file is made; a case that differs from the design is fixed in the
module, not in the case file.

## Produces

The module's case files, for 03/02 (the run by hand).

## Checks

- `npm run sandbox:test` passes for both files and every other case file (their goldens unchanged).
- Once per kind, by hand, written in this step's conclusions: a danger changed (`medium` → `low` on the LM Studio
  case) and a left-out's items changed (`1` → `2` on the socket case) each make the autotest fail; then reverted.
- `npm run sandbox -- local-ai-models` writes the sandbox; `EXPECTED.md` lists every case, negatives first.
- `cargo nextest run -p fresher-sandbox -p fresher-app`.

## Conclusions

Built as written: `local-ai-models.toml` and `local-ai-models-lm-studio-open.toml` with their goldens; the sandbox
test passes for both, twice with the same result. The case files carry no age and no `changed` date: the goldens hold
no dates, and the sandbox's fixed `now` and fake clock keep them deterministic.
