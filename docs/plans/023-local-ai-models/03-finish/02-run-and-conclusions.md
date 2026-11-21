# 023/03/02 — The runs by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs, modules

A checklist for the user, then the conclusions.

## In the sandbox

- [ ] `rm -rf sandbox/local-ai-models && npm run dev:sandbox -- local-ai-models`. Scan; walk `EXPECTED.md` line by
  line, negatives first: nothing of a negative is offered (the Space, the locks, the folder without blobs, the name off
  the shape, the loose file, Ollama's store, the project's model file, the project's link); the socket case reads as
  kept with its reason; each positive is offered in its category at its danger (Hugging Face Low, LM Studio Medium);
  titles read `meta/tiny`, `gpt2`, `o/words`, `pub/model-a`; each reason names a date and a size.
- [ ] The storage map: the offered models in the explained part; `home/.ollama/models` in your folders.
- [ ] Select everything; Review names Medium; confirm to the Trash: the model folders land in `home/.Trash` with their
  `snapshots` links inside; `home/Code/linked/weights.safetensors` is still a link (now broken). Cleanups → put back:
  the folders are back and the link resolves again.
- [ ] `npm run dev:sandbox -- local-ai-models-lm-studio-open`: the LM Studio model is kept "LM Studio is open"; the
  Hugging Face model is offered.
- [ ] Ignore the LM Studio category, then the module; restore.

## On the developer's Mac (a dry run)

`npm run dev`, a scan of `~` (a development build on the real Mac makes every cleanup a dry run). Compare with these
read-only commands:

```sh
ls ~/.cache/huggingface/hub ~/.lmstudio/models/* ~/.cache/lm-studio/models/* 2>/dev/null
du -sh ~/.cache/huggingface/hub/models--* ~/.cache/huggingface/hub/datasets--* 2>/dev/null
du -sh ~/.lmstudio/models/*/* ~/.cache/lm-studio/models/*/* 2>/dev/null
find ~/.cache/huggingface/hub -maxdepth 2 -name snapshots -prune -o -maxdepth 2 -type d -print | head
osascript -e 'id of app "LM Studio"'
```

- [ ] Every offered model checked by hand: a repository folder with `blobs` and `snapshots`, or a folder two levels
  down in LM Studio's models folder; its size against `du`; its title against the folder's name.
- [ ] Nothing offered outside the three places; a Space or a non-model folder of the hub is not offered.
- [ ] The facts "to verify":
  - LM Studio's bundle id (the `osascript` line) against `stores.toml`; with LM Studio open, its models are kept;
  - LM Studio's older place `~/.cache/lm-studio/models` on a Mac that has it (or note that none was found);
  - whether LM Studio records when a model was last used (its folder under `~/.lmstudio`), for the date in the reason
    (agent's call 8);
  - a Hugging Face download in progress (a big model, cancelled half way): what the scan offers and what the cleaner's
    rehearsal says while the download runs;
  - an LM Studio model imported by hand (`lms import` or the app's import): it reads as a download, at Medium, with
    the "added yourself" sentence.
  - LM Studio may keep a background server running after its window closes: whether the running-app check sees it (the
    cleaner's open-file check still protects a loaded model).
  The table, texts and category docs are corrected where they differ.

## Files

- `docs/plans/023-local-ai-models/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was
  built, what changed from the design and why, the numbers (tests, coverage, what the module offered
  on the developer's Mac by category), the calls taken while building, what the real Mac showed.
- `modules/local-ai-models/stores.toml`, `locales/en.json`, `docs/categories/local-ai-models-*.md` — corrected where
  the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

The agent's part is done: a final review fix commit (the vacuous restore assertion dropped, two comments reworded, the
manual-testing and architecture sentences made exact, the scan-time check dropped from this step, the background
server of LM Studio added to the facts "to verify"), then the checks below on the branch's last code commit. Every box
above is the user's and is **pending**: the sandbox walk, the dry run on the real Mac and the facts "to verify". The
plan stays 🔨 in progress until the user has looked at the result. The conclusions of the whole plan are in the
[global readme](../readme.md#conclusions).

