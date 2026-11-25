# 03/01 — The licence notices

**Status:** 🔨 in progress (awaiting the user's look) · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, ci, scripts, app/ui, docs

Closes the deferred "Licence notices" and the known issue "No licence check of npm dependencies yet" (both removed).

## Files

- `about.toml` — cargo-about's config: macOS targets, no dev-dependencies, no workspace crate, the allowlist and the
  five MPL-2.0 crates of `deny.toml`.
- `scripts/notices-build.ts` (pure: the policy from `deny.toml`, the `about.toml` mirror check, SPDX expressions, the
  Rust and npm entries, the grouping) and `scripts/notices.ts` (runs cargo-about offline, reads `package-lock.json`
  and `node_modules`, writes or compares the file); their tests `scripts/notices-build.test.ts`,
  `scripts/notices.test.ts`.
- `package.json`: `notices:generate`, `check:notices`, the latter in `pre-push`; `.github/workflows/ci.yml`: cargo-about
  0.9.3 installed, `cargo fetch --locked`, `npm run check:notices` in `lint`; `_typos.toml` skips `*.generated.json`.
- `app/ui/src/screens/settings/notices.generated.json` — the generated notices (about 330 KB).
- `app/ui/src/product/notices-dialog/` — the dialog, its styles, tests and gallery stories (`open`, `reading`);
  `app/ui/src/screens/settings/use-notices.ts` (the dynamic import) and its test; `about-group.tsx` — the line and the
  button; `locales/en.json` — `ui.notices.*`, `ui.screen.settings.about.notices`.
- e2e: `flows.spec.ts` "the open-source licences open from About, bundled into the app"; the gallery baselines
  `NoticesDialog/open` and `NoticesDialog/reading`.
- Docs: `dependencies.md` (cargo-about, the notices), `restrictions/dependencies.md`, `restrictions/repository.md`,
  `quality.md`, `testing.md`, the deferred and known-issues indexes.

## Checks

- `npm run check:notices` passes; editing one character of the generated file, or adding a GPL npm package to the
  production tree, makes it fail (the tests do both on temporary folders).
- `npm run test:scripts`, the UI tests, `npm run pre-push` and the affected e2e specs pass.
- The MPL-2.0 crates `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext` and `selectors` are listed with their
  crates.io addresses; the npm part lists `@tauri-apps/api`, `lucide-react`, `react`, `react-dom` and `scheduler`.

## For the user's look

Settings → About → "Show the open-source licences": the wording, the folded rows, the MPL-2.0 addresses shown as text.
