# 042/03 — Scan

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e, sandbox (demo trace)

Scan rebuilt from the mockups: the disk card, the separator, the groups and their tiles; the scanning card; what has
no meaning while High is withheld. The design: [`../00-design.md`](../00-design.md), section 7. Mockups:
[`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html), [`scanning.dc.html`](../../../mockups/0.1.0/scanning.dc.html).

## Steps

| #   | Step                                                            | Status      | Shared files |
| --- | --------------------------------------------------------------- | ----------- | ------------ |
| 01  | [The screen, its groups and tiles](./01-screen-groups-tiles.md) | 🔨 awaiting the user's walk | `screens/scan/`, `product/{category-tile,category-grid,selection-bar,scan-hero}/`, the locales, the e2e specs |
| 02  | [The scanning card](./02-scanning-card.md)                      | 🔨 awaiting the user's walk | `screens/scan/`, `product/{scanning-card,scan-progress}/`, `state/`, the e2e specs |
| 03  | [What has no meaning without High](./03-without-high.md)        | 🔨 awaiting the user's walk | `screens/frame-sidebar.tsx`, `state/navigation.ts`, `e2e/core-mock.ts`, the e2e specs |

01, then 02, then 03.

## Conclusions

- Scan as the mockup: the disk card, the scanning card, the separator, the groups and tiles, the floating bar; Largest
  hidden while High is withheld, with the e2e `withheld` helper.
