# 024/03/01 — The case files

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. Four files, since settings and access are shared by every case of a file, and a
manager's pointer files (`settings.toml`, `alias/default`) are written by one case only. Pins and settings use 01/02's
`text` and `[settings]`. The design: [`../00-design.md`](../00-design.md), section 9.

## Files

- `fixtures/sandboxes/unused-toolchains.toml` (new) — `feature = "unused-toolchains"`, `module = "unused-toolchains"`,
  `[settings] "unused-toolchains.node-versions" = true`; a head comment: what the file covers, that toolchain and
  version folders hold one small `bin/` file with `fill = "data"`, that pins are `text`. With these exact names:
  - negative, in this order:
    - "rustup's default and its override are never offered" — `home/.rustup/settings.toml` (`default_toolchain =
      "1.80.0-aarch64-apple-darwin"`, `[overrides]` naming `nightly-2024-04-01-aarch64-apple-darwin`) and both
      toolchains;
    - "a moving channel is never offered" — `home/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustc`,
      `nightly-aarch64-apple-darwin/bin/rustc`;
    - "a toolchain a project's rust-toolchain.toml names is never offered, in Documents too" —
      `home/Documents/dev/pinned/{Cargo.toml,rust-toolchain.toml}` (`channel = "1.79"`) and
      `home/.rustup/toolchains/1.79.0-aarch64-apple-darwin/bin/rustc`;
    - "a legacy rust-toolchain file keeps its toolchain" — `home/Code/legacy/rust-toolchain` (`nightly-2024-03-01`)
      and its toolchain;
    - "a .tool-versions keeps its toolchain and its Node version" — `home/Code/asdf/.tool-versions` (`rust 1.77.2`,
      `nodejs 18.19.0`), the toolchain `1.77.2-…` and `home/.nvm/versions/node/v18.19.0/bin/node`;
    - "a linked toolchain and a folder off the shape are never offered" — `[[case.symlink]]`
      `home/.rustup/toolchains/dev` → `../../Code/rust/build/stage1` (with its target folder), and
      `home/.rustup/toolchains/my-custom/bin/rustc`;
    - "nvm's default and the newest Node version are never offered" — `home/.nvm/alias/default` (`20`),
      `v20.11.0`, `v22.1.0`;
    - "an alias chain keeps its version" — `home/.nvm/alias/myproj` (`lts/hydrogen`), `home/.nvm/alias/lts/hydrogen`
      (`v18.20.0`), `v18.20.0`;
    - "a project's .nvmrc keeps its version, in Documents too" — `home/Documents/dev/web/{package.json,.nvmrc}`
      (`16`), `v16.20.2`;
    - "a .node-version keeps its version" — `home/Code/app2/.node-version` (`v19.9.0`), `v19.9.0`;
    - "a toolchain with a socket inside is kept" — `home/.rustup/toolchains/1.60.0-aarch64-apple-darwin/{bin/rustc,lib/s}`
      (`kind = "socket"`); left out `unused-toolchains.left-out.special` 1 in `unused-toolchains.rust`;
  - positive:
    - "an old toolchain nothing points at goes" — `home/.rustup/toolchains/1.70.0-aarch64-apple-darwin/bin/rustc`;
      in `unused-toolchains.rust`, low, `in-trash`; `[case.map]` `explained`;
    - "a dated nightly nothing points at goes" — `nightly-2023-06-01-aarch64-apple-darwin`; low, `in-trash`;
    - "a pin inside node_modules keeps nothing" — `home/Code/web3/package.json`,
      `home/Code/web3/node_modules/dep/.nvmrc` (`14`), `home/.nvm/versions/node/v14.21.3/bin/node`; in
      `unused-toolchains.node`, medium, `in-trash`;
    - "an old Node version nothing points at goes, at Medium" — `v12.22.12`; medium, `in-trash`.
