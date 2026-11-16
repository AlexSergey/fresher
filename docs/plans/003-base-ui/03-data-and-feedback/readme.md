# 003/03 — Data and feedback

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

The components that show numbers and lists, and the ones that tell the user what happened: honest sizes, counting
numbers, the danger badge, the storage ring with its scan sweep, the usage bar, rows and the virtual list, notices,
empty and error states, toasts and the announcer. Generic: they take numbers, texts and render props, never the
core's types. The design: [`../00-design.md`](../00-design.md), "Components", "Motion", "Performance budget".

Depends on plan 01 and on the primitives each step uses (`Tooltip`, `VisuallyHidden`, `Skeleton` of 02/04 for all;
`Checkbox` of 02/02 for 03; `Button` of 02/01 for 04). The four steps write separate folders and run in parallel once
those exist.

## Steps

| #   | Step                                                          | Status   | Depends on       | Shared files                             |
| --- | ------------------------------------------------------------- | -------- | ---------------- | ---------------------------------------- |
| 01  | [Values](./01-values.md)                                      | ✔️ done    | 01, 02/04        | `design/icons/icons.ts` (new names only) |
| 02  | [Charts and the scan sweep](./02-charts-and-sweep.md)         | ✔️ done    | 01, 02/04, 03/01 | —                                        |
| 03  | [Rows and the virtual list](./03-rows-and-virtual-list.md)    | ✔️ done    | 01, 02/02, 02/04 | `design/icons/icons.ts` (new names only) |
| 04  | [Feedback](./04-feedback.md)                                  | ✔️ done    | 01, 02/01, 02/04 | `design/icons/icons.ts` (new names only) |

02 waits for 01 only for `SizeText` and `AnimatedNumber`, which the stat tile shows; it may start with the ring and
the sweep.

## Conclusions

Done on 2026-11-16, in two parallel worktrees (01–02 and 03–04); each step's conclusions are in its file.

- Values, charts, the sweep, rows, the virtual list and the feedback components are in `design/data/`,
  `design/effects/scan-sweep/` and `design/feedback/`, each with tests (negative first), stories and WebKit screenshots.
- No new dependency and no new token; one new icon (`CircleCheck`).
- Agent's calls (accepted by the user on 2026-11-16): a row keeps `role="listitem"` without `aria-selected`/`aria-expanded` (ARIA does not allow
  them there): a selectable row's state is read from its checkbox, and `aria-expanded` sits on the tree row's real
  disclosure button. `Segment` carries a formatted `size` for the legend. `Notice` takes an explicit `live` prop.
- Left for 06/03: whether a row's checkbox keeps its own tab stop inside the virtual list's roving focus.
