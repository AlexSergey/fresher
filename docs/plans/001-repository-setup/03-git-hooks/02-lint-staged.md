# 001/03/02 — lint-staged

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo

## Files

- `.lintstagedrc.cjs` — as in [`../00-design.md`](../00-design.md), "lint-staged".
- `.husky/pre-commit` — `npm run check:repo`, then `npm run pre-commit` (`lint-staged --config .lintstagedrc.cjs`).

## Checks

- A staged Markdown-only change runs Prettier and nothing else (no cargo, no `tsc`).
- A staged `.rs` file with bad formatting is fixed by rustfmt and re-staged; one with a Clippy warning fails the commit.
- A staged `.tsx` file with an ESLint error fails; a type error in an unstaged file still fails through `tsc` (the
  whole-project check), as intended.
- A staged `Cargo.toml` change runs `cargo deny check` and `cargo machete`.
