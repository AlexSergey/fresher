# The `all` sandbox's checklist is wrong where case files' worlds interact

**Status:** open · **Updated:** 2026-11-19

## Symptom

`npm run sandbox -- all` writes every case file into one sandbox, and its `EXPECTED.md` lists every case as its own
file expects it. Where one file's installed or open apps change what another file's cases show, the checklist is
wrong: in the `all` sandbox, the app of the vendor that `jetbrains-unknown-app-open.toml` declares open keeps every
folder of `jetbrains.toml`, and the one `jetbrains-unknown-app-installed.toml` installs keeps every removed IDE's data,
so `jetbrains.toml`'s positives are not offered there; and `app-leftovers-spotlight-off.toml` turns Spotlight off for
the whole sandbox, so `app-leftovers.toml`'s positives (and any module's that asks Spotlight) are kept there. Access
joins as Spotlight does: the most restrictive wins, so `downloads-denied.toml` denies Downloads for every case file of
`all`, and `downloads.toml`'s positives are not offered there.

`all` orders the case files by file name without `.toml` (so a base file precedes its variants) and leaves out whole a
case file that writes a path an earlier file wrote, naming it on stderr and in the checklist; a left-out file is
written by name (`npm run sandbox -- <name>`). The `unused-toolchains*` files do: rustup's `settings.toml` and nvm's
`alias/default` sit at fixed places and the module's settings and access are shared by every case of a file, so each
world needs them in its own file. In `all`, `unused-toolchains` stays and its three variants (`-documents-denied`,
`-off`, `-unknown`) are left out. The main file is written there but the module does not run: `downloads-denied.toml`
denies Downloads for every case file of `all`, and the module needs Downloads. A run by hand uses the feature's own
sandbox.

## Cause

A sandbox has one world: the generator joins every file's installed apps and `world.toml`'s running apps and
Spotlight's answers. The autotest runs each case file in a sandbox of its own, so it never sees the clash; `all` only
leaves out a file writing a path an earlier file wrote.

## What was tried

Nothing beyond keeping the interacting cases in case files of their own (plan 016/03/01), so each file's own sandbox
and autotest hold.

## Options left

- `all` warns, or leaves out, the files whose world changes another file's cases (it would need to know which do).
  This is a different issue from the path clash above: the apps, Spotlight and access interaction is not a path two
  files write, so `all` does not see it.
- `EXPECTED.md` of `all` names the files whose apps may change the others' cases.
- Use a feature's own sandbox (`npm run sandbox -- jetbrains`) for a run by hand; `all` only for a look around.
