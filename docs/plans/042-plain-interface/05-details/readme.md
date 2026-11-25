# 042/05 — Category details and menus

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

A category's details answer three questions side by side, and every row and tile menu offers the same items with the
accent hover. The design: [`../00-design.md`](../00-design.md), section 9; agent's calls 15 and 16. Mockups:
[`category-details.dc.html`](../../../mockups/0.1.0/category-details.dc.html),
[`row-menu.dc.html`](../../../mockups/0.1.0/row-menu.dc.html).

## Steps

| #   | Step                                            | Status      | Shared files |
| --- | ----------------------------------------------- | ----------- | ------------ |
| 01  | [The details screen](./01-details-screen.md)    | 🔨 awaiting the user's walk | `screens/category/`, `product/{finding-row,left-out-lines}/`, the locales, the e2e specs |
| 02  | [The menus](./02-menus.md)                      | 🔨 awaiting the user's walk | `design/primitives/menu/`, `product/{category-tile,finding-row}/`, `screens/category/category-findings.tsx`, the locales, the e2e specs |

01, then 02.

## Conclusions

- The details screen with its three answers and left-out bars, and one menu on every row and tile, without hints.