- `fixtures/sandboxes/unused-toolchains-unknown.toml` (new) — the switch on; no `settings.toml`; negative "without
  rustup's settings no toolchain is offered" — `1.71.0-aarch64-apple-darwin`; left out
  `unused-toolchains.left-out.default-unknown` 1 in `unused-toolchains.rust`; negative "an nvm pin that cannot be
  resolved keeps every Node version" — `home/Code/odd/.nvmrc` (`lts/unknown`), `home/.nvm/alias/default` (`20`),
  `v20.11.0`, `v13.14.0`; left out `unused-toolchains.left-out.pin-unknown` 1 in `unused-toolchains.node`. File paths
  apart from the other files'.
- `fixtures/sandboxes/unused-toolchains-off.toml` (new) — no `[settings]` (the switch off); negative "no Node version
  is offered while the switch is off" — `home/.nvm/alias/default` (`20`), `v20.11.0`, `v11.15.0` (nothing claimed,
  nothing left out); positive "a toolchain still goes with the Node switch off" — `home/.rustup/settings.toml`
  (`default_toolchain = "stable-aarch64-apple-darwin"`), `stable-…`, `1.72.0-aarch64-apple-darwin`; low, `in-trash`.
- `fixtures/sandboxes/unused-toolchains-documents-denied.toml` (new) — a head comment as `downloads-denied.toml`'s (a
  world of its own; in `all` the most restrictive access wins); negative "nothing is offered without access to
  Documents" — `access = { documents = "denied" }`, `home/.rustup/settings.toml` (stable), `stable-…`,
  `1.73.0-aarch64-apple-darwin`: nothing claimed (the module is not run).
- `fixtures/golden/sandbox-unused-toolchains.json`, `-unknown.json`, `-off.json`, `-documents-denied.json` (new) —
  `UPDATE_GOLDEN=1`, reviewed: the findings, the left-out counts, the map, the cleanup's outcome, the tree after (the
  pins and the linked toolchain's target untouched), the module unavailable in the denied world.
- `app/tauri/tests/sandboxes.rs` — the four files in `case_files!`, sorted by file name.
- `docs/testing.md` — the four case files in the list of case files.

What the code does for each case is written down when the file is made; a case that differs from the design is fixed
in the module, not in the case file.

## Produces

The module's case files, for 04/02 (the runs by hand).

## Checks

- `npm run sandbox:test` passes for the four files and every other case file (their goldens unchanged).
- Once per kind, by hand, written in this step's conclusions: a danger changed (`medium` → `low` on a Node case) and a
  left-out's items changed (`1` → `2` on the socket case) each make the autotest fail; then reverted. Also once: the
  `[settings]` line removed from `unused-toolchains.toml` makes the Node positives fail.
- `npm run sandbox -- unused-toolchains` writes the sandbox with its settings file; `EXPECTED.md` lists every case,
  negatives first.
- `cargo nextest run -p fresher-sandbox -p fresher-app`.

## Conclusions

Built with these changes. The four case files share one world per file. The main file grants Desktop, Documents and
Downloads: without a grant the module still runs but does not read a pin in `Documents`, so 1.79.0 was offered;
granting only Documents makes the others unknown and the module does not run. `-unknown` and `-off` state no access
and say why in their head comments. `[settings]` of the main file turns the Node switch on. The first version of the
step made `npm run sandbox -- all` refuse (the fixed places `settings.toml` and `alias/default` are written by several
files); a fix round made `all` sort case files by stem and leave out whole any file writing a path an earlier one
wrote, naming it on stderr and in `EXPECTED.md`. So `all` holds the main file and leaves out `-documents-denied`,
`-off` and `-unknown`, which run by name. The denied golden has no "unavailable" marker, as the other denied goldens.
Mutations by hand, each reverted: a Node case at Low failed the autotest on the danger; two socket items instead of one
failed on the count; removing `[settings]` failed on the pin in `node_modules`.
