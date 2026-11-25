# 040/03/02 — The social image, description and topics

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, docs

The image GitHub shows when the repository is shared, and the description and topics, written for the user to apply.
Nothing is changed on GitHub by this step. The design: [`../00-design.md`](../00-design.md), section 4, and agent's
call 9.

## Context

- GitHub's social preview is uploaded only through the repository's settings page (Settings → General → Social
  preview), 1280 × 640 recommended, under 1 MB. The description and topics can be set with `gh repo edit`; both are
  the user's to do (never run `gh repo edit` or any command that changes the repository on GitHub).
- Today on GitHub (read with `gh repo view --json`, 2026-11-22): description "Make Mac fresh again", no topics.
- On the developer's Mac: `rsvg-convert` 2.62.2 (librsvg, Homebrew). It is a development tool used by hand, never in
  the app or in CI.
- The look: Graphite and Iris (`docs/goal.md`, section 6): a near-black graphite background (`#141519` or the UI's
  background colour token, found with `grep -rn -- '--' app/ui/src/**/*.css | grep -i background` — use the real
  token value), white title, a muted line, the logo.
- The staged-files guard refuses a file over 1 MiB (`scripts/check-staged.ts`).

## Files

- `assets/social-preview.svg` (new) — 1280 × 640: background; `assets/logo.png` embedded by relative `href`
  (`logo.png`, same folder) at about 220 px on the left third; on the right "Fresher" (bold, about 96 px),
  "Your Mac is full. See why." (about 44 px), and "Free, open source, private. macOS 13+." (about 28 px, muted).
  Fonts: `font-family="-apple-system, 'Helvetica Neue', Helvetica, Arial, sans-serif"`. Keep at least 80 px of margin
  on every side (GitHub crops the edges in some views).
- `assets/social-preview.png` (new) — rendered from the folder `assets/`:
  `rsvg-convert --width 1280 --height 640 social-preview.svg -o social-preview.png`; under 1 MB (check with
  `ls -l`; if over, `--background-color` set and re-render, or reduce the logo's size).
- `docs/dependencies.md` — section "Tools on the machine", a row: `rsvg-convert` (librsvg) | 2.62.2 |
  LGPL-2.1-or-later | "renders `assets/social-preview.svg` to its PNG, by hand; not needed to build or test". A
  development tool may be under any open-source licence (`docs/restrictions/dependencies.md`). If plan 038 has added
  `ffmpeg` there, keep both rows.
- This step file's "Conclusions" — the exact texts and the command for the user (they move to the user-review list in
  04/02):
  - description: "A free, open-source macOS disk cleaner that explains every gigabyte and deletes nothing you have not
    confirmed."
  - topics: `macos`, `disk-cleaner`, `disk-space`, `rust`, `tauri`, `privacy`, `developer-tools`, `open-source`;
  - command: `gh repo edit AlexSergey/fresher --description "<description>" --add-topic macos --add-topic
    disk-cleaner --add-topic disk-space --add-topic rust --add-topic tauri --add-topic privacy --add-topic
    developer-tools --add-topic open-source`;
  - by hand: upload `assets/social-preview.png` in Settings → General → Social preview; enable Settings → Security →
    Private vulnerability reporting (needed by `SECURITY.md`'s link).

## Checks

- Look at `assets/social-preview.png` (open it, or read it as an image): text inside the margins, readable, the logo
  sharp, no clipping.
- `ls -l assets/social-preview.png` under 1 MB; `npm run check:repo` style guard passes at commit (the hook runs it).
- `npm run check:readme` still passes (the README does not reference the image).
- Commit: `docs(repo): social preview image and the repository's description`.

## Conclusions

The image `assets/social-preview.svg` (1280 x 640, background `#0e0f12` = `--gray-950`, muted text `#a3a9b4` =
`--gray-300`) is rendered to `assets/social-preview.png` (67 KB) with `rsvg-convert`. Text and logo sit inside an
80 px margin. Nothing was changed on GitHub. For the user to apply:

- description: "A free, open-source macOS disk cleaner that explains every gigabyte and deletes nothing you have not
  confirmed."
- topics: `macos`, `disk-cleaner`, `disk-space`, `rust`, `tauri`, `privacy`, `developer-tools`, `open-source`;
- command:

```sh
gh repo edit AlexSergey/fresher --description "A free, open-source macOS disk cleaner that explains every gigabyte and deletes nothing you have not confirmed." --add-topic macos --add-topic disk-cleaner --add-topic disk-space --add-topic rust --add-topic tauri --add-topic privacy --add-topic developer-tools --add-topic open-source
```

- by hand: upload `assets/social-preview.png` in Settings -> General -> Social preview; enable Settings -> Security
  -> Private vulnerability reporting (needed by `SECURITY.md`'s link).

**Applied on 2026-11-25.** At the user's request the description and 18 topics were set with `gh repo edit`
(description: "Free, open-source macOS disk cleaner made for developers' Macs. Explains every gigabyte in plain
English with a danger level, never sends your data anywhere, and removes nothing you have not confirmed."; topics:
`macos`, `macos-app`, `mac-cleaner`, `disk-cleaner`, `disk-cleanup`, `disk-space`, `free-up-space`, `storage`,
`cleaner`, `cleanup`, `cache-cleaner`, `developer-tools`, `xcode`, `uninstaller`, `privacy`, `rust`, `tauri`,
`open-source`), and the user uploaded the social preview image by hand. Private vulnerability reporting was turned on through
the API (`gh api -X PUT repos/AlexSergey/fresher/private-vulnerability-reporting`): the settings page answered 500.
