# 043 — Build flags, and High and Largest back on

**Status:** 🔨 in progress · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** core, app/tauri, app/ui, sandbox,
scripts, e2e, ci, docs

Written as a one-file draft on 2026-11-26 and approved by the user the same day, with every recommended decision and
agent's call taken; turned into plans and steps from a read of the code on `feat/043-build-flags` (d50221fc). The
design, with its agent's calls: [`00-design.md`](./00-design.md). Asked after the first tester's feedback
([`../../deferred/tester-feedback-0.1.0.md`](../../deferred/tester-feedback-0.1.0.md)). Goes before
[044](../044-largest-folders/readme.md), which needs Largest shown. Branch `feat/043-build-flags`, squash-merged into
`main` once, when plan 03 is done.

## Intent

The user's task (translated, 2026-11-26): "A more advanced system of feature flags, set before the build: look at how
it is today and improve it, as its own plan. Turn Large and the High level back on."

## How it is today

- **High** is one hard-coded value: `compose.rs` builds every engine with `HighLevel::Withheld` (plan 042), the same in
  release and development builds; tests and the sandbox name it explicitly. The engine refuses every High finding and
  every way to remove one.
- **Largest** has no switch of its own: the sidebar shows it only while `get_app_info` reports `high: offered`
  (`frame-sidebar.tsx`). Its files and folders carry High ticks, so the two were tied on purpose.
- **Modules** are chosen by `ModuleSet` (`Every`, `Shipped` without the Debug group, `Only(id)` for a case file's
  autotest); a debug build adds the Debug group by `cfg(debug_assertions)`.
- Nothing lists what a build ships with; turning one thing on means editing code, and the release notes cannot say
  which switches were on.

## Goals

1. `flags.toml` at the repository root says what a build ships with: per flag, its value for a release and for a
   development build, and what it does.
2. Each flag is a typed value in the core (`BuildFlags`, enums), set by the app's build script, given to the engine and
   reported by `get_app_info`; the UI shows a screen only from it.
3. Only the build's command line overrides a flag (`FRESHER_FLAGS=largest=hidden`); nothing the user can write does.
4. The first flags, apart: `high` (`offered` / `withheld`) and `largest` (`shown` / `hidden`). 0.2.0 ships both on.
5. `npm run check:flags` in the pre-push checks and CI; the release's text lists the flags it was built with.
6. The e2e mock and the sandbox take the flags like the app.

## Plans

| #   | Plan                                                      | Status      | Depends on | What                                                                                     |
| --- | --------------------------------------------------------- | ----------- | ---------- | ---------------------------------------------------------------------------------------- |
| 01  | [Flags in the core and the shell](./01-flags/readme.md)   | ✅ approved | —          | `BuildFlags`, `flags.toml` and the build script, the composition, `get_app_info`         |
| 02  | [The UI, the mock and the guard](./02-ui-and-guard/readme.md) | ✅ approved | 01     | Largest from its own flag, the e2e mock's flags, `check:flags` and the release text      |
| 03  | [High and Largest on for 0.2.0](./03-turn-on/readme.md)   | ✅ approved | 02         | the values, the sandbox's line, the rule's doc, the test cases, the user's walk          |

### Order

```
01/01 ─ 01/02 ─ 01/03 ─ 02/01 ─ 02/02 ─ 02/03 ─ 03/01 ─ 03/02 ─ 03/03
```

Every step is one commit and leaves the workspace green (`cargo nextest`, clippy, fmt, `npm run lint:ts`, the unit
tests and the e2e specs it touches). The steps run one at a time: 01/01 to 02/02 share the bindings and `AppInfo`;
`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand.

## Conclusions

To be written when plan 03 is done: what was built, what changed from the design, the calls taken while building,
and the user's walk.
