# The protection lets a link shaped like an app bundle through

**Status:** open · **Updated:** 2026-11-22

## Symptom

In the apps folder, `Protection::check` lets through `/Applications/<name>.app` and `/Applications/<sub>/<name>.app`
by their names alone. A link with such a name (on a Mac, `/Applications/Safari.app` is a link into the system's
cryptexes) passes it as if it were a bundle, and so does a bundle-shaped path through a link
(`/Applications/Link/B.app` with `Link` a link).

The same holds for a name the file system takes for another. Below an app folder a bundle is also refused when its
names match a `never`, `whole` or ignored place inside that folder ignoring case (`/Applications/fresher.app` against
Fresher's own bundle), but the names are only lowered with `to_lowercase`: no Unicode case folding and no
normalisation is done. A variant the case-insensitive volume reads as the same bundle but that differs by folding
(`Freſher.app` with a long `ſ`, a ligature such as `ﬃ`) or by normal form (a decomposed `é` against a composed one)
passes the lexical check.

## Cause

The protection is lexical by design: it reads path components only, no IO (plan 036/01/02). Whether a name is a real
folder, a link or a file is not its to know, nor which names the volume takes for the same one: that needs the
volume's own folding tables or a call to the disk.

## What was tried

Nothing in the protection: reading the disk there would break its contract (pure, the same answer for the same path).

## How it is kept safe

- The uninstall listing names bundles as the system lists them, with the names the disk holds (never a name the
  interface sends), and reads each with `lstat`: only a real folder with a real `Contents` folder and a
  `Contents/Info.plist` file, in a real folder, has facts; a bundle that is a link, a folder without `Info.plist` or a
  file named `X.app` is "the bundle cannot be read", never offered (plan 036/01/03, the bundle facts;
  `crates/system/src/bundle_facts/mod.rs`).
- The uninstall plan is built only from that listing and refuses a bundle unless its canonical path
  (`fs::canonicalize`) equals the listed path byte for byte and it is a real folder with `Contents/Info.plist`, so a
  case, folding or normal-form variant and a link never reach the plan (plan 036/02/01; tests with `Freſher.app` and
  the decomposed form).
- The cleaner opens every item's parents through no link (check 5, `O_NOFOLLOW_ANY`) and checks both the physical and
  the lexical form of a path, so a path through a link never reaches a bundle's inside; a link item is acted on as
  itself, never its target.

## Options left

None needed while the three walls above stand; a test in 01/03 and 02/01 keeps each of them. The general gap
behind the cleaner's own check (names compared as written, for every `never` place) is its own issue:
[`cleaner-protection-compares-names-as-written.md`](./cleaner-protection-compares-names-as-written.md).
