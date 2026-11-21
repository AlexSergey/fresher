# 028/01 — The row's action

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** app/ui, e2e

The finding pages follow an ignore at once; a finding row can carry a settings menu within the list's keyboard
model; a category's details give each finding with a path "Ignore this item". The design:
[`../00-design.md`](../00-design.md), sections 1 to 5 (agent's calls 1 to 10).

## Steps

| #   | Step                                                          | Status      | Depends on | Shared files                                                                    |
| --- | ------------------------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------- |
| 01  | [The list follows an ignore](./01-list-follows-ignore.md)     | 🔨 in progress | —          | `app/ui/src/state/reduce.ts`, `app/ui/src/state/findings.ts`                    |
| 02  | [The row's menu](./02-row-menu.md)                            | 🔨 in progress | —          | `app/ui/src/product/finding-row/`, `app/ui/src/locales/en.json`, gallery baselines |
| 03  | [The details wire it](./03-details-wiring.md)                 | 🔨 in progress | 01, 02     | `app/ui/src/screens/category/`, `app/ui/e2e/`, `app/ui/src/locales/en.json`     |

01 and 02 share no file; 03 after both.

## Conclusions

Built in four commits (7bf3c77b, 64e19464, f929530d with its fix round 6e75ed1a); each step was reviewed and approved
with no critical or important findings. The steps ran in order, not as 01 and 02 in parallel, because they share the
baselines and `en.json`. What changed from the design and the calls: see
[`../readme.md`](../readme.md), "Conclusions". The user's walk is pending.
