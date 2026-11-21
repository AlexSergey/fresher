# 023/03/01 — The docs

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/README.md` — the crate list with `fresher-module-local-ai-models`.
- `docs/architecture/modules.md` — the module `local-ai-models` in the list of examples: code with a data table of
  stores, one whole claim per store, the model's name read from its folder, the decision in `finish` (the running apps for
  LM Studio), the Trash rather than a tool command, Ollama deferred and why.
- `docs/testing.md` — the two case files (02/01) and `local_ai_models.rs` (02/02), checked.
- `docs/manual-testing.md` — 03/02's runs by hand as items of the consolidated checklist, with links to the step.
- `docs/categories/local-ai-models-*.md` — as built (the table's rows, the texts, the decision); the catalogue's rows.
- `docs/ideas/local-ai-models.md` — its line on plan 023: what was built; what stays an idea (other apps' models, the
  Hugging Face chunk cache, the tools' own commands) and what is deferred (Ollama). `docs/ideas/README.md`: its row.
- `docs/deferred/ollama-models.md` — checked against what was built (its "what doing it needs").
- `docs/plans/023-local-ai-models/` — every step's conclusions written as it was built (checked here); the design's
  tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves.

## Conclusions

The architecture list and `modules.md` name the module and its pattern; `testing.md` got the UI scan and its two
scripts beside the case files and the core flows of the earlier steps; `manual-testing.md` has the sandbox runs (1.9)
and the real Mac's (2.8) with the table rows, the scan-time item moving to 2.9 and the guards sandbox to 1.10; the
design and the UI step say Scan's cards and Details, not a module page. The category files, the catalogue rows, the
idea and its row, and the deferred Ollama item were checked against the code and needed no change. The plans'
conclusions of the module, end-to-end and finish plans and of the global plan are written in 03/02.
