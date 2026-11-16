# 005/03/02 — Folders and the picker

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** app, app/ui

## Files

- `app/tauri/Cargo.toml`, `app/tauri/src/{lib,commands}.rs` — `tauri-plugin-dialog` (pinned, MIT OR Apache-2.0),
  registered from Rust only; `pick_folder` answers the chosen folder or `null`. No web view permission is added.
- `docs/dependencies.md` — the plugin and what it brings.
- `app/ui/src/ipc/index.ts` — `pickFolder`.
- `app/ui/src/product/settings-row/folders-field.tsx` — the list with **Remove** per folder and **Add folder…**; the new
  list goes through `update_setting`; the core's refusal shows as the row's error.

## Checks

- Negative: a cancelled pick sends nothing; a folder already in the list is not sent twice; a refused folder shows the
  core's text; a read-only file disables both buttons.
- Positive: add sends the list with the new folder; remove sends it without the folder.
