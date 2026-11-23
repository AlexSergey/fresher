# 039/04 — The wording

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** scripts, docs, core, modules, app/ui

Part B, built last before the security audit, after every other plan of the session is on `main`. A writing guide and
a check that keeps the texts to it, then a pass over every dictionary as it stands, then the hints every control lacks.
The steps are procedures over whatever dictionaries, modules and screens exist when they run: nothing here is a fixed
list. The design: [`../00-design.md`](../00-design.md), section 4; agent's call 5.

Before 04/01 starts: the branch is rebased onto `main` (plans 031 to 038 merged), `Cargo.lock`, `package-lock.json`
and `bindings.generated.ts` regenerated, and the whole suite run: `npm run pre-push` and `npm run test:e2e`. A
failure that comes from part A meeting a later plan (a demo scenario or a cleanup flow that meets the agreement sheet
or the High tick) is fixed first, in its own commit, by accepting or ticking as a user would — never by weakening a
guard. On 2026-11-23 plans 031 to 038 are already on `main` (740cf3c3) and part A's steps name what they meet, so this
run is expected to find nothing; a rebase is needed only if `main` moved.

Since plan 038, the demo scenarios (`fixtures/demos/scenarios/*.toml`) press controls by their text, each traced to a
text key by `crates/sandbox/src/scenario/traced.rs`: a step of part B that changes the text of a target updates the
scenario and the trace tests in the same commit.

## Steps

| #   | Step                                                              | Status      | Shared files                                                                    |
| --- | ----------------------------------------------------------------- | ----------- | ------------------------------------------------------------------------------- |
| 01  | [The guide and the check](./01-guide-and-check.md)                | 🔨 awaiting the user's walk | `docs/writing.md`, `scripts/`, `package.json`, `docs/quality.md`, `docs/README.md`, `docs/rules.md` |
| 02  | [The core's and the modules' texts](./02-core-and-module-texts.md) | 🔨 awaiting the user's walk | `crates/core/locales/en.json`, `modules/*/locales/en.json`, `docs/categories/`, `fixtures/demos/scenarios/`, the baselines |
| 03  | [The interface's texts and the hints](./03-interface-texts-and-hints.md) | 🔨 awaiting the user's walk | `app/ui/src/locales/en.json`, the components that gain a hint, `fixtures/demos/scenarios/`, `package.json`, `docs/quality.md`, the baselines |

01, then 02, then 03.

## Conclusions

The writing guide and the check are built (01); the pass changed nine texts in the core and the modules (02) and none
in the interface (03), and `check:texts` stands in pre-push. The check found 22 violations at first and reports 0 now.
Six kinds of hint were added. Texts whose meaning looked wrong are listed in the steps and in the user's review, not
rewritten.
