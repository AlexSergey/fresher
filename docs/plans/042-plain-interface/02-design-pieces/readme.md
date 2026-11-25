# 042/02 — Design pieces

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** app/ui, e2e

The pieces the new screens are built from, each with its story, unit tests and gallery baseline, before any screen uses
them. The design: [`../00-design.md`](../00-design.md), section 6. Mockups: every file of
[`docs/mockups/0.1.0/`](../../../mockups/README.md).

## Steps

| #   | Step                                                                  | Status      | Shared files |
| --- | --------------------------------------------------------------------- | ----------- | ------------ |
| 01  | [Tokens, the button's sheen and the menu's hover](./01-tokens.md)     | 🔨 awaiting the user's walk | `design/tokens/`, `design/primitives/{button,menu}/`, every baseline |
| 02  | [Separator, tile and icon badge](./02-separator-tile-icon.md)         | 🔨 awaiting the user's walk | `design/layout/`, `design/icons/icons.ts`, `product/category-icon/`, the gallery |
| 03  | [The disk card](./03-disk-card.md)                                    | 🔨 awaiting the user's walk | `product/disk-card/`, `design/data/storage-ring/`, the locales |
| 04  | [The swoosh progress](./04-swoosh.md)                                 | 🔨 awaiting the user's walk | `design/data/swoosh-progress/` |
| 05  | [The slider](./05-slider.md)                                          | 🔨 awaiting the user's walk | `design/primitives/slider/` |

01 first (its tokens are used by the rest); then 02 to 05 one at a time (they share the gallery's registry and its
baselines).

## Conclusions

- Tokens for every mockup colour, the sheen and the menu hover, the separator, the tile, the icon badge, the disk card,
  the swoosh and the slider, each with stories and gallery baselines.
