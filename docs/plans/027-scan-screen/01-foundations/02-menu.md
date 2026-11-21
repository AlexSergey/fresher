# 027/01/02 — The menu primitive

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

The design layer's first menu: a button that opens a short list of actions, for the card's settings button (feedback
item 10). Nothing in the app uses it yet. The design: [`../00-design.md`](../00-design.md), section 4 (agent's
call 5).

## Files

- `app/ui/src/design/primitives/menu/menu.tsx` (new) — `Menu`, `MenuItem`, `MenuProps` exactly as in the design,
  section 4. State: open or closed, the active item's index. The popup rendered with `createPortal` into
  `document.body`, as the tooltip is, with `position: fixed` at `placeMenu`'s place, measured in a layout effect after
  it mounts and again on window resize and scroll while open. `useId` ties the trigger's `aria-controls` to the
  menu's `id`; each item's description is its `aria-describedby`.
- `app/ui/src/design/primitives/menu/place.ts` (new) —
  `export const placeMenu = ({ anchor, menu, viewport }): { readonly left: number; readonly side: 'above' | 'below'; readonly top: number }`:
  below the anchor with a 4 px gap, its right edge on the anchor's right edge; above when it does not fit below and
  does fit above (below wins when neither fits); `left` kept within 8 px of both window edges.
- `app/ui/src/design/primitives/menu/menu.module.css` (new) — a raised surface (`--elevation-2` tokens, `--radius-lg`),
  items with the list row's paddings, the active item's background `--surface-hover`,
  the focus ring on the active item, the description in `--text-secondary` footnote type; `z-index: var(--z-tooltip)`;
  a fade over `--duration-fast`, none under reduced motion (both the media query and `[data-motion='reduced']`, as
  the ring's CSS does).
- `app/ui/src/design/primitives/menu/menu.stories.tsx` (new) — `closed`, `open` (forced open, two items with
  descriptions), `open-near-the-bottom` (placed above). Opening for a story uses a prop the stories alone pass
  (`defaultOpen?: boolean`, documented "for the gallery"), as other primitives force their states
  (`design/primitives/forced.ts` if it fits).
- `app/ui/src/design/primitives/menu/menu.test.tsx` and `place.test.ts` (new).
- `app/ui/src/design/primitives/icon-button/icon-button.tsx` — optional `ref`, `aria-controls`, `aria-expanded`,
  `aria-haspopup` passed to the `button`; nothing else changes (its stories and baselines stay as they are).
- `app/ui/e2e/screenshots/gallery.spec.ts/Menu/` — the new baselines.

## Produces

`Menu` and `MenuItem` (`design/primitives/menu/menu.tsx`) for 02/02.

## Checks

- `menu.test.tsx`, negatives first:
  - `describe('negative cases')`: `it('is closed until the trigger is used')`; `it('closes on Escape and gives the
    focus back to the trigger')`; `it('closes on Tab without keeping the focus')`; `it('closes on a pointer down
    outside without moving the focus')`; `it('does not call an item when the menu closes without a choice')`.
  - `describe('positive cases')`: `it('opens on Enter, Space and ArrowDown on the first item, on ArrowUp on the
    last')`; `it('moves with the arrows, wraps, and jumps with Home and End')`; `it('selecting an item closes the
    menu, focuses the trigger, then calls the item')`; `it('names each item by its label and describes it by its
    description')`; `it('ties the trigger to the menu with aria-controls and aria-expanded')`.
- `place.test.ts`: negatives first (`it('never leaves the window on either side')`, `it('stays below when neither
  side fits')`), then `it('opens below, aligned to the anchor's right edge')`, `it('opens above when only above
  fits')`.
- `npm run test --workspace app/ui -- src/design/primitives/menu`
- `npm run lint:ts`; `npm run knip`; `npx eslint` and `npx prettier --check` on the changed files.
- `npm run test:e2e:update --workspace app/ui -- gallery.spec.ts` (only the new `Menu/*` baselines appear; look at
  them), then `npm run test:e2e --workspace app/ui -- gallery.spec.ts keyboard.spec.ts axe.spec.ts` (the keyboard
  spec walks every story's focus stops; axe checks the open story).

## Conclusions

Not written yet.
