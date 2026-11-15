# 001 — Repository setup: design

**Status:** ✅ approved · **Version:** 0.0.1 · **Date:** 2026-11-15 · **Modules:** repo, ci, workspace, app

## Intent

The user's task (translated): "Set up the repository for a fast Rust desktop app for macOS that cleans the disk
smartly. Create `.gitignore` and the other basics; build the best lint system for Rust using best practices; connect
lint-staged so only the changed parts are checked; prepare the architecture for unit and e2e testing; connect
commitlint and husky. Take anything from `prod-forge` for the quality gates — everything there is set up very well.
Write it all down as the first plan for the repository setup."

## Goals

1. A Cargo workspace and a Tauri 2 + React + TypeScript app skeleton that builds, runs and is tested — with no product
   logic yet.
2. The six-layer quality defence of `prod-forge/backend` (`guide/code-quality.md`), with the Rust equivalent of every
   layer: formatting, static analysis, pre-commit, commit messages, pre-push and CI, dependency locking.
3. lint-staged runs only on staged files; whole-project checks (Clippy, `tsc`) run once when a matching file is staged.
4. A testing architecture for unit, integration, property, golden, core e2e, UI e2e and visual tests, with fixture
   trees that never touch the real home folder, and coverage floors of 90 %.
5. Every tool and command documented in [`../../quality.md`](../../quality.md) and [`../../testing.md`](../../testing.md).

## Non-goals

- Any product logic: no scanner, rules, store or cleaner behaviour beyond a compiling skeleton with one test each.
- Code signing, notarization, the release process and auto-update (see the open questions).
- The UI design system (Dark Premium tokens, charts) — a plan of its own.

## Design

### Repository layout

```
mac-fresher/
├── Cargo.toml              workspace: members, [workspace.package], [workspace.lints], profiles
├── Cargo.lock
├── rust-toolchain.toml     1.99.1, clippy, rustfmt, llvm-tools-preview
├── rustfmt.toml  clippy.toml  deny.toml  .config/nextest.toml  .taplo.toml  _typos.toml
├── package.json            root tooling + npm workspace "app/ui"; scripts for every check
├── package-lock.json  .npmrc  .nvmrc  .prettierrc  .prettierignore
├── eslint.config.ts  knip.json  .commitlintrc.cjs  .lintstagedrc.cjs
├── .husky/                 pre-commit, commit-msg, pre-push
├── .github/workflows/      ci.yml, codeql.yml
├── crates/                 core, scanner, rules, store, cleaner, testing, e2e
├── app/
│   ├── tauri/              fresher-app: Tauri 2 shell (Cargo workspace member)
│   └── ui/                 React + TypeScript + Vite; Vitest; Playwright in e2e/
├── fixtures/               homes/ (fixture trees as data), golden/ (expected reports)
├── scripts/                check-staged.ts (repository guard)
└── docs/
```

One Cargo workspace for the core and the shell; one npm workspace at the root for the tooling and the UI. The crates
are described in [`../../architecture/README.md`](../../architecture/README.md).

### The quality layers

| Layer (prod-forge)          | TypeScript (taken from prod-forge)                                         | Rust (added)                                                          |
| --------------------------- | -------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| 1. Formatting               | EditorConfig, Prettier (same `.prettierrc`)                                | rustfmt, taplo                                                        |
| 2. Static analysis          | the prod-forge `eslint.config.ts`, adapted for React and Vitest; `tsc`; knip | Clippy `all` + `pedantic` + restriction picks; compiler lints; cargo-deny; cargo-machete |
| 3. Pre-commit               | husky + lint-staged                                                        | the same lint-staged config, Rust entries                             |
| 4. Commit messages          | commitlint + the AI-attribution guard                                      | —                                                                     |
| 5. Pre-push + CI            | knip, tests with coverage; CI: lint, tests, e2e, audit, CodeQL             | machete, deny, nextest + llvm-cov; CI on macOS runners                |
| 6. Dependency locking       | exact versions, `save-exact`, `npm ci`                                     | `Cargo.lock`, `--locked`, pinned toolchain                            |
| + Repository guards (bg)    | gitleaks, typos, staged-files guard                                        | the same                                                              |

