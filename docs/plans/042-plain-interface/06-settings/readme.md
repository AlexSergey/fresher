# 042/06 — Settings

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

Settings gets its own sidebar (a search, the core's sections, the module groups as a tree), one page per section, a
search that jumps to a setting, and sliders for whole numbers. The design: [`../00-design.md`](../00-design.md),
section 10; agent's call 17. Mockups: [`settings.dc.html`](../../../mockups/0.1.0/settings.dc.html),
[`settings-search.dc.html`](../../../mockups/0.1.0/settings-search.dc.html).

## Steps

| #   | Step                                              | Status      | Shared files |
| --- | ------------------------------------------------- | ----------- | ------------ |
| 01  | [The sidebar and the pages](./01-sidebar-pages.md) | 🔨 awaiting the user's walk | `screens/settings/`, `state/navigation.ts`, `screens/scan/scan-screen.tsx` (its link), the locales, the e2e specs |
| 02  | [The search](./02-search.md)                       | 🔨 awaiting the user's walk | `screens/settings/`, the locales, the e2e specs |
| 03  | [The sliders](./03-sliders.md)                     | 🔨 awaiting the user's walk | `product/settings-row/`, the e2e specs |

01, then 02, then 03.

## Conclusions

- A second sidebar with the module tree and pages, the search that jumps to a setting, sliders for numbers.
