# 001/05 — CI

**Status:** ✔️ done · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** ci

Branch: `feat/001-repository-setup` (with 001/04). Depends on open question 1 of [`../00-design.md`](../00-design.md).

## Steps

| #   | Step                                           | Status      |
| --- | ---------------------------------------------- | ----------- |
| 01  | [CI workflow](./01-ci-workflow.md)             | ✔️ done     |
| 02  | [CodeQL and audits](./02-codeql-and-audits.md) | ✔️ done     |

## Conclusions

Implemented 2026-11-15.

- `.github/workflows/ci.yml`: jobs `lint` → `test` → `e2e` → `build` on `macos-latest`, as the steps describe;
  `.github/workflows/codeql.yml`: `javascript-typescript` and `rust`, on pull requests, on `main` and weekly.
- Every action is pinned to a full commit SHA with its version in a comment (`actions/checkout` v7.0.1,
  `actions/setup-node` v7.1.0, `actions/upload-artifact` v7.0.2, `Swatinem/rust-cache` v2.9.3,
  `taiki-e/install-action` v2.90.5, `github/codeql-action` v4.39.3); the SHAs were read from the tags on GitHub.
- The cargo tools are installed at the versions used locally (cargo-deny 0.20.2, cargo-machete 0.9.2, cargo-nextest
  0.9.149, cargo-llvm-cov 0.9.1, typos 1.51.1), actionlint 1.7.12 by its own download script at that tag.
- Checked locally: `actionlint` (with shellcheck) passes over the workflows, Prettier and typos accept them, every
  command the jobs run passes on this machine, including `npm audit --omit=dev --audit-level=high` (0 vulnerabilities)
  and `npm run build` (an unsigned `Mac Fresher.app` of 5.12 MiB in 57 s from a warm cache).
- **On GitHub:** the four jobs and CodeQL ran green on the pull request (#1), CodeQL with no alert. The deliberate
  Clippy probe was not run on GitHub; the same `npm run lint` rejects it locally.
- Deviations from the steps:
  - **One composite action, `.github/actions/setup`**, holds the setup every job repeats (Node from `.nvmrc`, the Rust
    toolchain from `rust-toolchain.toml` through `rustup toolchain install --no-self-update`, the caches, `npm ci`).
  - **CodeQL runs on `ubuntu-latest`**: both languages are analysed with `build-mode: none`, nothing is compiled, so a
    macOS runner buys nothing.
  - **nextest runs with its `ci` profile** (`NEXTEST_PROFILE=ci`: no fail-fast, no retries) and its JUnit report is
    kept with the coverage reports.
  - The `test` job also runs `npm run test:scripts`, as the pre-push hook does.
  - `HUSKY=0`: `npm ci` installs no git hooks on the runner.
