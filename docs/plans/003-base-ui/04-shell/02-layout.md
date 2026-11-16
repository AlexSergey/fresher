# 003/04/02 — Layout

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-15 · **Modules:** app/ui

## Files

- `design/layout/app-shell/` — `AppShell`: a grid of the sidebar (232 px) and the main pane; the main pane is the
  opaque window surface with a 1 px `--border-subtle` on its leading edge and the ambient light slot behind its content;
  `[data-window='vibrant']` leaves the sidebar transparent, otherwise it paints `--surface-sidebar`'s gradient.
  Under 1,000 px wide the gutters shrink to 24 px; the sidebar keeps its width (the window's minimum is 900 px).
- `design/layout/sidebar/` — `Sidebar`: the top 52 px is a drag region (`data-tauri-drag-region`) that leaves room for
  the traffic lights; `children` are `NavSection`s (a `--type-label` heading and items); a footer slot (Settings).
- `design/layout/nav-item/` — `NavItem`: icon, label, optional trailing (`Badge` count or a size), `active`; a link-like
  `<button>` with `aria-current="page"` when active. One indicator element per `Sidebar` (a soft `--surface-pressed`
  pill with a 2 px Iris edge) slides to the active item by `transform` (`--duration-move`, `--ease-standard`), measured
  from the item's offset; items stagger in on the first render.
- `design/layout/toolbar/` — `Toolbar`: 52 px, a drag region except over its controls, the page title slot and an
  actions slot; a sentinel under it (`IntersectionObserver`) shows a `--border-strong` hairline and a soft shade (both
  by opacity) once content scrolls under it.
- `design/layout/page-header/` — `PageHeader`: `title` (`--type-title-2`), optional `subtitle` and `actions`.
- `design/layout/scroll-area/` — `ScrollArea`: the scrolling region of a screen, `tabIndex={0}` only when it overflows
  and has no focusable child, thin overlay scrollbars styled for dark (`::-webkit-scrollbar`), scroll restored per
  screen key.
- `design/layout/surface/` — `Surface`: `elevation: 1 | 2 | 3`, `radius`, `padding`; the gradient fill, hairline and
  top highlight of the design's elevation table.

## Produces

The frame of every screen (07/01) and the surfaces of every card and panel.

## Checks

- Negative: `the toolbar's controls are not part of the drag region`, `the nav indicator does not animate under motion
  off`, `a scroll area that does not overflow is not focusable`, `an inactive nav item has no aria-current`.
- Positive: `the active nav item has aria-current page`, `the indicator moves to the active item`, `the toolbar shows
  its hairline when content scrolls under it`, `the sidebar's top strip is a drag region`, `elevation 3 has the overlay
  surface`.
- Stories: app shell narrow and wide with the fallback sidebar; sidebar with sections and a count; nav item rest,
  hover, active, focus; toolbar at rest and scrolled; page header variants; scroll area short and overflowing; surface
  1, 2, 3; screenshots looked at.

## Conclusions

- Built as planned: `AppShell`, `Sidebar` with `NavSection` (in `sidebar/`), `NavItem`, `Toolbar`, `PageHeader`,
  `ScrollArea`, `Surface`, each with tests (negative first), stories and WebKit screenshots looked at.
- **The content column** is one custom property: the main pane sets `--content-inset` (`max(gutter, (100% − 1,120 px)
  / 2)`), and the toolbar, the page header and later screens use it as their side padding. The 24 px gutter under
  1,000 px is a container query on the shell (`@container shell`), so the narrow story shows it in a wide viewport.
- **Over the material** the sidebar is not fully transparent: it takes `--surface-sidebar-over-material` (gray 950 at
  45 %), the value of the design's palette table for "over the material".
- **The toolbar is the first child of its `ScrollArea`**, sticky; a 1 px sentinel just above it leaves the view once
  the content scrolls. At rest the bar is transparent (the light behind the screen shows); scrolled, a window-coloured
  fill with the `--border-strong` hairline and a short shade fading from the window colour fade in by opacity.
- **Drag region:** the toolbar is `data-tauri-drag-region="deep"` (the pinned Tauri's drag script supports it and
  skips buttons, links and focusable elements by itself) and its actions slot is `data-tauri-drag-region="false"`; the
  sidebar's strip is a plain drag region (it holds nothing).
- **The nav indicator** is placed from the active item's `offsetTop` after every render of the sidebar, written into
  `--indicator-y`; its first placement jumps (`data-instant`), later ones slide over `--duration-move`. The sidebar does
  not scroll, so an offset is a place; the stagger index `--i` of each item is set by the sidebar once.
- `ScrollArea` is a named region (`role="region"`, `tabIndex={0}`) only while it is focusable; the scroll per screen key
  is kept in a module map, written on scroll and restored when the key changes.
- `Surface` takes `className` and `ref`, so a card can spread the spotlight's props on it (04/04).
- The gallery test that renders every story queries the first `main`: the app shell's stories hold their own.
