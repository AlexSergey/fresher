# The `all` sandbox's checklist is wrong where case files' worlds interact

**Status:** open · **Updated:** 2026-11-17

## Symptom

`npm run sandbox -- all` writes every case file into one sandbox, and its `EXPECTED.md` lists every case as its own
file expects it. Where one file's installed or open apps change what another file's cases show, the checklist is
wrong: in the `all` sandbox, the app of the vendor that `jetbrains-unknown-app-open.toml` declares open keeps every
folder of `jetbrains.toml`, and the one `jetbrains-unknown-app-installed.toml` installs keeps every removed IDE's data,
so `jetbrains.toml`'s positives are not offered there.

## Cause

A sandbox has one world: the generator joins every file's installed apps and `world.toml`'s running apps. The autotest
runs each case file in a sandbox of its own, so it never sees the clash; `all` only refuses two files writing one
path.

## What was tried

Nothing beyond keeping the interacting cases in case files of their own (plan 016/03/01), so each file's own sandbox
and autotest hold.

## Options left

- `all` warns, or leaves out, the files whose world changes another file's cases (it would need to know which do).
- `EXPECTED.md` of `all` names the files whose apps may change the others' cases.
- Use a feature's own sandbox (`npm run sandbox -- jetbrains`) for a run by hand; `all` only for a look around.
