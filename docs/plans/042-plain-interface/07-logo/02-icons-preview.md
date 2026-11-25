# 042/07/02 — The app icons and the social preview

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/tauri (icons), repo (assets), docs

The app's icons and the repository's social preview made from the vector logo. Source:
[`docs/mockups/0.1.0/logo.svg`](../../../mockups/0.1.0/logo.svg) (800 × 800, the chosen variant A, already behind
`assets/logo.png`).

## Read first

`app/tauri/icons/` (32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.png), `app/tauri/tauri.conf.json`
(`bundle.icon`), `app/tauri/tests/config.rs` (what it checks of the icons), `assets/social-preview.svg` (embeds
`logo.png`), `docs/plans/040-readme/03-github/02-social-and-settings.md` (how the preview was rendered),
`docs/dependencies.md` ("Tools on the machine": `rsvg-convert`; the Tauri CLI).

## Files

- `app/tauri/icons/{32x32.png,128x128.png,128x128@2x.png,icon.icns,icon.png}` — made in the scratchpad, never in the
  repository: `rsvg-convert --width 1024 --height 1024 docs/mockups/0.1.0/logo.svg -o <scratch>/logo-1024.png`, then
  `npm run tauri icon <scratch>/logo-1024.png -- --output <scratch>/icons`; copy only the five files above (the
  command makes more: Windows, iOS and Android sizes, which the app does not ship). Look at each PNG, and at the
  `.icns` with `sips -g pixelWidth` (read-only). Nothing else in `app/tauri/` changes.
- `assets/social-preview.png` — rendered again from `assets/` with `rsvg-convert --width 1280 --height 640
  social-preview.svg -o social-preview.png` (it embeds the new `logo.png`); under 1 MB (`ls -l`). The SVG does not
  change.
- `docs/dependencies.md` — the `rsvg-convert` row: "renders `assets/social-preview.svg` and the app icons' source
  from `docs/mockups/0.1.0/logo.svg`, by hand; not needed to build or test".

## Tests

None new: `app/tauri/tests/config.rs` (the icons named in the config exist) stays green.

## Checks

`CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-app --test config`; `git status` shows only the five icons,
the preview PNG and `docs/dependencies.md`; each image looked at.

Commit: `chore(repo): the app icons and the social preview from the vector logo` (scope `repo`).

## Conclusions

- The five app icons in `app/tauri/icons/` and `assets/social-preview.png` (47 KB) from `logo.svg`; the
  dependencies table updated. The icon fills its canvas without the usual rounded tile: on the user's list.
