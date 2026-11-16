# 003/08/03 — Docs and version

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-16 · **Modules:** docs, repo, app

## Files

- `docs/architecture/README.md` — a "UI" section: the folders `design/`, `product/`, `screens/`, `gallery/` and their
  import boundaries; the store keeps totals, lists are paged; the window (overlay title bar, the translucent sidebar,
  `macOSPrivateApi` and why it is acceptable outside the Mac App Store); the two window permissions of the capability.
- `docs/testing.md` — the gallery and its screenshots (WebKit), the screens' screenshots (both engines), motion off in
  tests, the CSS guard, the contrast test, `npm run build:e2e`, `npm run test:perf` and when to run it.
- `docs/quality.md` — the CSS guard and the import boundaries among the checks.
- `docs/goal.md` — section 6 names the visual direction "Graphite and Iris" and links the design, if the user approved
  it with the plan.
- The version files — `0.2.0` in the root `Cargo.toml` (`[workspace.package]`), `app/tauri/tauri.conf.json`, the root
  and `app/ui` `package.json`; `Cargo.lock` and `package-lock.json` regenerated.
- `docs/plans/003-base-ui/` — the conclusions of every plan and of the global plan: what was built, the measurements of
  08/01, the deviations (each recorded as an agent's call in the design or a note in its step), the checks by hand.

## Produces

Docs that describe what was built; the release number of the base UI.

## Checks

- `npm run pre-push` passes on the branch's last commit; coverage at or above 90 %.
- `npm run test:e2e` passes.
- `npm run build` bundles `Fresher.app` 0.2.0; its size is written in the conclusions.
- The plan's "Checks by hand" list is complete for the user: the native window (04/01), VoiceOver and reduced motion
  (08/02), a demo scan end to end in `npm run tauri dev`.

## Conclusions

Done on 2026-11-16.

- **The version is 0.2.0** (was 0.1.0) in the root `Cargo.toml` (`[workspace.package]`, so every crate),
  `app/tauri/tauri.conf.json`, the root and the `app/ui` `package.json`; `Cargo.lock` and `package-lock.json`
  regenerated (only the workspace's own version lines changed).
- **`npm run build`** bundles `Fresher.app` 0.2.0 (`CFBundleShortVersionString`), **6.94 MiB** (the release binary
  5.9 MiB); unsigned.
- **Docs**: `docs/architecture/README.md` has a "UI" section (the folders and their import boundaries, the state, the
  window and its two permissions); `docs/testing.md` describes the gallery, the screenshots, motion off, the guards,
  the accessibility specs, what the core mock can do and the performance spec (`test:perf`, the projects `perf` and
  `perf-chromium`); `docs/quality.md` names the import boundaries and the design system's guards, and says the
  performance spec is run by hand only; `docs/goal.md` section 6 and `docs/rules.md` name the visual direction
  "Graphite and Iris" (approved with the plan on 2026-11-15, agent's call 12), linking the design.
- [`ui-findings-kept-in-memory.md`](../../../known-issues/ui-findings-kept-in-memory.md) is closed as fixed by plan
  05, with the heap measured in 08/01.
- The checks: `npm run pre-push` passes (UI coverage 98.9 % lines, 92.5 % branches, 94.9 % functions, 97.9 %
  statements; Rust lines over the 90 % floor); `npm run test:e2e` passes (the core e2e, 6 tests; Playwright, 157
  tests in the gallery, WebKit and Chromium projects). The checks by hand are listed in the global plan's
  conclusions, "Checks by hand".
