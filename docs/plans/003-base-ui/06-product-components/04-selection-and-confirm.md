# 003/06/04 — Selection and confirmation

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** app/ui

## Files

- `product/selection-bar/` — `SelectionBar`: props `summary: { count, sizes, highest } | null`, `onReview`,
  `onClear`. Floats at the bottom centre of its screen (elevation 3, solid, `--radius-lg`): the count, the total size
  (`AnimatedNumber` with a 200 ms tween — this is not the confirmation), the highest level as a `DangerBadge`, **Clear**
  (ghost) and **Review** (primary). It rises `--distance-3` and fades in when the summary appears, leaves by fading
  (`usePresence`); `role="region"` with its label, announced once when it appears.
- `product/confirm-dialog/` — `ConfirmDialog` over `Dialog` (large):
  - props `content: ConfirmContent`, `onConfirm(method: 'trash' | 'permanent')`, `onCancel`. `ConfirmContent` is the
    UI's own type for now: `{ count, sizes: Sizes, highest: DangerLevel, groups: readonly { category, count, sizes,
    danger, items: readonly { id, title: Text, subject: string, sizes, danger }[] }[] }`; plan 008 fills it from the
    core's cleanup plan or replaces it with a generated type.
  - Order, top to bottom: the title naming the action and the count ("Move 128 items to the Trash?"); the highest
    level as a large `DangerBadge` with its sentence (`core.danger.<level>.meaning`); the total size (static
    `SizeText`, `--type-display`); the exact list grouped by category, each group with one plain sentence of what is
    lost (`<category>.if-removed`) and its items (a `VirtualList` inside the dialog when there are more than 50); then
    the `Checkbox` "Delete permanently instead — this cannot be undone", off.
  - Footer: **Cancel** (secondary) has the initial focus; the confirming button is primary "Move to Trash", and with the
    permanent choice it becomes the danger variant "Delete permanently". Enter does nothing unless a button has the
    focus.
  - Static by the safety rule: no counting, no stagger, no settle curve, no glow pulse; the dialog's one fade and
    scale is all that moves. The dialog's accessible description reads the count, the size and the highest level
    before the buttons.
- `locales/en.json` — `ui.selection-bar.*`, `ui.confirm.*` (the titles with plurals for both methods, the permanent
  choice, the buttons).

## Produces

`SelectionBar` for the module page and the review screen; `ConfirmDialog` complete in every state for plan 008.

## Checks

- Negative: `Cancel has the initial focus`, `Enter on the dialog does not confirm`, `the confirming button is not the
  danger variant until the permanent choice is ticked`, `the permanent choice is off when the dialog opens again`,
  `nothing in the dialog animates but its entrance` (no `AnimatedNumber`, no stagger class, no settle easing in its
  styles), `the selection bar is absent when nothing is selected`, `the highest level is shown with its icon and word`.
- Positive: `confirm calls onConfirm with trash`, `with the permanent choice it calls onConfirm with permanent`,
  `Escape and Cancel call onCancel and return the focus`, `the dialog's description reads count, size and level`,
  `every group shows what is lost`, `a long list is virtualised`, `the selection bar shows count, size and highest
  level`.
- Stories: selection bar 1 Safe item, many with High, partial total; confirm dialog Trash, permanent (red button),
  long list, highest High; screenshots looked at.

## Conclusions

Done on 2026-11-16. `SelectionBar` (`product/selection-bar/`) and `ConfirmDialog` (`product/confirm-dialog/`), each
with tests (negative first), stories and WebKit gallery screenshots, looked at. No new dependency, token or icon
(`Trash2` was there); 23 `ui.selection-bar.*` and `ui.confirm.*` keys.

Decided while building (agent's calls, to check):

- **`Button` takes a `ref`** (React 19's ref as a prop, as `Surface` does), so the dialog's `initialFocus` is Cancel.
- **`ConfirmDialog` takes `open`** besides `content`, `onConfirm` and `onCancel`: the owner keeps it mounted, so the
  dialog's fade out plays and the focus goes back to the opener. The permanent choice is reset to off whenever `open`
  turns true. The title, the confirming button's label and variant follow the choice ("Delete 3 items permanently?",
  danger "Delete permanently"); Escape, the close button and Cancel call `onCancel`.
- **The accessible description** is the dialog's visible subtitle, "3 items, 2.6 GB in all, highest danger level
  Low." (`ui.confirm.description`, singular "1 item, 1.5 GB, danger level Safe."): it is read before the buttons.
- **Static by the safety rule**: the sizes are `SizeText`, never counted; a group of more than 50 items is a
  `VirtualList` in a 320 px frame whose rows are kept from entering (`animation: none` on `[data-entering]` in the
  dialog's styles); the permanent choice's check mark is shown at once (no settle curve) by a `transition: none` in
  the dialog's styles. A test reads the component and its styles for `AnimatedNumber`, the stagger and the settle
  curve.
- **Groups**: the category's name (`<category>.name`), its highest level, its count and size, then its
  `<category>.if-removed` sentence and its items (title, subject in mono, level, size). Each list is named "The items
  of <category>".
- **`SelectionBar`**: absent for `null` or a count of 0; its total counts over `--duration-base` (200 ms) through
  `useAnimatedNumber` (`AnimatedNumber` offers only the slow and deliberate durations), the screen-reader copy is the
  final value; it is `position: absolute` at the bottom centre of its containing block (the screen sets `position:
  relative`), above the content at `--z-selection-bar`. It rises and fades in by a CSS keyframe and fades out through
  `usePresence`, showing its last summary while it leaves. It is announced once, when it appears, through
  `useAnnounce`: the screens render it inside the app's `Announcer`.
- `ConfirmContent` is exported from `confirm-dialog.tsx`; its groups' and items' types stay local until plan 008.
- Test helper: `src/test/confirm-content.ts` (the content built from findings, as the cleanup plan will be).
