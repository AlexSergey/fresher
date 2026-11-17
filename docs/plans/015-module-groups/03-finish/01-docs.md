# 015/03/01 — The docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## Files

- `docs/architecture/modules.md` — "The crate" and "The `Module` trait in short": the group is a required part of the
  descriptor; the checklist gains "the module names its group"; a `Debug` module never ships.
- `docs/architecture/README.md` — the ignore lists (four kinds, the group → modules rule), `ModuleInfo.group`, the
  Settings tabs, the `Tabs` component in the design system.
- `docs/testing.md` — `crates/e2e/tests/ignore.rs`'s new flows; the e2e mock's ignore kinds.
- `docs/ideas/ignore-list.md` — "Taken by plan 015" for the module and the group; the item's button stays deferred.
- `docs/ideas/developer-environment-cleaner.md` — its tools sit in the group Development.
- `docs/deferred/ignore-an-item-button.md` — checked: still due with the row's actions; unchanged.
- `docs/plans/README.md` — the row's status.

## Produces

The docs as built.

## Checks

- Every link resolves; `npm run lint` passes; no product or project is named (clean room).

## Conclusions

Done: `docs/architecture/modules.md` (the group, the checklist box), `docs/architecture/README.md` (the four ignore
lists, `ModuleInfo.group`, `hiddenModules`, modules by group, the three tabs, `Tabs`), `docs/testing.md` (the new
core flows, the e2e mock's ignore part), the two ideas, the plans table. `docs/deferred/ignore-an-item-button.md`
checked: still due with the row's actions, unchanged.
