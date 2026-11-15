# 001/03 — Git hooks

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo

Branch: `feat/001-03-git-hooks`. Layers 3–5 of the quality system ([`../00-design.md`](../00-design.md)).

## Steps

| #   | Step                                                 | Status  |
| --- | ---------------------------------------------------- | ------- |
| 01  | [Husky and commitlint](./01-husky-and-commitlint.md) | ✔️ done |
| 02  | [lint-staged](./02-lint-staged.md)                   | ✔️ done |
| 03  | [Pre-push](./03-pre-push.md)                         | ✔️ done |

## Conclusions

Implemented 2026-11-15.

- `npm install` installs the hooks (`prepare: husky`). The plan's own commits went through them.
- Probes, each a real `git commit` undone afterwards:
  - `commit-msg`: "update stuff", an unknown scope and a `Co-Authored-By: Claude` trailer are rejected; a
    Conventional Commit passes.
  - A Markdown-only change runs Prettier and nothing else.
  - A badly formatted `.rs` file is fixed by rustfmt, re-staged and committed.
  - An `unwrap()` fails the commit through Clippy; an `any` in a `.ts` file fails it through ESLint.
  - A `Cargo.toml` change runs `cargo deny check` and `cargo machete`.
- `npm run pre-push` passes: lint, knip, machete, deny and, until plan 04 adds the coverage line, the scripts' tests.
- `npm audit` reports 9 moderate dev-only advisories (`minimatch` through the lint plugins): recorded in
  `docs/known-issues/repo-moderate-audit-advisories.md`; the production audit is clean.
