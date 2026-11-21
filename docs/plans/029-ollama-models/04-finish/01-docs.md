# 029/04/01 — The docs

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/removal.md` — the together kind, the flow and the threat model's row (01/02), checked; put back's
  order for a together item (01/03).
- `docs/architecture/modules.md` — "Removal" in a module's finding: `Removal::MoveTogether` (two to 64 paths, the first
  the subject, every part a matched path); the `local-ai-models` example: Ollama's row with its own shape, the
  manifests read through `small_file`, the layers shared between models, the store's and the model's rows; the
  sentence that Ollama is deferred removed.
- `docs/architecture/README.md` — the data flow's removal: a finding's removal may be several paths.
- `docs/restrictions/deletion-safety.md` — "Explicit confirmation only": an item may be several paths, listed in the
  confirmation on demand, all checked before the first moves; "Re-check before delete": every part; "Trash first": a
  together item's parts all go the same way, put back subject last. Each with its test names.
- `docs/restrictions/privacy.md` — the module reads Ollama's manifests (at most 4 KiB, never a link), never logs, keeps
  or sends them (plan 024's line, checked).
- `docs/testing.md` — the case files (03/01), `local_ai_models_ollama.rs` (03/02), the together cases of `never.rs`
  (01/02), checked.
- `docs/manual-testing.md` — 04/02's runs by hand as items of the consolidated checklist, with links to the step; plan
  023's line on Ollama's store updated.
- `docs/categories/local-ai-models-ollama.md` and the catalogue's row (02/02), checked.
- `docs/ideas/local-ai-models.md` — its Ollama line: built by plan 029; "freed bytes, not listed bytes" answered;
  `docs/ideas/README.md` — its row.
- `docs/plans/023-local-ai-models/` and `docs/plans/024-unused-toolchains/` — their text stays as written (history);
  only their links to the removed `docs/deferred/ollama-models.md` point at this plan instead, so the link check
  passes.
- `docs/plans/029-ollama-models/` — every step's conclusions written as it was built (checked here); the design's
  tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves (none points at `docs/deferred/ollama-models.md` any more).

## Conclusions

Done as the step said: the removal architecture (put back of a together item, its known limit, the together kind in
the checklist and the threat table), the modules page and README as built, deletion-safety and privacy (a together item
per rule with the tests that prove it, the manifests read), testing (the never property with together items, the harness
and the module paragraphs), the manual-test list (1.12, 2.11), the idea's Ollama line, the known issue about
`sandbox -- all`, the category doc, and the design's as-built tables. Checked, not rewritten: the catalogue row, the
ideas row, the deferred item's removal (no link points at it; three plain-text mentions in 023 and 024 stay as
history). The docs were folded into the final whole-branch review. After it, five long lines were rewrapped and the
partial-act texts documented in the design and `removal.md`.
