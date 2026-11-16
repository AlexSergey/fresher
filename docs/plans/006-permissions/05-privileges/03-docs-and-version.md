# 006/05/03 — Docs and version

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** docs, app

## Files

- `docs/architecture/permissions.md` (new) — the capability model in short (capabilities, states, needs,
  availability, coverage, removability), where the macOS knowledge lives, the rule "checks never prompt; a prompt only
  inside a scan the user started, for a place the user opted into", how a module declares a need, and **developer
  builds**: grants follow the code signature, an ad-hoc rebuild loses them, `npm run tauri dev` is attributed to the
  terminal app, the local signing identity of plan 013/01, and how to test each state (the gallery and `FakeAccess`
  first; a real grant by hand, reset with `tccutil reset All io.github.alexsergey.fresher` on a development Mac only).
- `docs/architecture/README.md` — the `AccessProbe` row of the ports table points at the real adapter, a `SystemUi`
  row is added, a link to `permissions.md`.
- `docs/architecture/modules.md` — the descriptor's `with_needs` and `with_tool_check`, one line in the checklist
  ("every category that cannot see everything without a permission declares it").
- `docs/restrictions/privacy.md` — "Least permissions": enforced by the engine's coverage step and its tests, and the
  `Info.plist` test; no longer "planned".
- `docs/restrictions/deletion-safety.md` — a rule "No administrator rights": the app never elevates; an item it may not
  remove is shown, never selected; enforced by `Removability`, the `Info.plist` test (no privileged helper) and review.
- `docs/testing.md` — `FakeAccess` and `FakeSystemUi` in the list of fakes.
- The version files (`Cargo.toml`, `tauri.conf.json`, both `package.json`) — 0.4.0, unless plan 005 already moved them
  (design, agent's call 18).
- `docs/plans/006-permissions/readme.md` and `00-design.md` — the conclusions: what was built, measured and checked by
  hand, deviations as agent's calls.

## Produces

The written contract every later module and plan 008 follows; the plan's conclusions for the user to look at.

## Checks

- `npm run pre-push` passes on the branch's last commit; coverage ≥ 90 % in the Rust crates and the UI.
- Every link in the changed docs resolves; the docs name no other product (clean room).
- The checks by hand of the global plan's "Acceptance" are done on a real Mac and written in the conclusions.
- `cargo clean` at the end.
