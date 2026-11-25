# 042/04 — Review

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core (texts), app/ui, e2e, sandbox (demo)

Review shows what goes and what the user has checked. A "Look before removing" item goes only when the user ticked it
by hand; 039's two parts and its Scan question are replaced. The design: [`../00-design.md`](../00-design.md), section
8; agent's calls 4 and 20. Mockup: [`review.dc.html`](../../../mockups/0.1.0/review.dc.html).

## Steps

| #   | Step                                          | Status      | Shared files |
| --- | --------------------------------------------- | ----------- | ------------ |
| 01  | [The tick model](./01-tick-model.md)          | 🔨 awaiting the user's walk | `state/{selection,selection-summary,reduce}.ts`, `product/danger-zone/zones.ts`, `screens/review/review-groups.ts`, `product/selection-bar/`, `screens/frame-sidebar.tsx`, the e2e flows |
| 02  | [The screen](./02-screen.md)                  | 🔨 awaiting the user's walk | `screens/review/`, `product/{finding-row,finding-list,danger-zone}/`, `crates/core/locales/en.json`, `crates/core/src/texts.rs`, `fixtures/demos/scenarios/cleanup.toml`, the e2e specs |

01, then 02.

## Conclusions

- The tick model (`Selection.toCheck`) and Review rebuilt: the summary card, a tile per category with its items and
  the `~` short path (`AppInfo.home`), the bar; 039's parts gone.
