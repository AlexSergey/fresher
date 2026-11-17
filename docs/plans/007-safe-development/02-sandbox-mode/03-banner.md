# 007/02/03 — The banner

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** app, app/ui

## Files

- `app/tauri/src/commands.rs` — `app_version` answers `AppInfo { version, build }`, with
  `Build::Release | Build::Development | Build::Sandbox { path }` (a `ts-rs` type; `app/tauri/tests/bindings.rs` and
  `bindings.generated.ts` updated).
- `app/ui/src/ipc/` — the typed call; `app/ui/src/product/build-banner/` (new): a thin, quiet strip above the frame —
  "Sandbox · <path>" or "Development build · nothing can be removed outside a sandbox"; nothing in a release build. Its
  texts in `app/ui/src/locales/en.json`; its stories in the gallery.
- `app/ui/src/screens/app-frame*` — the banner in the frame.
- `app/ui/e2e/` — the mock answers each build; the visual and axe specs cover the banner.

## Produces

The visible state of every development window.

## Checks

- Negative: `a_release_build_shows_no_banner`.
- Positive: `the_sandbox_banner_names_its_folder`, `the_development_banner_says_nothing_can_be_removed`; the gallery's
  screenshots; axe passes.
- `npx vitest run` on the changed files; `npm run test:e2e:ui`.

## Conclusions

Note: the command was called `app_version` when this step was written; the final review renamed it `app_info`.

The banner names the sandbox folder or says the build is a development build (nothing can be removed there); a release build
shows none. Deviation: `AppInfo` and `Build` live in `fresher-core`, not in `commands.rs`, so the bindings test can reach
them. The UI replaced `getAppVersion` and `use-app-version` with `getAppInfo` and `use-app-info`. Final review: a negative
test of the serialized build and a positive for a development start without the variable.
