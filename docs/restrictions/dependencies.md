# Dependencies

The rules every dependency follows. The list of the dependencies themselves, with their versions and licences, is
[`../dependencies.md`](../dependencies.md); a change that adds, removes or updates one updates it in the same change.

## Permissive licences in the app

- **Rule:** third-party code that ships in the app — the Rust crates and the production npm dependencies of `app/ui` —
  only under MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, Unlicense, CC0-1.0. Anything else
  (MPL-2.0, LGPL, …) is asked about before it is added; GPL, AGPL, SSPL, non-commercial and unlicensed code are not
  taken. The whole Rust tree keeps this list, its build and test crates included.
- **Why:** the app may be distributed; copyleft or unclear licences would bind it.
- **Enforced by:** `cargo deny check` (`deny.toml` `[licenses]`) for the whole Rust tree; `npm run check:notices`
  (`scripts/notices.ts`, pre-push and CI) for the npm production dependencies, against the same allowlist, read from
  `deny.toml` (plan 013/03/01).

## Any open-source licence for development tools

- **Rule:** an npm package or a machine tool that only builds, lints or tests the code may be under any open-source
  licence, copyleft included (LGPL, MPL-2.0, GPL), as long as it never ships in the app and no copy of it is kept in
  the repository. Non-commercial, source-available (SSPL, BUSL) and unlicensed tools are not taken. Every one is listed
  with its licence in [`../dependencies.md`](../dependencies.md). The user's decision of 2026-11-15.
- **Why:** a copyleft licence binds whoever distributes the licensed code. A tool run on a developer's machine or in CI
  is used, not distributed, and its output (a lint report, a test result) is not a work derived from it, so it does not
  bind the app.
- **Enforced by:** review; the npm licence check above covers only production dependencies, so a development package
  is checked by hand when it is added.

## MPL-2.0 for five crates of Tauri

- **Rule:** MPL-2.0 is allowed only for `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext` and `selectors`,
  which Tauri 2 needs (accepted by the user, 2026-11-15). Their source files are never modified; every build lists them
  in its licence notices. Any other MPL-2.0 crate is asked about first.
- **Why:** MPL-2.0 is a file-level copyleft: unmodified, these crates bind only their own files, not our code; the
  obligation is to name them and where their source is. There is no Tauri without them.
- **Enforced by:** `deny.toml` `[licenses] exceptions` by crate name, mirrored in `about.toml`; the licence notices
  under Settings → About name each with the address of its source, kept up to date by `npm run check:notices` (plan
  013/03/01).

## Exact versions, locked builds

- **Rule:** exact versions in `package.json` (`save-exact=true`), `package-lock.json` and `Cargo.lock` committed;
  hooks and CI run cargo with `--locked` and npm with `npm ci`. The Rust toolchain is pinned in `rust-toolchain.toml`.
- **Why:** a minor update can break a build with no code change.
- **Enforced by:** `.npmrc`, the hooks' and CI's commands (plan 001).

## Few dependencies

- **Rule:** a new dependency is added only when it saves real work; a crate with a large tree for a small task is
  avoided. Unused dependencies are removed.
- **Why:** build time, binary size, supply-chain risk.
- **Enforced by:** cargo-machete and knip in the pre-push hook; cargo-deny `[bans]` (plan 001).
