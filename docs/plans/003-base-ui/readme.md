# 003 — Base UI

**Status:** 📝 draft · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui, app/tauri

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Create a draft of plan 003 for the base UI. If we have HTML/CSS/JS in Tauri 2, think about
a design system for tests and extension, lay the ground for animations, list the components we will build, pick the base
style."

## Goals

1. A design system that is easy to test and to extend: tokens, primitives, components, each shown in every state.
2. Animation built in from the start, and switchable off.
3. The list of components, built once and reused by every module.
4. The base style: Dark Premium ([`goal.md`](../../goal.md), section 6) made concrete.
5. The base screens, working on the `demo` module of plan [002](../002-base-architecture/readme.md).

## Non-goals

- Screens of real modules, the treemap and the history charts: they come with their plans.
- A light theme (the tokens allow one later).

## Design

### The platform

The UI runs in the system webview, whose version is tied to macOS. With the minimum of macOS 13 the baseline is
Safari 16: container queries, `@layer`, `color-mix()`, `<dialog>` work; the View Transitions and Popover APIs do not.
The build targets that baseline and nested CSS is lowered by the bundler. The exact feature list is checked in the first
step.

### The design system

- **Tokens in three tiers**, as CSS custom properties: primitive (`--gray-900`) → semantic (`--surface-raised`,
  `--text-secondary`, `--danger-high`) → component. Components use semantic tokens only, so a theme is one file.
- **Plain CSS with CSS Modules** and `@layer reset, tokens, base, components`: built into Vite, no runtime, no extra
  dependency.
- **Native elements first**: `<dialog>` for modals (focus trap and Escape for free), `<details>`, real checkboxes and
  buttons. No headless component library until a component proves it needs one.
- **System fonts**: SF Pro and SF Mono through `system-ui` / `ui-monospace` — native, and nothing to download; tabular
  figures for every size.
- **Icons**: `lucide-react` (ISC), tree-shaken.
- **State**: a small store of our own on `useSyncExternalStore`, fed by the core's events. No router: the current
  screen is a discriminated union.
- **Boundaries by lint**: `design/` imports nothing from `ipc/` or `state/`; only `ipc/` imports `@tauri-apps/api`.

```
app/ui/src/
├── design/    tokens/, primitives/, components/, motion/, icons/
├── ipc/  i18n/  state/                        (from plan 002)
├── screens/   overview, module, review, settings
├── modules/   optional custom views per module; the generic module page is the default
└── gallery/   dev-only page with every component in every state
```

### Testing

- Every component has unit tests (Vitest + Testing Library), negative cases first.
- **A gallery of our own** instead of a component workshop tool: a dev-only page that renders every component in every
  state. Playwright takes a screenshot per component and state — this is the visual test — and checks keyboard use.
- Motion is off in tests (`data-motion="off"` zeroes every duration), so screenshots are deterministic.
- The demo module gives the screens stable made-up data.

### Animation

- **Motion tokens**: durations 120 / 200 / 320 / 480 ms and three easings (standard, enter, exit).
- **CSS first**: components change `data-state`; transitions and keyframes do the rest.
- Three small hooks of our own, no animation library: `usePresence` (exit animations before unmount),
  `useAnimatedNumber` (sizes counting up during a scan), `useFlipList` (rows gliding as the list re-sorts live).
- `prefers-reduced-motion` is honoured globally.
- Character: calm and quick; nothing bounces on the confirmation and deletion screens.

### The base style

| Token             | Value                               | Use                                       |
| ----------------- | ----------------------------------- | ----------------------------------------- |
| surface base      | `#0F1012`                           | the window                                |
| surface raised    | `#16181B`                           | cards, the sidebar                        |
| surface overlay   | `#1D2024`                           | dialogs, menus                            |
| border            | white at 6 % (subtle), 12 % (strong) | hairlines, 1 px                          |
| text              | `#ECEEF1` / `#A4A9B3` / `#6C727D`   | primary / secondary / tertiary            |
| accent            | `#7C8CFF`                           | the one accent: focus, primary action     |
| danger levels     | `#3DD68C` / `#5BB8FF` / `#F5B544` / `#FF6369` | Safe / Low / Medium / High      |

