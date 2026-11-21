# 024/04/01 — The docs

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/README.md` — the crate list with `fresher-module-unused-toolchains`; the core's ports with
  `SmallFiles`.
- `docs/architecture/modules.md` — the `small_file` row and its rule of use (01/01), checked; the module
  `unused-toolchains` in the list of examples: code with a data table of managers, whole claims of the version folders,
  item claims of the managers' pointer files and marker claims of the pins, the files read in `finish` through
  `small_file`, the projects' artifact folders used to skip a dependency's pin, the needs on the three guarded folders,
  the Node switch off by default, the Trash rather than a tool command, the other managers deferred and why.
- `docs/restrictions/privacy.md` — "Local data only": a module may read a small text file it matched (at most 4 KiB,
  never a link); its bytes are never logged, stored or sent (enforced by `SystemFacts::small_file`'s contract and
  review).
- `docs/testing.md` — `text` and `[settings]` (01/02), the four case files (03/01) and `unused_toolchains.rs` (03/02),
  checked.
- `docs/manual-testing.md` — 04/02's runs by hand as items of the consolidated checklist, with links to the step.
- `docs/categories/unused-toolchains-*.md` — as built (the table's rows, the texts, the decision); the catalogue's rows.
- `docs/ideas/unused-toolchain-versions.md` — its line on plan 024: what was built; what is deferred (the other
  managers) and the answers to its open questions. `docs/ideas/README.md`: its row.
- `docs/deferred/other-toolchain-versions.md` — checked against what was built (its "what doing it needs");
  `docs/deferred/ollama-models.md` — its first need marked built by plan 024 (`small_file`), checked.
- `docs/plans/024-unused-toolchains/` — every step's conclusions written as it was built (checked here); the design's
  tables brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves.

## Conclusions

Done: the architecture, privacy, testing and manual-testing docs, the category docs, the idea and the deferred
notes follow the build. The user's rule of 2026-11-21 (scan timing only when a plan changes the walk or the scanner)
removed the scan-time item the step first added to the manual list. The final fix wave and the first commit of 04/02
rewrapped the lines the review found over 120 columns.
