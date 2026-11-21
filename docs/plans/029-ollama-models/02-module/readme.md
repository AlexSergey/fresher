# 029/02 — Ollama in the module

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** modules, testing, docs

Ollama's store as a row of `local-ai-models` with a shape of its own: the manifests read in `finish`, the layers each
model uses, the store's and the model's rows, one finding per model removed together. The design:
[`../00-design.md`](../00-design.md), sections 5 to 9.

## Steps

| #   | Step                                                       | Status      | Shared files                                                                                   |
| --- | ---------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------- |
| 01  | [The store and the manifests](./01-store-and-manifests.md) | 🔨 in progress | `modules/local-ai-models/{stores.toml,Cargo.toml}`, `src/{stores,name,gathered,lib}.rs`, `docs/dependencies.md` |
| 02  | [The layers and the decision](./02-layers-and-decision.md) | 🔨 in progress | `modules/local-ai-models/{locales/en.json}`, `src/{categories,report,lib}.rs`, `docs/categories/` |
| 03  | [Fixtures and the contract](./03-fixtures.md)              | 🔨 in progress | `fixtures/homes/`, `fixtures/golden/`, `modules/local-ai-models/tests/`                        |

01 (needs only 024's `small_file`), then 02 (needs 01/01's `Removal::MoveTogether`), then 03.

## Conclusions

The module offers Ollama's models as findings of one removal each; built and reviewed, with the pure parts first and
the row, texts, category doc and fixtures in one commit. The module's coverage is 99.26 % of lines (floor 90 %); see
the measured numbers in [the global readme](../readme.md#conclusions). The changes from the design are in each step's
conclusions; the largest are the fail-closed store rows for a link or hidden folder in the manifests' tree.
