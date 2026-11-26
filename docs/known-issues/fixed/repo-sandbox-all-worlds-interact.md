# The `all` sandbox's checklist is wrong where case files' worlds interact

**Status:** fixed (the `all` checklist opens with a warning naming the files whose worlds join; the interaction itself stays by design) · **Updated:** 2026-11-26

## Symptom

`npm run sandbox -- all` writes every case file into one sandbox, and its `EXPECTED.md` lists every case as its own
file expects it. Where one file's installed or open apps change what another file's cases show, the checklist is
wrong: in the `all` sandbox, the app of the vendor that `jetbrains-unknown-app-open.toml` declares open keeps every
folder of `jetbrains.toml`, and the one `jetbrains-unknown-app-installed.toml` installs keeps every removed IDE's data,
so `jetbrains.toml`'s positives are not offered there; and `app-leftovers-spotlight-off.toml` turns Spotlight off for
the whole sandbox, so `app-leftovers.toml`'s positives (and any module's that asks Spotlight) are kept there. Access
joins as Spotlight does: the most restrictive wins, so `downloads-denied.toml` denies Downloads for every case file of
`all`, and `downloads.toml`'s positives are not offered there.

The same holds for Xcode: `xcode-open.toml`'s running Xcode joins the shared world of `all`, so every positive of
`xcode.toml` is left out there as `app-open`; and `dev-caches-xcode-open.toml` does the same to the derived data project
of `dev-caches.toml`, which is not offered there.

The installed-app worlds interact too: in `all`, `app-caches` offers the cache of every app installed by any file, so
`app-leftovers.toml`'s installed Editor (its `Library/Caches/com.example.Editor`) is offered by `app-caches` there, which
that file's checklist does not say.

`all` orders the case files by file name without `.toml` (so a base file precedes its variants) and leaves out whole a
case file that writes a path an earlier file wrote, naming it on stderr and in the checklist; a left-out file is
written by name (`npm run sandbox -- <name>`). The `unused-toolchains*` files do: rustup's `settings.toml` and nvm's
`alias/default` sit at fixed places and the module's settings and access are shared by every case of a file, so each
world needs them in its own file. In `all`, `unused-toolchains` stays and its three variants (`-documents-denied`,
`-off`, `-unknown`) are left out. The main file is written there but the module does not run: `downloads-denied.toml`
denies Downloads for every case file of `all`, and the module needs Downloads. The same holds for the Ollama store,
which is one folder of one home: a left-out of the whole store (an unreadable manifest, a download in progress) hides
every model beside it. In `all`, `local-ai-models-ollama` stays and `-ollama-unreadable` and `-ollama-download` are
left out, each writing the lone layer the first writes so that the clash leaves them out. A run by hand uses the
feature's own sandbox.

## Cause

A sandbox has one world: the generator joins every file's installed apps and `world.toml`'s running apps and
Spotlight's answers. The autotest runs each case file in a sandbox of its own, so it never sees the clash; `all` only
leaves out a file writing a path an earlier file wrote.

## Fix

The `EXPECTED.md` of a sandbox of several case files opens with a warning: the files share one world, a case can show
differently than its own file expects, and a check by hand uses a feature's own sandbox (`npm run sandbox -- <name>`).
It names the files whose world joins the others' (a case installs or opens an app, turns Spotlight off or sets an
access), read from the case files by the generator (`sets_the_world` in `crates/sandbox/src/expected.rs`), so no list is
kept by hand. The interaction itself stays by design: one sandbox has one world, and the autotest runs each file in a
sandbox of its own.
