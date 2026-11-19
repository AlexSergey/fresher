# Dependencies

Every dependency of the repository, its version and its licence: what the app ships, what only builds and checks it,
and the tools a machine needs. The rules they must follow are in
[`restrictions/dependencies.md`](./restrictions/dependencies.md); this page is the inventory.

A change that adds, removes or updates a dependency updates this page in the same change. Versions are exact
(`Cargo.toml`, `package.json`); the lock files hold the full trees. Checked against `Cargo.lock`,
`package-lock.json` and `cargo metadata` on 2026-11-16 (plan 008).

## Shipped with the app

### Rust crates

Direct dependencies of the workspace. All are taken from `[workspace.dependencies]` except the Tauri crates, which only
`fresher-app` uses. "Used by" names our crates without the `fresher-` prefix.

| Crate                          | Version | Licence                   | Used by                                                                             |
| ------------------------------ | ------- | ------------------------- | ----------------------------------------------------------------------------------- |
| `crc32fast`                    | 1.5.5   | MIT OR Apache-2.0         | store                                                                               |
| `libc`                         | 0.2.191 | MIT OR Apache-2.0         | macos, scanner, testing                                                             |
| `miniz_oxide`                  | 0.9.1   | MIT OR Zlib OR Apache-2.0 | scanner, store                                                                      |
| `objc2`                        | 0.6.5   | MIT                       | macos                                                                               |
| `objc2-foundation`             | 0.3.2   | MIT                       | macos                                                                               |
| `serde`                        | 1.0.229 | MIT OR Apache-2.0         | core, e2e, module-app-leftovers, module-dev-caches, module-downloads, module-jetbrains, rules, sandbox, store, testing |
| `serde_json`                   | 1.0.154 | MIT OR Apache-2.0         | app, cleaner, core, e2e, engine, module-app-leftovers, module-demo, module-dev-caches, module-downloads, module-jetbrains, module-temp, store, testing |
| `tauri`                        | 2.12.5  | Apache-2.0 OR MIT         | app                                                                                 |
| `tauri-plugin-dialog`          | 2.8.4   | Apache-2.0 OR MIT         | app (the folder picker, from Rust only; brings `rfd` and `tauri-plugin-fs`)        |
| `tauri-plugin-single-instance` | 2.5.5   | Apache-2.0 OR MIT         | app                                                                                 |
| `tempfile`                     | 3.27.0  | MIT OR Apache-2.0         | app, cleaner, e2e, macos, sandbox, scanner, store, system, testing                  |
| `thiserror`                    | 2.0.21  | MIT OR Apache-2.0         | app, cleaner, core, engine, macos, module-app-leftovers, module-demo, module-dev-caches, module-downloads, module-jetbrains, module-temp, module-trial, rules, sandbox, scanner, store, system, testing |
| `toml`                         | 1.2.1   | MIT OR Apache-2.0         | module-app-leftovers, module-dev-caches, module-downloads, module-jetbrains, rules, sandbox, testing |
| `tracing`                      | 0.1.44  | MIT                       | app, engine, macos, module-downloads, scanner, store, system                        |
| `ts-rs`                        | 12.0.1  | MIT                       | app, core                                                                           |

The full tree is `Cargo.lock`: 510 packages for every platform, our own crates included. Every licence in it is on the
allowlist, or is a choice (`A OR B`) that includes an allowed one, except the MPL-2.0 crates that Tauri needs:
`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext` and `selectors`, accepted by the user on 2026-11-15
([`restrictions/dependencies.md`](./restrictions/dependencies.md), "MPL-2.0 for five crates of Tauri"). `cargo deny
check` checks the whole tree on every push.

### npm packages

The production dependencies of `app/ui`, bundled into the UI.

| Package           | Version | Licence           |
| ----------------- | ------- | ----------------- |
| `@tauri-apps/api` | 2.12.4  | Apache-2.0 OR MIT |
| `lucide-react`    | 1.70.0  | ISC               |
| `react`           | 19.3.1  | MIT               |
| `react-dom`       | 19.3.1  | MIT               |

