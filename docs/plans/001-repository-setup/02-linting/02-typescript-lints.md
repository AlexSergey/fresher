# 001/02/02 — TypeScript lints

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** app, repo

## Files

- `eslint.config.ts` — ported from `prod-forge/backend/eslint.config.ts` (same plugins, rules and structure:
  typescript-eslint `strictTypeChecked` + `stylisticTypeChecked`, Prettier, perfectionist `recommended-natural`, regexp,
  sonarjs picks with cognitive complexity 20, unicorn picks, check-file kebab-case, import-lite no default export,
  no-only-tests, `@eslint/json`, `eslint-plugin-package-json`, `eslint-config-flat-gitignore`), with:
  - `eslint-plugin-jest` replaced by `@vitest/eslint-plugin` (the same rule picks where they exist);
  - added `eslint-plugin-react` (`recommended`, `jsx-runtime`), `eslint-plugin-react-hooks` (`recommended`),
    `eslint-plugin-jsx-a11y` (`recommended`), `eslint-plugin-testing-library` (`react`) on unit tests,
    `eslint-plugin-playwright` on `app/ui/e2e/`;
  - NestJS-only overrides removed; `no-console` stays an error; default exports allowed only in config files.
- `tsconfig.eslint.json` (root) covering `app/ui`, `scripts/` and the config files; `eslint-plugin.d.ts` for untyped
  plugins.
- `knip.json` — entries: `app/ui/src/main.tsx`, `scripts/*.ts`, configs; workspace `app/ui`.
- Root scripts: `lint:ts` (`tsc --noEmit -p app/ui`), `lint:code` (`eslint .`), `lint` (Rust + TS), `format`
  (`prettier --write` + `eslint --fix` + `cargo fmt`), `knip`.

## Checks

- A probe component with `any`, a default export, an unsorted import and a missing hook dependency fails `npm run lint`
  with each rule named; the probe is removed.
- `npm run lint` and `npm run knip` pass on the skeleton.
