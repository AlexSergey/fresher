# 029/04/02 — The runs by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs, modules

A checklist for the user, then the conclusions. The user's walk is pending until the user has looked at the result.

## In the sandbox

- [ ] `rm -rf sandbox/local-ai-models-ollama && npm run dev:sandbox -- local-ai-models-ollama`. Scan; walk
  `EXPECTED.md` line by line, negatives first: the lone layer and the manifest at the wrong depth are not offered; the
  incomplete model reads as kept with its reason; `llama3:latest, llama3:8b` is one finding; the derived model's reason
  says how large it is in all, how much is freed and that the rest is shared with the base; every Ollama finding is
  Medium.
- [ ] The storage map: the offered models' layers in the explained part; the shared layers and the lone layer in your
  folders.
- [ ] Select the derived model and `mistral:7b`; the confirm dialog names Medium; open "more files" on each: exactly
  the manifest's companions the golden lists. Confirm to the Trash: the manifests and the unshared layers land in
  `home/.Trash`; the shared layers and the base's manifest stay. Cleanups → put back: every file is back.
- [ ] `npm run dev:sandbox -- local-ai-models-ollama-unreadable`: nothing of Ollama offered, the card says a model list
  could not be read.
- [ ] `npm run dev:sandbox -- local-ai-models-ollama-download`: nothing of Ollama offered, the card says a download is
  in progress.

## On the developer's Mac (a dry run)

`npm run dev`, a scan of `~` (a development build on the real Mac makes every cleanup a dry run). Compare with these
read-only commands:

```sh
ollama list
find ~/.ollama/models/manifests -type f | sed 's|.*/manifests/||'
find ~/.ollama/models/manifests -type f -exec wc -c {} + | sort -n | tail -3
ls ~/.ollama/models/blobs | grep -c partial
du -sh ~/.ollama/models/blobs
```

- [ ] Every offered model is in `ollama list` with the same tags; aliases (the same id in `ollama list`) are one
  finding; each freed size checked against the layers its manifest names that no other manifest names.
- [ ] Select an Ollama model, confirm: the dry run checks every part and ends "a dry run"; with a model loaded
  (`ollama run <model>` in another window), the item is skipped as in use.
- [ ] The facts "to verify", written here and corrected in the code and the texts where they differ:
  - the largest manifest's size (the `wc -c` line) against the 4 KiB limit;
  - the default host and namespace in `ollama list`'s names (the titles);
  - partial download names while `ollama pull` runs (in a throwaway `OLLAMA_MODELS` folder, never the real store);
  - that Ollama's server deletes layers no manifest names at its start (in the throwaway folder: remove a manifest by
    hand, restart `ollama serve`, list `blobs/`), and that `ollama list` no longer shows a model whose manifest is gone;
  - what a model loaded while its files move does (in the throwaway folder).
- [ ] No scan bench: the walk did not change (`git diff` of the acceptance is empty).

## Files

- `docs/plans/029-ollama-models/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built,
  what changed from the design and why, the numbers (tests, coverage, what the module offered on the developer's Mac,
  the largest manifest), the calls taken while building, what the real Mac showed.
- `modules/local-ai-models/{stores.toml,locales/en.json}`, `docs/categories/local-ai-models-ollama.md` — corrected
  where the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Written without the user's walk: the automated checks and the conclusions are done, the boxes above are **unchecked on
purpose** (the sandbox walks and the dry run on the real Mac are the user's). The conclusions, the numbers and the
facts "to verify" are in [the global readme](../readme.md#conclusions); each fact below is checked against the code
and the texts when the user reports what the real Mac shows. The status stays 🔨 in progress until then.

**Facts "to verify", in addition to the five listed above** (the final review could not judge them from the code):

- hidden tag files (a manifest whose name starts with a dot) and tag files whose name is not UTF-8 are ignored, not read
  and not claimed; whether Ollama lists such a model decides whether this matters;
- a manifests folder the user owns but cannot list (a permission the walk cannot read): the module should leave the
  store out as `ollama-unreadable`; check what the walk reports on a real folder of that kind;
- Ollama's pruning of layers a "only part of it went" item left behind: a model that ended partly moved leaves layers
  no manifest names, and Ollama may delete them at its start (the same behaviour as the fact above);
- Ollama deleting unnamed layers at its start, so that two models sharing a base, both removed, lose the base layer
  outside the Trash and their put back needs `ollama pull`;
- a manifest over 4 KiB blocks every Ollama offer: the largest manifest on this Mac against the limit.
