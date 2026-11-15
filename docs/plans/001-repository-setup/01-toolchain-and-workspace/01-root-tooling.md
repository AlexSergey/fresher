# 001/01/01 — Root tooling

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo

## Files

- `package.json` — `private`, `type: module`, `workspaces: ["app/ui"]`, `engines` (`node >= 24.15.0`,
  `npm >= 11`), exact devDependency versions; scripts as placeholders filled by later steps (`lint`, `format`, `test`,
  `test:cov`, `test:e2e`, `pre-commit`, `pre-push`, `prepare`, `check:*`).
- `.npmrc` — `save-exact=true`, `engine-strict=true`.
- `.nvmrc` — `24`.
- `.prettierrc` — prod-forge's: `singleQuote`, `trailingComma: all`, `semi`, `printWidth: 120`, `endOfLine: lf`.
- `.prettierignore` — lockfiles, `target/`, `dist/`, `coverage/`, `app/tauri/gen/`, `fixtures/golden/`.
- `.taplo.toml` — bg's: `column_width = 120`, two-space indent, no key reordering; excludes `target/`, `node_modules/`.
- `_typos.toml` — excludes lockfiles, `fixtures/`; a short commit hash in backticks is not a word.

## Done when

- `npm install` succeeds with the lockfile created; `npx prettier --check .` and `taplo fmt --check` pass.
- `typos` passes over the repository.
