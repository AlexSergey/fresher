# Dependencies

## Permissive licences only

- **Rule:** third-party code only under MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0,
  Unlicense, CC0-1.0. Anything else (MPL-2.0, LGPL, …) is asked about before it is added; GPL, AGPL, SSPL,
  non-commercial and unlicensed code are not taken.
- **Why:** the app may be distributed; copyleft or unclear licences would bind it.
- **Enforced by:** `cargo deny check` (`deny.toml` `[licenses]`) for Rust; a licence check of npm production
  dependencies in CI (plan 001).

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
