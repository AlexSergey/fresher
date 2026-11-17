# 011/01 — Problem texts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** core,
cleaner, app, app/ui

Closes the error gaps the audit found ([`../00-design.md`](../00-design.md), "What already exists" and sections 1–2):
the rule that a technical detail is an arg the sentence never names, the tool's message moved into it, the tests that
keep an error's `Display` out of the UI, a failed start that says so, and the UI's side of it.

## Steps

| #   | Step                                                    | Status                                  | Shared files                                                                                                                   |
| --- | ------------------------------------------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 01  | [The detail rule in the core](./01-detail-rule.md)      | ✔️ done | `crates/core/src/{error,text,texts}.rs`, `crates/core/src/cleanup/outcome.rs`, `crates/core/locales/en.json`, `crates/cleaner/src/remove/tool.rs` |
| 02  | [A failed start says so](./02-failed-start.md)          | ✔️ done | `app/tauri/src/lib.rs`, `crates/core/locales/en.json`, `crates/core/src/texts.rs`                                              |
| 03  | [The UI shows only texts](./03-ui-texts.md)             | ✔️ done | `eslint.config.ts`, `app/ui/src/product/{result-summary,module-section}/`, `app/ui/src/locales/en.json`                        |

01 first; then 02 and 03 in parallel (02 is `app/tauri` and one core text, 03 is `app/ui` and the lint).

## Conclusions

All three steps built. A technical detail is an arg no sentence names and the UI shows only on demand; a failed start
says so in a native window instead of the app vanishing; the UI renders only texts, with a name-based lint as a
tripwire. The start-failure window is checked by hand in 03/01. Calls: see the steps (the non-blocking dialog with the
destroyed window and the held exit; the lint is a tripwire, not a guarantee).

