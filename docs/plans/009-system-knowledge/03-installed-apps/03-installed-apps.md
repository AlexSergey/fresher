# 009/03/03 — The apps

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** system, app

## Files

- `crates/system/src/apps.rs` (new) — `MacInstalledApps::new(Arc<dyn KnownFolders>)`: the bundles in the apps
  folders and one folder below, never inside a bundle or through a link; each bundle's `Info.plist`
  (`CFBundleIdentifier`, `CFBundleShortVersionString`, `CFBundleVersion`, `CFBundleDisplayName`/`CFBundleName`,
  `CFBundleExecutable`), the team id of `Contents/MacOS/<executable>`, the helpers' ids; sorted by path.
- `crates/system/src/bundle.rs` (new) — reading one bundle (shared with 04).
- `app/tauri/src/compose.rs` — `MacInstalledApps` in the system ports.

## Checks

- Negative: a bundle without `Info.plist` or bundle id is skipped; a link to a bundle is not followed; a bundle inside
  a bundle is not an app; a missing apps folder answers no apps.
- Positive: a fixture with an app, a vendor folder and helpers gives each app's facts; the read-only check on
  `/System/Applications` finds apps with `com.apple.` ids (skipped where the folder is missing).

## Conclusions

- Built as written: `bundle.rs` reads one bundle (its `Info.plist` through a link-refusing, 1 MiB-capped read; the
  name is the display name, else the name, else the folder's), its team id (`Contents/MacOS/<executable>`, a plain
  name only) and its helpers; `apps.rs` lists the apps folders. A folder named twice is listed once.
- An apps folder that cannot be read is logged; only when none can be read is it an error. A missing folder is no
  error.
- The app composes `MacInstalledApps` over the same `MacKnownFolders` as the scanner's anchor; the stand-in
  `NoInstalledApps` is gone.
- The read-only check of this Mac's `/System/Applications` passes: every app there has a `com.apple.` id.
