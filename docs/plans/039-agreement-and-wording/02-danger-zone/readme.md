# 039/02 — The danger zone

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 ·
**Modules:** core, engine, testing, cleaner (two test files), e2e, app/tauri, app/ui

Part A. The last screen before a removal looks as serious as it is: Review shows "Safe to remove" and a "Danger zone"
apart, the confirmation of a High plan asks for its own tick and the core refuses a High plan without it, and Scan's
"Select all" of a risky zone asks first. The design: [`../00-design.md`](../00-design.md), section 2.

Refreshed on 2026-11-23: the echo builders are listed by place (the cleaner's two test files and `crates/testing`
among them), the confirmation has a second owner (an app's page, plan 036), and the High plans the UI already
confirms (an uninstall of everything, a Largest pick, Xcode archives, a large untouched download) tick in 02/02.

## Steps

| #   | Step                                                                   | Status      | Shared files                                                                                          |
| --- | ---------------------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------------------------- |
| 01  | [The core's High consent](./01-high-consent.md)                        | 🔨 awaiting the user's walk | `crates/core/src/{cleanup/confirm.rs,error.rs}`, `crates/core/tests/compile_fail/`, `crates/engine/src/cleanup/`, the echo helpers, `app/ui/src/state/{cleanup.ts,cleanup-actions.ts}`, `app/ui/e2e/support/cleanup-mock.ts`, `docs/restrictions/deletion-safety.md` |
| 02  | [Review, the confirmation and Scan](./02-review-and-confirmation.md)   | 🔨 awaiting the user's walk | `app/ui/src/{product/confirm-dialog,product/danger-zone,screens/review,screens/scan,screens/apps/app-screen.tsx}`, both dictionaries, the e2e specs |

01, then 02.

## Conclusions

The danger zone is built as designed (steps 01 and 02): the core refuses a High plan without its tick, Review shows the
safe part and the danger zone apart, the confirmation asks for the tick every time, and Scan asks before a Select all
of a Medium or High zone. `crates/cleaner` changed in two test echoes only.