ESLint changes from prod-forge's config: drop `eslint-plugin-jest` for `@vitest/eslint-plugin`; add
`eslint-plugin-react`, `eslint-plugin-react-hooks`, `eslint-plugin-jsx-a11y`, `eslint-plugin-testing-library` (unit
tests) and `eslint-plugin-playwright` (`app/ui/e2e/`); kebab-case file names apply to `app/ui/src/` (React components
included); globals are browser-only in `app/ui/src/`, Node in configs and `scripts/`.

Rust lints (workspace `Cargo.toml`), on top of `bg/engine`'s set:

- `[workspace.lints.rust]`: `unsafe_code = "forbid"` (the shell crate overrides to `deny` only if Tauri macros need
  it), `missing_debug_implementations`, `rust_2018_idioms`, `unreachable_pub`, `unused_qualifications`,
  `unused_must_use = "deny"`, `unsafe_op_in_unsafe_fn = "deny"`.
- `[workspace.lints.clippy]`: `all`, `pedantic`; restriction picks `allow_attributes_without_reason`, `dbg_macro`,
  `exit`, `expect_used`, `unwrap_used`, `panic`, `todo`, `unimplemented`, `print_stdout`, `print_stderr`, `mem_forget`,
  `rc_mutex`, `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`, `float_cmp_const`, `lossy_float_literal`,
  plus for this product `string_slice` (names are UTF-8: no byte slicing), `fn_to_numeric_cast_any`,
  `large_include_file`, `semicolon_outside_block`, `clone_on_ref_ptr`, `rest_pat_in_fully_bound_structs`. Size
  arithmetic is covered by pedantic `cast_*` lints and by `checked_*`/`saturating_*` calls in the code.
- `clippy.toml`: tests may unwrap/expect/print; `disallowed-methods` lists `std::fs::remove_file`,
  `std::fs::remove_dir`, `std::fs::remove_dir_all` (and the `tokio::fs` twins) — the cleaner crate allows them with
  `#[expect(clippy::disallowed_methods, reason = "...")]` at its single deletion point.
- `deny.toml`: the licence allowlist of [`../../restrictions/dependencies.md`](../../restrictions/dependencies.md),
  advisories deny, `multiple-versions = "warn"`, wildcards deny, unknown registries and git sources deny; HTTP client
  crates (`reqwest`, `hyper`, `ureq`) banned for the core crates ([`../../restrictions/privacy.md`](../../restrictions/privacy.md)).

### lint-staged

```js
module.exports = {
  '*.rs': ['rustfmt --edition 2024', () => 'cargo clippy --workspace --all-targets --locked -- -D warnings'],
  '*.{ts,tsx}': ['prettier --write', 'eslint --fix', () => 'npm run lint:ts'],
  '*.json': ['prettier --write', 'eslint --fix'],
  '*.{md,yml,yaml,css}': ['prettier --write'],
  '*.toml': ['taplo fmt'],
  '{Cargo.toml,Cargo.lock,deny.toml}': [() => ['cargo deny check', 'cargo machete']],
};
```

A function entry ignores the file list, so a whole-project check runs once per commit, only when a matching file is
staged.

### Testing architecture

As [`../../testing.md`](../../testing.md) describes. The key pieces this plan creates:

- `fresher-testing`: `FixtureTree` builds a fake home from a TOML description into a `tempfile::TempDir`, with sparse
  files so a declared size costs no disk; a `FakeClock`; traits for the Trash and external commands with fakes.
- `fresher-e2e`: a tests-only crate with one smoke e2e (build the fixture home `fixtures/homes/empty.toml`, run the
  skeleton pipeline, compare with `fixtures/golden/empty.json`).
