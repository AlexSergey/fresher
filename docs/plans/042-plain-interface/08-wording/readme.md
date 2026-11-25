# 042/08 — The wording

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core, modules (locales), sandbox, app/ui, scripts, docs

The last implementation plan (agent's call 9): every text a person reads, rewritten shorter to the writing guide, on
the new screens; then the check holds the new rules. The steps are procedures over whatever texts exist then. The
design: [`../00-design.md`](../00-design.md), section 12. The feedback's items 5 to 8 are the examples to fix.

## Steps

| #   | Step                                                     | Status      | Shared files |
| --- | -------------------------------------------------------- | ----------- | ------------ |
| 01  | [The core and module texts](./01-core-module-texts.md)   | 🔨 awaiting the user's walk | every Rust `locales/en.json`, `docs/writing.md`, `docs/categories/`, the trial module and its case file, the UI's fixtures |
| 02  | [The interface texts](./02-interface-texts.md)           | 🔨 awaiting the user's walk | `app/ui/src/locales/en.json`, the UI's tests, the baselines |
| 03  | [The guide and the check](./03-guide-and-check.md)       | 🔨 awaiting the user's walk | `docs/writing.md`, `scripts/check-texts.ts`, `scripts/check-texts.test.ts` |

01, then 02, then 03 (the check comes last, when every text already passes it).

Every step: never change a key, a placeholder or a danger level; a text whose meaning looks wrong is listed in the
report, not rewritten; each text changed is listed (key, before, after) in the report.

## Conclusions

- 315 texts changed across the dictionaries (core 42, interface 44, modules 229, the trial and demo included); the
  check gained the line's and the name's rules; 15 violations under today's rules before the pass, 0 after.
