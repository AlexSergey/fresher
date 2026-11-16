# 003/04 — Shell and window

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/tauri, app/ui

The frame everything sits in: a native macOS window with an overlay title bar, inset traffic lights and a translucent
sidebar; the layout components; the dialog and the sheet; the ambient light, the spotlight and the noise that give the
surfaces their depth. The design: [`../00-design.md`](../00-design.md), "The window", "Tauri changes", "Motion".

Depends on plan 01; steps 02–04 also on plan 02 (`Button`, `IconButton`, `Badge`, `Tooltip`).

## Steps

| #   | Step                                                     | Status   | Depends on | Shared files                                                                                       |
| --- | -------------------------------------------------------- | -------- | ---------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The native window](./01-native-window.md)               | ✔️ done    | 01         | `app/tauri/Cargo.toml`, `Cargo.lock`, `app/tauri/tests/` (the capability test), `app/ui/src/ipc/`, `app/ui/src/main.tsx` |
| 02  | [Layout](./02-layout.md)                                 | ✔️ done    | 01, 02     | `design/icons/icons.ts` (new names only)                                                           |
| 03  | [Dialog and sheet](./03-dialog-and-sheet.md)             | ✔️ done    | 01, 02     | —                                                                                                  |
| 04  | [Light and texture](./04-light-and-texture.md)           | ✔️ done    | 01         | `design/base/effects.css` (the noise's tuning)                                                     |

01 runs in wave 2; 02–04 in wave 3, in parallel (separate folders).

## Conclusions

Done on 2026-11-16 (01 in wave 2; 02–04 in one worktree in wave 3); each step's conclusions are in its file.

- Layout (`AppShell`, `Sidebar`, `Toolbar`, `PageHeader`, `ScrollArea`, `Surface`), `Dialog` and `Sheet` on one native
  `<dialog>`, `AmbientLight` and the spotlight; new e2e specs `layout.spec.ts` and `effects.spec.ts`.
- One new token pair (`--black-a55`, `--surface-backdrop`); both token blocks also apply to `::backdrop`, which
  inherits no custom properties before Safari 17.4. No new dependency.
- Agent's calls (accepted by the user on 2026-11-16): the noise stays a normal blend at 3 % (overlay and soft-light did not hide the banding);
  with motion off the spotlight is drawn at rest instead of not at all; a modal `<dialog>` does not loop the focus,
  but it never reaches the page behind.
- Left for later: `ConfirmDialog` (06/04) needs a `ref` on `Button` to focus Cancel; Playwright's WebKit draws no
  `backdrop-filter`, so the dialog's blur is checked by hand in 08.