- `cargo-nextest` profile in `.config/nextest.toml` (`ci` profile: retries 0, fail-fast off, JUnit output).
- `cargo llvm-cov nextest --workspace --fail-under-lines 90`; the shell crate's `main.rs` excluded from coverage.
- UI: Vitest + Testing Library + happy-dom with a v8 coverage floor of 90 % (four metrics); Playwright in
  `app/ui/e2e/` against `vite preview` with `@tauri-apps/api/mocks` `mockIPC`, one smoke test and one screenshot test.

### CI

`.github/workflows/ci.yml` on `macos-latest` (the app is macOS-only): job `lint` (fmt check, Clippy, `tsc`, ESLint,
knip, machete, deny, typos, `npm audit --omit=dev --audit-level=high`), job `test` (coverage floors), job `e2e`
(`crates/e2e`, Playwright), job `build` (`tauri build` without signing). Cargo and npm caches. `codeql.yml` for
`javascript-typescript` and `rust`, weekly and on pull requests. Actions pinned by version.

## Open questions

1. **CI now or later?** macOS runners are billed at a higher rate on private repositories. Option: keep the workflows
   in this plan (recommended — the same checks as the hooks) or defer plan 05 to `docs/deferred/`.
2. **Release process.** prod-forge uses release-it with a conventional changelog. For a desktop app a release also
   needs signing and notarization (an Apple Developer account). Take release-it now without signing, or defer the whole
   release process to its own plan?
3. **Repository visibility and licence.** Public open source (which licence?) or private? It changes the CI cost and
   whether `LICENSE` and `CONTRIBUTING.md` are created now.
4. **Minimum macOS version.** Tauri 2 supports macOS 10.15+; a higher floor (for example macOS 13) allows newer APIs.

## Decisions

- 2026-11-15 — The plan is approved ("commit this and move on to implementing plan 001"). The open questions were not
  answered; the agent takes the defaults below (agent's calls 10–13) until the user overrules them.
- 2026-11-15 — UI stack: Tauri 2 with a React + TypeScript UI over the Rust core (the user chose it over pure-Rust UI
  toolkits).
- 2026-11-15 — Everything in the repository is in English; the user's words are translated when recorded.
- 2026-11-15 — The quality gates follow `prod-forge/backend`; anything from it may be taken.

## Agent's calls (to check)

1. **Crate names and prefix** `fresher-*`, folders `crates/<name>`; the shell at `app/tauri`, the UI at `app/ui`.
2. **typos over cspell** for spell checking: one fast tool for Rust, TypeScript and Markdown (prod-forge uses cspell,
   bg uses typos).
3. **No pre-push selector** like bg's: the project is small, the pre-push runs everything. Revisit when it takes more
   than ~2 minutes.
4. **E2E not in the pre-push hook** (they run in CI and by hand), as in bg; unit tests with coverage are.
5. **Coverage floor 90 %** for both Rust and UI (bg's bar, higher than prod-forge's 80 %).
6. **Prettier does format `docs/`** (unlike bg, where it broke Markdown); if it breaks tables, `docs/` goes to
   `.prettierignore`.
7. **Four danger levels** — Safe, Low, Medium, High — as defined in [`../../categories/README.md`](../../categories/README.md).
8. **`disallowed-methods` for deletion** outside the cleaner crate, so the "scanning is read-only" restriction is
   enforced by the linter, not only by review.
9. **Node 24 LTS** (installed locally: 24.15.0) and **Rust 1.99.1** (installed locally) as the pinned versions.
10. **Open question 1 → CI is built** (plan 05), on macOS runners.
11. **Open question 2 → the release process is not part of this plan**: no release-it, no signing; a plan of its own.
12. **Open question 3 → private, `UNLICENSED`** until the user decides; no `LICENSE` or `CONTRIBUTING.md` yet.
13. **Open question 4 → minimum macOS 13.0** (Ventura): `minimumSystemVersion` in `tauri.conf.json`.
