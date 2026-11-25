# 02 — Package

**Status:** 🔨 in progress (built, awaiting the user's look) · **Version:** 0.1.0 · **Date:** 2026-11-25 · **Modules:** repo, scripts, app, docs

Written and approved in chat on 2026-11-25 ("the release CI — go"; see the global plan's "Decisions"). Design:
[`../00-design.md`](../00-design.md), "The package". One file: the plan is one step.

## What

- **One universal build** for Apple silicon and Intel (`--target universal-apple-darwin`; both targets are already in
  `rust-toolchain.toml`), signed ad hoc with the hardened runtime as before; macOS 13 stays the minimum
  (`bundle.macOS.minimumSystemVersion`).
- **A disk image as the download**: Tauri's own `dmg` bundle, `Fresher_<version>_universal.dmg`, holding the app and a
  link to Applications. `bundle.targets` stays `["app"]`, so `npm run build` (CI's `build` job, a developer's build)
  is unchanged.
- **`npm run build:release`**: `tauri build --target universal-apple-darwin --bundles app,dmg -- --locked` with the
  home folder remapped, then `npm run check:bundle -- --release`. It sets `CI=true`: Tauri's disk image script then
  skips laying out the image's window through Finder (AppleScript), so a local build opens no window and needs no
  automation permission, as on a CI runner.
- **The bundle check** (`scripts/check-bundle.ts`), `--release`: the universal app passes every check of the app
  (strict deep verify, the `runtime` flag, no entitlements, no network import, no home folder) and its executable
  holds `arm64` and `x86_64` (`lipo -archs`); exactly one `Fresher_<version>_universal.dmg` is in `bundle/dmg/`; it
  verifies (`hdiutil verify`), opens read-only and hidden from Finder at a temporary mount point, and the app inside
  passes the same checks; the image is detached whatever they say, and only the empty mount folder is removed.

## Files

`package.json` (`build:release`), `scripts/check-bundle.ts` and its test, `docs/quality.md` (Layer 6),
`docs/restrictions/privacy.md` ("No foreign code under the app's access").

## Checks

- `node --test scripts/check-bundle.test.ts`: a missing architecture, an unlistable executable, an image that does not
  verify or open, an app inside that fails, an image left open, no image or two are refused; a universal app and a
  good image pass, and the image is detached.
- `npm run build:release` on the developer's Mac, with the worktree's own target folder.

## Conclusions

Built on 2026-11-25. `npm run build:release` built both architectures (2 min 23 s and 2 min 10 s of the release
profile on the developer's Mac, with a warm registry) and the image: `Fresher.app` 21.9 MiB (`Mach-O universal
(x86_64 arm64)`, `flags=0x10002(adhoc,runtime)`, identifier `io.github.alexsergey.fresher`), the image 9.6 MiB
(UDZO, zlib), holding `Fresher.app`, `Applications -> /Applications` and the volume icon. `check:bundle -- --release`
passed on it; pointed at a missing folder it named every missing part.

Not checked yet: the app opened from the image on another Mac, on Intel, and on macOS 13 — the release checklist's
step 8 ([`../../../workflow.md`](../../../workflow.md), "Releases"). The image itself is not signed: an ad hoc
signature on it would prove nothing, and Gatekeeper checks the app.
