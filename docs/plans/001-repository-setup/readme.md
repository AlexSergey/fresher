# 001 — Repository setup

**Status:** 🔨 in progress · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo, ci, workspace, app

## Goal

A repository where every commit and push is checked by a complete quality system for Rust and TypeScript, with a
Cargo workspace, a Tauri 2 + React app skeleton and a testing architecture for unit and e2e tests — before any product
code is written. The design: [`00-design.md`](./00-design.md).

Already in place before this plan (created with the docs): `.gitignore`, `.editorconfig`, `.gitattributes`,
`CLAUDE.md`, `docs/`.

## Plans

In this order. 001/01–001/03 were merged one by one under the earlier rule (a branch per plan); from 2026-11-15 the
rest follows [`../../workflow.md`](../../workflow.md): 001/04 and 001/05 finish in one branch `feat/001-repository-setup`
(it takes the commit of `feat/001-04-testing-architecture`) and are squash-merged into `main` once.

| #   | Plan                                                              | Status      | What                                                                             |
| --- | ----------------------------------------------------------------- | ----------- | -------------------------------------------------------------------------------- |
| 01  | [Toolchain and workspace](./01-toolchain-and-workspace/readme.md) | ✔️ done     | root npm tooling, pinned Rust toolchain, Cargo workspace, Tauri + React skeleton |
| 02  | [Linting](./02-linting/readme.md)                                 | ✔️ done     | Rust lints, the prod-forge ESLint config for React, repository guards            |
| 03  | [Git hooks](./03-git-hooks/readme.md)                             | ✔️ done     | husky, commitlint, lint-staged, pre-push                                         |
| 04  | [Testing architecture](./04-testing-architecture/readme.md)       | ✔️ done     | test helpers, fixtures, core e2e, UI unit and e2e, coverage floors               |
| 05  | [CI](./05-ci/readme.md)                                           | ✅ approved | GitHub Actions on macOS, CodeQL, audits                                          |

## Acceptance

- `npm install` on a clean clone installs the hooks; `npm run lint`, `npm test`, `npm run test:cov`,
  `npm run test:e2e` and `npm run tauri dev` work.
- A commit with a formatting error, a Clippy warning, an ESLint error, a type error, a typo, a secret, a 2 MiB file or a
  message that is not a Conventional Commit is rejected; a commit that touches only Markdown runs no Rust or TS check.
- A push under 90 % coverage is rejected.
- CI is green on the plan's pull request.
- [`../../quality.md`](../../quality.md) and [`../../testing.md`](../../testing.md) match what was built; their status
  notes are removed.

## Conclusions

Not written yet.