- Depth comes from lighter surfaces and hairlines, not from shadows; only overlays cast one.
- A 4 px spacing grid; radii 6 / 10 / 14; type scale 11 / 12 / 13 / 15 / 17 / 22 / 28 / 40, with one large "hero" number
  per screen.
- A danger level is never colour alone: always an icon and a word.
- The accent is cool on purpose: green is reserved for Safe.
- The window: a hidden title bar with inset traffic lights, a draggable top strip, a sidebar.
- Contrast is checked against WCAG AA in the foundations step; the values above may shift by it.

### Components

| Group        | Components                                                                                              |
| ------------ | ------------------------------------------------------------------------------------------------------- |
| Primitives   | Button, IconButton, Checkbox (with the mixed state), Switch, SegmentedControl, TextField, SearchField, Select, Tooltip, Badge, Spinner, ProgressBar, Skeleton, Divider |
| Data         | SizeText (complete, partial `42 GB+`, unknown), AnimatedNumber, DangerBadge, PathText (middle truncation), UsageBar (stacked), Ring, StatTile, ListRow, TreeRow, VirtualList |
| Feedback     | Notice (inline banner), EmptyState, ErrorState, Toast                                                   |
| Layout       | AppShell, Sidebar, NavItem, PageHeader, Toolbar, ScrollArea, Dialog, Sheet                              |
| Product      | CategoryCard, ModuleSection, FindingList, SelectionBar (count, total size, highest danger), ConfirmDialog (the exact list, Trash or permanent), ScanProgress, ResultSummary, PermissionCard (Full Disk Access), SettingsRow (label, plain description, control, reset) |

### Screens

Overview ("where did my space go"), the generic module page (renders any module from its descriptor, so a new module
needs no UI code), review and confirm, settings, and the missing-permission state.

## Plans (after approval)

| #   | Plan                | What                                                                     |
| --- | ------------------- | ------------------------------------------------------------------------ |
| 01  | Foundations         | tokens, layers, reset, typography, motion, icons, the gallery, visual tests |
| 02  | Primitives          | the primitives group                                                     |
| 03  | Data and feedback   | the data and feedback groups                                             |
| 04  | Shell               | the window, AppShell, navigation                                         |
| 05  | Product components  | the product group                                                        |
| 06  | Base screens        | the screens on the demo module                                           |

Depends on plan 002 (IPC, dictionary, demo module) and on plan 001/04 (Vitest and Playwright), which is not merged yet.

## Ideas this touches

[Cleanup preview and safe removal](../../ideas/cleanup-preview-and-safe-removal.md) (ConfirmDialog, ResultSummary),
[user settings](../../ideas/user-settings.md) (SettingsRow), the [ignore list](../../ideas/ignore-list.md), honest
sizes and the demo mode with automatic screenshots from [best practices](../../ideas/best-practices.md).

## Decisions

None yet.

## Agent's calls (to check)

1. **Plain CSS with CSS Modules and tokens**; no utility framework, no CSS-in-JS.
2. **Native elements, no headless library**, until a component needs one.
3. **Our own gallery** plus Playwright screenshots instead of a component workshop tool.
4. **CSS-first animation with three hooks of our own**; no animation library.
5. **System fonts; `lucide-react` for icons** (a new dependency, ISC).
6. **Dark only**, accent `#7C8CFF`, the palette above.
7. **Our own store and no router.**
8. **macOS 13 stays the minimum**, so the baseline is Safari 16.
9. **A generic module page** by default, custom views optional.
10. **Target version 0.2.0.**

## Conclusions

Not written yet.
