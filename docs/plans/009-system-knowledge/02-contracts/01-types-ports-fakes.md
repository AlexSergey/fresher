# 009/02/01 — Types, ports and fakes

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** core, testing

## Files

- `crates/core/src/system/` (new) — `ids.rs`: `BundleId` (reverse-DNS, compared ignoring case), `TeamId` (ten
  capitals and digits); `apps.rs`: `InstalledApp`, `InstalledApps`; `running.rs`: `RunningApp`, `RunningApps`,
  `RunningSet::owner_of`; `projects.rs`: `ProjectKind` (markers and artifacts as data), `Project`; `error.rs`:
  `SystemError`.
- `crates/testing/src/system_fakes.rs` (new) — `FakeInstalledApps`, `FakeRunningApps` (a script of answers, counting
  calls).

## Produces

The contracts of 03, 04 and 05.

## Checks

- Negative: a bundle id that is empty, has a space or a slash, or starts with a dot is refused; a team id of nine
  characters or lowercase is refused; `owner_of` does not take `com.example.Appx` for `com.example.App`, nor a prefix
  inside a name; an empty set owns nothing.
- Positive: ids compare ignoring case; `owner_of` finds the id as a whole name and as `<id>.<suffix>`, in any case;
  every project kind has at least one marker and no marker belongs to two kinds; the fakes answer their script.

## Conclusions

- Built as written, in `crates/core/src/system/` (`ids`, `apps`, `running`, `projects`, and `SystemError` in `mod.rs`).
- `BundleId` keeps its spelling and compares, hashes and orders by its lower-case form; ASCII letters, digits, `.`,
  `-`, `_`, no leading, trailing or double dot, at most 255 bytes. `AppIdError` names the refused id.
- `InstalledApp::owns(id)` (its own id or a helper's) came with the type; `RunningSet::owner_of` matches a whole name
  or the id followed by a dot.
- The fakes: `FakeInstalledApps` (a list or an error) and `FakeRunningApps` (a script, the last answer repeating),
  both counting calls; `installed_app`, `signed_by` and `running_app` make up an app in one line and answer an
  `AppIdError` for a wrong id, so a fake never panics.
