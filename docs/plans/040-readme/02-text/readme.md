# 040/02 — The text

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, scripts, ci, docs

The new `README.md` in the design's order, with the generated block between its markers; the stack table moved into
`docs/README.md`; `npm run check:readme` in `pre-push` and in CI. The design: [`../00-design.md`](../00-design.md),
section 1.

## Steps

| #   | Step                                          | Status      | Shared files                                                              |
| --- | --------------------------------------------- | ----------- | ------------------------------------------------------------------------- |
| 01  | [The new README and its check](./01-readme.md) | 🔨 in progress | `README.md`, `package.json`, `.github/workflows/ci.yml`, `docs/README.md`, `docs/quality.md` |

## Conclusions

Done. The README is rewritten; the check runs in `pre-push` and CI. See [`01-readme.md`](./01-readme.md).
