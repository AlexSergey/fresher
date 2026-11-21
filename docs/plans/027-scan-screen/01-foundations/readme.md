# 027/01 — Foundations

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** core, app/tauri, app/ui, e2e

Three independent pieces the Scan screen builds on: the sandbox's Trash in the build and its line in the banner and
a cleanup's result, the menu primitive of the design layer, and the storage ring that draws a disk (its own colour
per part, the Free gap, hover). The design: [`../00-design.md`](../00-design.md), sections 2.1, 2.3, 4 and 7
(agent's calls 1, 2, 5, 6 and 8).

## Steps

| #   | Step                                                  | Status      | Shared files                                                                                       |
| --- | ----------------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------- |
| 01  | [The sandbox's Trash](./01-sandbox-trash.md)          | 🔨 in progress | `crates/core/locales/en.json`, `app/ui/src/ipc/bindings.generated.ts`, `app/ui/src/locales/en.json` |
| 02  | [The menu primitive](./02-menu.md)                    | 🔨 in progress | `app/ui/src/design/primitives/icon-button/icon-button.tsx`                                           |
| 03  | [The ring draws a disk](./03-ring.md)                 | 🔨 in progress | `app/ui/src/design/tokens/semantic.css`, `app/ui/src/product/storage-map/parts.ts`, `app/ui/src/screens/overview/overview-screen.tsx` |

The three share no source file and run in parallel (the gallery collects stories by glob); the second and third to
merge regenerate any gallery baseline both touched.

## Conclusions

Built in three steps (01/01 to 01/03, each reviewed and fixed), awaiting the user's walk.

- **01/01** the sandbox's Trash is in the build (`Build::Sandbox { trash }`), the dev banner names it, and a cleanup
  to it says the same. Deviation: the banner's one-line ellipsis cuts a long "Simulated: ..." line.
- **01/02** the menu primitive (`design/primitives/menu/`): arrow keys, Home and End, Escape and Tab close it, focus
  returns to the trigger; a fix round added a forced-colours ring and a re-measure on item change.
- **01/03** the ring draws the disk: Free is the gap, one colour per part, a hover highlight shared with the legend.
  The explained part is split by danger level in the UI from the core's shares (design call 3).
