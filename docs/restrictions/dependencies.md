# Dependencies

The rules every dependency follows. The list of the dependencies themselves, with their versions and licences, is
[`../dependencies.md`](../dependencies.md); a change that adds, removes or updates one updates it in the same change.

## Permissive licences only

- **Rule:** third-party code only under MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0,
  Unlicense, CC0-1.0. Anything else (MPL-2.0, LGPL, …) is asked about before it is added; GPL, AGPL, SSPL,
  non-commercial and unlicensed code are not taken.
- **Why:** the app may be distributed; copyleft or unclear licences would bind it.
- **Enforced by:** `cargo deny check` (`deny.toml` `[licenses]`) for Rust; a licence check of npm production
  dependencies in CI (plan 001).

## MPL-2.0 for five crates of Tauri

- **Rule:** MPL-2.0 is allowed only for `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext` and `selectors`,
  which Tauri 2 needs (accepted by the user, 2026-11-15). Their source files are never modified; every build lists them
  in its licence notices. Any other MPL-2.0 crate is asked about first.
- **Why:** MPL-2.0 is a file-level copyleft: unmodified, these crates bind only their own files, not our code; the
  obligation is to name them and where their source is. There is no Tauri without them.
- **Enforced by:** `deny.toml` `[licenses] exceptions` by crate name; the notices are due before the first release
  ([`../deferred/licence-notices.md`](../deferred/licence-notices.md)).

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
