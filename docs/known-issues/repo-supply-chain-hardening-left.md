# Supply-chain hardening left for later

**Status:** open · **Updated:** 2026-11-25

## Symptom

The security audit of plan 041 (finding SUPPLY-8, low; part of SUPPLY-6) listed hardening that is not done:

- `.npmrc` has no `ignore-scripts=true`: `npm ci` runs the lifecycle scripts of the root and of every package that has
  one (today only `fsevents`, a dev dependency), on CI and on a reviewer's machine.
- CI runs neither gitleaks (`npm run check:secrets`) nor the staged-files guard: a contributor without the hooks is not
  checked for secrets until review.
- `tauri-plugin-fs` is compiled into the release binary through `tauri-plugin-dialog`. No `fs:` permission is granted
  (`app/tauri/capabilities/default.json`, held by its test), so the page cannot reach it, but it is code the app does
  not use.
- The CI runner image is `macos-latest`, which moves (Xcode, the SDK): a release built on it is not reproducible.
- The bundle is arm64 only; whether 0.1.0 ships arm64 only or `universal-apple-darwin` is not decided.

## Cause

Each is cheap alone but changes how the repository is set up or built (the hooks are installed by `prepare`, a
lifecycle script; the folder picker comes from the dialog plugin), and none is a defect in shipped code.

## What was tried

Nothing yet.

## Options left

- `ignore-scripts=true` in `.npmrc`, with `npx husky` as an explicit setup step, and a line in CONTRIBUTING that
  checking out a pull request and running any `npm` or `cargo` command runs its code.
- A gitleaks step in CI's `lint` job (installed by the pinned install action, as the other tools).
- The folder picker through the system's own panel from Rust (`fresher-macos`), dropping `tauri-plugin-dialog` and
  with it `tauri-plugin-fs` and `rfd`.
- The release workflow (plan 013) pins its runner image and decides the architectures.