Their only transitive package is `scheduler` 0.28.0 (MIT). No script checks npm licences yet
([`known-issues/repo-no-npm-licence-check.md`](./known-issues/repo-no-npm-licence-check.md)).

## Build and checks only

Never shipped: they build, test and check the code. `fresher-e2e` and `fresher-testing` are our own test crates;
`fresher-testing` and `fresher-sandbox` are also normal dependencies of `fresher-app`, used by its sandbox mode in
debug builds only (`docs/hacks/README.md`, "The demo module linked in release builds"). As normal dependencies they
bring their own (`tempfile`, `toml`, `libc`, `serde_json`) into the release build's dependency graph, though the code
that uses them is compiled out of it.

### Rust crates

| Crate         | Version | Licence           | Used by                                                                       |
| ------------- | ------- | ----------------- | ----------------------------------------------------------------------------- |
| `criterion`   | 0.8.2   | Apache-2.0 OR MIT | the benches of `scanner` (default features off)                               |
| `proptest`    | 1.12.0  | MIT OR Apache-2.0 | tests of `cleaner`, `core`, `engine`, `macos`, `module-app-leftovers`, `module-dev-caches`, `module-downloads`, `module-jetbrains`, `module-temp`, `rules`, `scanner`, `store` and `system` |
| `tauri-build` | 2.7.4   | Apache-2.0 OR MIT | the build script of `fresher-app`                                             |
| `trybuild`    | 1.0.125 | MIT OR Apache-2.0 | the compile-fail tests of `cleaner` and `core`                                |

### npm packages: the UI (`app/ui`)

| Package                     | Version | Licence    |
| --------------------------- | ------- | ---------- |
| `@axe-core/playwright`      | 4.14.0  | MPL-2.0    |
| `@playwright/test`          | 1.65.1  | Apache-2.0 |
| `@testing-library/dom`      | 10.4.2  | MIT        |
| `@testing-library/jest-dom` | 7.0.1   | MIT        |
| `@testing-library/react`    | 16.3.3  | MIT        |
| `@types/react`              | 19.3.0  | MIT        |
| `@types/react-dom`          | 19.3.0  | MIT        |
| `@vitejs/plugin-react`      | 6.3.0   | MIT        |
| `@vitest/coverage-v8`       | 5.1.0   | MIT        |
| `happy-dom`                 | 21.0.4  | MIT        |
| `typescript`                | 6.0.3   | Apache-2.0 |
| `vite`                      | 8.3.8   | MIT        |
| `vitest`                    | 5.1.0   | MIT        |

### npm packages: the repository (root)

| Package                           | Version | Licence           |
| --------------------------------- | ------- | ----------------- |
| `@commitlint/cli`                 | 21.2.4  | MIT               |
| `@commitlint/config-conventional` | 21.2.4  | MIT               |
| `@eslint-react/eslint-plugin`     | 5.30.2  | MIT               |
| `@eslint/js`                      | 10.0.1  | MIT               |
| `@eslint/json`                    | 2.1.1   | Apache-2.0        |
| `@tauri-apps/cli`                 | 2.12.4  | Apache-2.0 OR MIT |
| `@types/node`                     | 26.6.6  | MIT               |
| `@vitest/eslint-plugin`           | 1.6.28  | MIT               |
| `eslint`                          | 10.15.0 | MIT               |
| `eslint-config-flat-gitignore`    | 2.4.0   | MIT               |
| `eslint-config-prettier`          | 10.1.8  | MIT               |
| `eslint-plugin-check-file`        | 3.3.2   | Apache-2.0        |
| `eslint-plugin-import-lite`       | 0.6.0   | MIT               |
| `eslint-plugin-jsx-a11y-x`        | 0.2.0   | MIT               |
| `eslint-plugin-no-only-tests`     | 3.4.0   | MIT               |
| `eslint-plugin-package-json`      | 1.14.0  | MIT               |
| `eslint-plugin-perfectionist`     | 5.13.0  | MIT               |
| `eslint-plugin-playwright`        | 2.12.2  | MIT               |
| `eslint-plugin-prettier`          | 5.5.7   | MIT               |
| `eslint-plugin-regexp`            | 3.3.3   | MIT               |
| `eslint-plugin-sonarjs`           | 4.2.2   | LGPL-3.0-only     |
| `eslint-plugin-testing-library`   | 7.16.2  | MIT               |
| `eslint-plugin-unicorn`           | 79.0.0  | MIT               |
| `globals`                         | 17.14.0 | MIT               |
| `husky`                           | 9.1.7   | MIT               |
| `knip`                            | 6.45.1  | ISC               |
| `lint-staged`                     | 17.6.0  | MIT               |
| `prettier`                        | 3.10.1  | MIT               |
| `typescript`                      | 6.0.3   | Apache-2.0        |
| `typescript-eslint`               | 8.72.2  | MIT               |

