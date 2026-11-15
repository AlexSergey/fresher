# 001 — Repository setup

**Status:** 🔨 in progress · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo, ci, workspace, app

> Awaiting the user's review and CI on the pull request: all five plans are implemented and their conclusions written;
> the plan becomes ✔️ done once the user has looked at the result and the workflows have run green on GitHub.

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
| 05  | [CI](./05-ci/readme.md)                                           | ✔️ done     | GitHub Actions on macOS, CodeQL, audits                                          |

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

Implemented 2026-11-15; the user's review and the first CI run are pending.

- **Acceptance, checked on this machine:**
  - `npm install` installs the hooks; `npm run lint`, `npm test`, `npm run test:cov` and `npm run test:e2e` pass;
    `npm run dev` opened the window (001/01, checked by the user); `npm run build` gives an unsigned `Mac Fresher.app`.
  - The rejected commits (formatting, Clippy, ESLint, type error, typo, secret, 2 MiB file, a message that is not a
    Conventional Commit) and the Markdown-only commit that runs no Rust or TS check were probed in 001/02 and 001/03.
  - A push under 90 % coverage is rejected: the probe of 001/04 (37 % without the `fresher-testing` tests).
  - [`../../quality.md`](../../quality.md) and [`../../testing.md`](../../testing.md) describe what was built; their
    status notes are removed.
  - **Not checked: CI on the pull request.** The branch is not pushed; `actionlint` passes locally, but the workflows
    have never run on GitHub.
- **Measured:**
  - Tests: 29 Rust (unit, integration, property, core e2e), 2 UI unit, 6 Playwright (WebKit and Chromium), 7 for the
    repository scripts.
  - Coverage: Rust lines 94.9 %, UI 100 % on every metric; the floors are 90 %.
  - `npm run pre-push`: 15 s with a warm `target/`. From a cold `target/` the two builds dominate: Clippy over the
    workspace about 70 s, the coverage build about 95 s, so about three minutes in all. Agent's call 3 (no pre-push
    selector until the hook takes more than ~2 minutes) holds for the usual warm case.
  - `npm run build`: 57 s from a warm cache, a 5.12 MiB bundle.
- **Learned:**
  - **A broken Xcode stops every Rust test, not only the app build.** 001/04 was written while the Xcode install on
    this machine was broken and test binaries could not be linked without an `SDKROOT` workaround in the shell. After
    Xcode 27.2 was repaired, every check was run again with no workaround and passed unchanged: nothing in the
    repository depends on the workaround. A machine that cannot link should repair Xcode (or its Command Line Tools),
    not set `SDKROOT` in the repository's scripts.
  - `npm install` rewrites `package-lock.json` when a field of `package.json` changes (the licence, here): the lock is
    committed in the same change as `package.json`, or `npm ci` in CI starts from a lock that disagrees with it.
  - The repository needs no platform-specific screenshot names: the snapshots carry only the browser, and both the
    local machine and CI are macOS.
