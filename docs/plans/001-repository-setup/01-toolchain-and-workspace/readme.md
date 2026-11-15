# 001/01 — Toolchain and workspace

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo, workspace, app

Branch: `feat/001-01-toolchain-and-workspace`.

## Steps

| #   | Step                                                         | Status  |
| --- | ------------------------------------------------------------ | ------- |
| 01  | [Root tooling](./01-root-tooling.md)                         | ✔️ done |
| 02  | [Rust toolchain and workspace](./02-rust-workspace.md)       | ✔️ done |
| 03  | [Tauri shell and UI skeleton](./03-tauri-and-ui-skeleton.md) | ✔️ done |

## Conclusions

Implemented 2026-11-15.

- `npm run dev` opens the window with the version returned by the Rust command `app_version` (checked by the user);
  `cargo build --workspace` and `npm run build --workspace app/ui` pass; `prettier --check`, `taplo fmt --check` and
  `typos` pass.
- Versions pinned: Rust 1.99.1, Tauri 2.12.5 (`tauri-build` 2.7.4, CLI and `@tauri-apps/api` 2.12.4), React 19.3.1,
  Vite 8.3.8, TypeScript 6.0.3.
- Deviations from the steps:
  - **TypeScript 6.0.3, not the latest 7.0.2** — the same as prod-forge; see [`../../../hacks/README.md`](../../../hacks/README.md).
  - **No dependencies between the skeleton crates yet**: an empty crate that declares one fails cargo-machete (plan 02).
    Each dependency of [`../../../architecture/README.md`](../../../architecture/README.md) is added with the first code
    that uses it (plan 04).
  - The app icon is generated from the user's logo `assets/logo.png` by `tauri icon assets/logo.png -o
app/tauri/icons`; only the macOS sizes are kept.
  - `devCsp` is `null`: Vite's dev server needs inline scripts and a WebSocket; the production CSP stays strict.
- Prettier formats `docs/` without breaking anything (it aligns tables, writes `_` for emphasis): agent's call 6 holds.