### Licences outside the allowlist, in development packages

The installed npm tree (385 packages) is mostly MIT, Apache-2.0, ISC and BSD. These are not on the allowlist; none of
them ships in the app, so they are allowed as development tools
([`restrictions/dependencies.md`](./restrictions/dependencies.md), "Any open-source licence for development tools"):

| Package                                        | Licence       | Pulled in by                                       |
| ---------------------------------------------- | ------------- | -------------------------------------------------- |
| `eslint-plugin-sonarjs`                        | LGPL-3.0-only | direct                                             |
| `@sonarsource/analyzer-commons-configurations` | LGPL-3.0-only | `eslint-plugin-sonarjs`                            |
| `@axe-core/playwright`                         | MPL-2.0       | direct (the accessibility specs)                   |
| `axe-core`                                     | MPL-2.0       | `eslint-plugin-jsx-a11y-x`, `@axe-core/playwright` |
| `lightningcss`, `lightningcss-darwin-arm64`    | MPL-2.0       | `vite` (CSS transforms at build)                   |
| `minimatch`, `@isaacs/nonrecursive-glob-match` | BlueOak-1.0.0 | the lint tooling                                   |
| `argparse`                                     | Python-2.0    | the lint tooling                                   |
| `caniuse-lite`                                 | CC-BY-4.0     | the build tooling (browser data)                   |
| `spdx-exceptions`                              | CC-BY-3.0     | the lint tooling (licence data)                    |

## Tools on the machine

Installed once per machine ([`quality.md`](./quality.md), "Setting up a machine"); not part of any lock file.

| Tool             | Version  | Licence           | What for                                              |
| ---------------- | -------- | ----------------- | ----------------------------------------------------- |
| Rust toolchain   | pinned   | MIT OR Apache-2.0 | `rust-toolchain.toml`, with `rustfmt`, `clippy`, llvm |
| Node             | `.nvmrc` | MIT               | the UI and the repository scripts                     |
| `cargo-deny`     | 0.20.2   | MIT OR Apache-2.0 | licences, advisories, bans                            |
| `cargo-machete`  | 0.9.2    | MIT               | unused Rust dependencies                              |
| `cargo-nextest`  | 0.9.149  | Apache-2.0 OR MIT | the Rust test runner                                  |
| `cargo-llvm-cov` | 0.9.1    | Apache-2.0 OR MIT | Rust coverage                                         |
| `gitleaks`       | 8.30.1   | MIT               | secrets in staged changes                             |
| `typos-cli`      | 1.52.2   | Apache-2.0 OR MIT | spelling                                              |
| `taplo`          | 0.10.0   | MIT               | TOML formatting                                       |
| `actionlint`     | 1.7.12   | MIT               | GitHub Actions workflows                              |
