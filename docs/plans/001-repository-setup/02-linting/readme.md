# 001/02 — Linting

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo, workspace, app

Branch: `feat/001-02-linting`. Layers 1–2 of the quality system ([`../00-design.md`](../00-design.md)).

## Steps

| #   | Step                                           | Status  |
| --- | ---------------------------------------------- | ------- |
| 01  | [Rust lints](./01-rust-lints.md)               | ✔️ done |
| 02  | [TypeScript lints](./02-typescript-lints.md)   | ✔️ done |
| 03  | [Repository guards](./03-repository-guards.md) | ✔️ done |

## Conclusions

Implemented 2026-11-15.

- `npm run lint` (rustfmt check, Clippy with `-D warnings`, `tsc` for the tooling and the UI, ESLint), `npm run knip`,
  `cargo deny check`, `cargo machete` and `npm run test:scripts` pass on the skeleton.
- Probes: a Rust file with `unwrap()`, `println!`, a bare `#[allow]` and `std::fs::remove_file` gives exactly those four
  errors; a React component with `any`, a default export, unsorted imports, a missing effect dependency and a click
  handler on a `div` is reported by each rule; a staged 2 MiB file, a `.sqlite` file, an AWS key and a misspelled word
  each fail `check:staged`, `check:secrets` or `check:typos`.
- `unsafe_code = "forbid"` holds for the Tauri shell too: its macros need no unsafe code in our crate.
- Deviations from the steps:
  - **MPL-2.0** for five crates Tauri needs (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`,
    `selectors`): asked and accepted by the user, recorded in `docs/restrictions/dependencies.md`; the licence notices
    are deferred to the first release (`docs/deferred/licence-notices.md`).
  - cargo-deny checks the macOS targets only (`[graph] targets`); six duplicate crates of Tauri's tree are warnings.
  - **React plugins:** `@eslint-react/eslint-plugin` and `eslint-plugin-jsx-a11y-x` instead of the classic ones, which
    do not support ESLint 10; see `docs/hacks/`.
  - typescript-eslint uses the project service (`projectService: true`) instead of a separate `tsconfig.eslint.json`:
    the root `tsconfig.json` covers the tooling files, `app/ui/tsconfig.json` the UI.
  - `disallowed-methods` lists the `std::fs` functions only: `tokio` is not a dependency.
  - The size-limit exception for `app/tauri/icons/icon.icns` (see `docs/hacks/`).
