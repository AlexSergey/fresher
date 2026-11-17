# 016/01/01 — Installed apps as bundles

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
testing, sandbox

## Files

- `crates/testing/src/app_bundle.rs` (new) — `AppBundle { path, bundle_id, name, version, build }` and its writer:
  `<path>/Contents/Info.plist` as an XML property list (the keys of the design, section 7; text escaped), and an empty
  `Contents/MacOS/` folder. No dependency: the plist is a fixed template.
- `crates/testing/src/tree_description.rs` — `[[app]]` in the tree vocabulary (`path`, `bundle-id`, `name`, optional
  `version` and `build`); checked: the path ends in `.app` and is a plain relative path; the bundle id passes
  `BundleId::new`. `merge` refuses one path written twice, as for files. Fixture homes may use it too.
- `crates/sandbox/src/case.rs` — `[[case.app]]` (the tree's `[[app]]` plus `running`, read in 01/02); refused outside
  `Applications/`, `System/Applications/` or `home/Applications/`.
- `crates/sandbox/src/generate.rs` — writes the bundles; on a sandbox it made before, rewrites them as for files.
- `crates/sandbox/src/expected.rs` — "Installed: <name> <version>" under a case with apps.

## Produces

Real bundles plan 009's `MacInstalledApps` reads; `[[case.app]]` for 01/02 and 03/01.

## Checks

- Negative first: a bundle id that is not reverse-DNS, a path not ending in `.app`, an app outside the apps folders, two
  apps at one path, a `..` in the path are refused, nothing written.
- Positive: `MacInstalledApps` (with `FakeKnownFolders` on the generated sandbox's apps folders) lists exactly the
  declared apps with their ids, names and versions; an app with no `version` lists `None`; a name with `&` and `<` is
  read back the same.
- `cargo nextest run -p fresher-testing -p fresher-sandbox -p fresher-system`.

## Conclusions

Built as written. `[[app]]` in the tree vocabulary (`AppBundle` in `fresher-testing`) writes a bundle with an XML
`Info.plist`; `[[case.app]]` adds `running`. A case may declare apps only in an apps folder (`Applications/`,
`System/Applications/`, `home/Applications/`, directly or one folder below), and nothing but apps goes into
`home/Applications/`. The generator counts a bundle's files as its own, and `EXPECTED.md` lists "Installed: <name>
<version>". Changed from the brief: the one-folder-below limit was added to the check, so a declared app is always one
the reader sees. Not done: a check that `MacInstalledApps` lists exactly the declared apps of a generated sandbox, as a
test of its own (the generator and the tree share the same writer).
