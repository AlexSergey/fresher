# 001/02/03 — Repository guards

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo

## Files

- `scripts/check-staged.ts` — fails on a staged file over 1 MiB, a `*.sqlite`/`*.db`, a `.env` (except
  `.env.example`) or a signing key; run with Node's native TypeScript support; `scripts/check-staged.test.ts` (`node
  --test`). One recorded exception to the size limit: `app/tauri/icons/icon.icns` (1.03 MB — the 1024 px Retina size
  macOS needs), listed by path in the script and in `docs/hacks/`.
- Root scripts: `check:staged`, `check:secrets` (`gitleaks git --pre-commit --staged --redact --no-banner`),
  `check:typos` (`typos`), `check:repo` (all three), `test:scripts`.

## Checks

- Staging a 2 MiB file, a fake AWS key, a `test.sqlite` and a misspelled word each fails `npm run check:repo`.
- `npm run test:scripts` passes.
