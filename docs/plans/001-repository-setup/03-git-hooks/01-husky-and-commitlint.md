# 001/03/01 — Husky and commitlint

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo

## Files

- `husky` (exact version) with `prepare: "husky"`.
- `.commitlintrc.cjs` — `@commitlint/config-conventional`; `type-enum`: `ci`, `chore`, `docs`, `feat`, `fix`, `perf`,
  `refactor`, `revert`, `style`, `test`; `scope-enum`: `core`, `rules`, `scanner`, `store`, `cleaner`, `app`, `ui`,
  `e2e`, `deps`, `docs`, `repo`, `ci`.
- `.husky/commit-msg` — prod-forge's: reject `Co-Authored-By` / `Generated-By` / `Assisted-By` trailers naming
  Claude or Anthropic, then `npm run lint:commit -- "$1"`.
- Root script `lint:commit` (`commitlint --config .commitlintrc.cjs --edit`).

## Checks

- `git commit -m "update stuff"` fails; `git commit -m "chore(repo): add commitlint"` passes; a message with a
  `Co-Authored-By: Claude …` trailer fails.
