# 040/01 — The generator

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** scripts, docs

The `**Example:**` line in every category file, the script that writes the README's "What it finds" block from the
catalogue and checks it, and the link and image check. Nothing is wired into `pre-push` yet: the README gets its
markers in 02/01, which wires the check. The design: [`../00-design.md`](../00-design.md), sections 2, 3 and 5.

## Steps

| #   | Step                                                      | Status      | Shared files                                    |
| --- | --------------------------------------------------------- | ----------- | ----------------------------------------------- |
| 01  | [The example line in every category](./01-examples.md)    | 🔨 in progress | `docs/categories/` (every file and the README)  |
| 02  | [The table's generator and check](./02-generator.md)      | 🔨 in progress | `package.json`                                  |
| 03  | [The link and image check](./03-link-check.md)            | 🔨 in progress | `scripts/readme.ts` (02's)                      |

In order: 01, then 02, then 03.

## Conclusions

Done. Three steps, three script files (`readme-catalogue.ts`, `readme-links.ts`, `readme.ts`) and their tests. All
sixteen problem kinds of the design are produced and tested; the by-hand checks are in 02/01's conclusions.
