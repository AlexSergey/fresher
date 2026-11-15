# Unused toolchain versions

**Since:** 2026-11-15 · **Proposed by:** the agent

## Problem

The case that started the project — every old WebStorm version left on disk — happens with language toolchains too.
Version managers install a new version next to the old one and never remove anything: a Rust toolchain per nightly
tried, a Node version per project from years ago, three JDKs, Android system images for emulators deleted long ago.
Each is hundreds of megabytes to several gigabytes.

[Developer Environment Cleaner](./developer-environment-cleaner.md) covers Xcode and IDE versions; this idea covers
the version managers and SDKs it does not.

## Direction

The rule is **"not used", not "old"**: a version is offered only when nothing on the Mac points at it.

| Toolchain     | Where versions live                                                       | What marks a version as used                                                                   |
| ------------- | ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Rust (rustup) | `~/.rustup/toolchains/<name>`                                             | the default toolchain, a directory override, a `rust-toolchain(.toml)` in a project            |
| Node          | `~/.nvm/versions/node/`, fnm's and volta's version folders                | the default alias, a `.nvmrc` / `.node-version` / `volta` pin in a project                     |
| Python        | `~/.pyenv/versions/<version>`                                             | the global version, a `.python-version` in a project, **a virtualenv built on it** (see risks) |
| Java          | `~/Library/Java/JavaVirtualMachines`, SDKMAN candidates, `~/.gradle/jdks` | `.java-version` / `.sdkmanrc` in a project, the SDKMAN default                                 |
| Android       | `~/Library/Android/sdk/system-images`, `ndk/`, `build-tools/`             | an AVD's `config.ini` naming the image, a project's `ndkVersion`                               |

- "A project" is any project found by [stale project cleanup](./stale-project-cleanup.md) in the folders the user
  chose; the finding says so: "not used by any project in `~/dev`".
- One finding per version: the toolchain, the version, its size, the last time it was used when that can be told, and
  how it comes back (`rustup toolchain install 1.82`).
- Removal through the manager's own command where it has one (`rustup toolchain uninstall`, `nvm uninstall`,
  `pyenv uninstall`, `sdkmanager --uninstall`), so its records stay consistent.
- Danger level **Low** (reinstalled with one command); **Medium** for a Node version that holds global packages
  (they go with it — the finding lists them) and for an AVD (an emulator's own data: apps, accounts, files).

## Risks — when a rule must NOT claim a version

- **A Python virtualenv points at its base interpreter** (`home = …` in `pyvenv.cfg`): removing the base breaks every
  venv built on it, with a confusing error. A version that any found venv uses is not claimed.
- **Pins outside the chosen folders** cannot be seen. That is why the level is never Safe, and the finding states
  which folders were checked.
- **A default set by the shell profile** cannot always be read reliably; when the default is unknown, the toolchain's
  newest version and the default alias are both kept.
- The only installed version of a toolchain is never offered, even if no project uses it.
- JDKs in `/Library/Java/JavaVirtualMachines` need administrator rights: shown, cleaned only with the separate
  permission step of [System Junk Cleaner](./system-junk-cleaner.md).

Open questions:

- Gradle's auto-provisioned JDKs come back on the next build by themselves: **Safe** instead of Low?
- Should AVDs be in this idea at all, or with the simulators in
  [Developer Environment Cleaner](./developer-environment-cleaner.md)?
- The managers' own commands delete for good, while our default is the Trash
  ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)): which wins for a toolchain?
