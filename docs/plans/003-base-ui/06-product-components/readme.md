# 003/06 — Product components

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

The app's own components, built from the design system: they take the core's types (`import type` from the generated
bindings) and render their texts through `i18n/`, but never call the core or read the store — the screens pass them
data and callbacks. This is where the signature moments live: the scan hero, the gliding category cards, the
virtualised finding list, the static and exact confirmation, the cleaned moment. The design:
[`../00-design.md`](../00-design.md), "Key product components", "Choreography rules", "Accessibility".

Depends on plans 02, 03, 04 and 05 (the types and the state shapes the props mirror). Every component lives in
`app/ui/src/product/<name>/` with its `.tsx`, `.module.css`, `.test.tsx` and `.stories.tsx`; stories use the demo
scan's fixtures and `src/test/large-scan.ts`. The steps write separate folders and run in parallel; each adds its texts
to `app/ui/src/locales/en.json` under `ui.<component>.*`.

## Steps

| #   | Step                                                                  | Status   | Shared files                                   |
| --- | --------------------------------------------------------------------- | -------- | ---------------------------------------------- |
| 01  | [The scan hero](./01-scan-hero.md)                                    | ✔️ done    | `locales/en.json`, `design/icons/icons.ts`     |
| 02  | [Category cards](./02-category-cards.md)                              | ✔️ done    | `locales/en.json`                              |
| 03  | [The finding list](./03-finding-list.md)                              | ✔️ done    | `locales/en.json`                              |
| 04  | [Selection and confirmation](./04-selection-and-confirm.md)           | ✔️ done    | `locales/en.json`, `design/icons/icons.ts`     |
| 05  | [Result, permission and settings rows](./05-result-permission-settings.md) | ✔️ done    | `locales/en.json`, `design/icons/icons.ts` |

## Conclusions

Done on 2026-11-16 in three parallel worktrees (01–02, 03–04, 05); each step's conclusions are in its file.

- The scan hero and progress, category cards, the grid and the module section, the finding row and list, the
  selection bar, the confirmation dialog, the result summary, the permission card and the settings row are in
  `app/ui/src/product/`, each with tests (negative first), stories and WebKit screenshots. No new dependency or token;
  one new icon (`Lock`).
- Primitives changed on the way: `Button` takes a `ref` (the dialog focuses Cancel first), `Checkbox` takes
  `tabIndex={-1}` (a finding row is the list's one tab stop), `ListRow` passes `aria-label` and `aria-describedby`.
- One size helper for all of them: `honestSize` in `product/sizes.ts` (a lower bound reads "at least …" with the core's
  reason, an unknown size is a dash, never 0); the finding list's own copy was merged into it.
- For plan 07: the screens provide the `Announcer` (the hero and the selection bar speak through it), give the selection
  bar a positioned parent, and pass `total: undefined` to `FindingList` until its first page has answered. Stories set
  the core dictionary in their render, so the order story files load in does not matter.
- Agent's calls (accepted by the user on 2026-11-16): the hero's number is the hero type with a smaller unit (the full string did not fit the
  ring); a failed scan shows its error beside the ring, not inside it; the reason tooltip of a finding sits on its
  badge, not on the row; nothing waits for the freed size to finish counting before the result's buttons appear.
- To watch in 07 and 08: while pages refresh at different times one finding could briefly sit at two indices.
- Parallel gallery runs on fixed ports photographed each other's servers; each agent re-ran on its own port and the
  final run on the plan branch checks every screenshot again.
