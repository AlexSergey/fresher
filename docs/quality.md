# Code quality tooling

What checks the code, when, and why. Configs live next to the code they check; this page explains them.

Set up by plan [001](./plans/001-repository-setup/readme.md).

The system is the six-layer defence of `prod-forge/backend` (its `guide/code-quality.md`), extended with the Rust
equivalent of every TypeScript tool.

## Layer 1. Formatting and consistency

| Tool                           | Checks                                   | Command                   |
| ------------------------------ | ---------------------------------------- | ------------------------- |
| EditorConfig (`.editorconfig`) | indentation, line endings, final newline | the editor                |
| rustfmt (`rustfmt.toml`)       | Rust formatting                          | `cargo fmt --all`         |
| Prettier (`.prettierrc`)       | TS, TSX, JSON, YAML, CSS, Markdown outside `docs/` | `npm run format:prettier` |
| taplo (`.taplo.toml`)          | TOML formatting                          | `taplo fmt`               |

**`docs/` is excluded from Prettier** (`.prettierignore`; the user, 2026-11-15): the documents keep their style by hand
— 120 columns, aligned tables — and neither the hooks nor `npm run format` touch them.

## Layer 2. Static analysis

### Rust

| Tool                                         | What it is                                                          | Command                                                          |
| -------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------- |
| `rust-toolchain.toml`                        | pins the compiler for everyone, with `rustfmt`, `clippy`, llvm      | installed by `rustup` on first use                               |
| compiler lints (`[workspace.lints.rust]`)    | stricter compiler warnings; `unsafe_code = "deny"` (see below)      | part of every build                                              |
| Clippy (`clippy.toml` + `[workspace.lints]`) | `all` + `pedantic` + picked restriction lints                       | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| cargo-deny (`deny.toml`)                     | licences, security advisories, duplicate and banned crates, sources | `cargo deny check`                                               |
| cargo-machete                                | dependencies a crate declares and never uses                        | `cargo machete`                                                  |

What the Rust lints enforce:

- **Clippy `all` + `pedantic`**; the pedantic lints that only add noise are allowed one by one in `Cargo.toml`.
- **No `unwrap()` / `expect()` / `panic!` / `todo!` / `unimplemented!` / `dbg!` / `println!` / `eprintln!`** outside
  tests: errors are returned, output goes through `tracing`.
- **`unsafe` only in `fresher-macos`'s `ffi` module**: `unsafe_code = "deny"` in the workspace;
  `#![forbid(unsafe_code)]` at the root of every crate but `fresher-macos`, checked by the crate guard
  (`npm run check:crate-deps`); in `fresher-macos` only `mod ffi` carries `#[expect(unsafe_code, reason = "...")]`, with
  one unsafe operation per block and a `// SAFETY:` comment (Clippy's `multiple_unsafe_ops_per_block` and
  `undocumented_unsafe_blocks`).
- **`#[allow(...)]` needs a reason**: `#[expect(lint, reason = "...")]`.
- **File system hygiene**: `std::fs::remove_dir_all`, `remove_dir`, `remove_file` and `fresher-macos`'s `remove_at`
  are disallowed by `clippy.toml` (`disallowed-methods`) everywhere except the cleaner's single deletion point, so
  nothing else can delete (`crates/cleaner/tests/deletion_point.rs`).
- **The home folder is looked up once**: `std::env::home_dir` and Tauri's `PathResolver::home_dir` are disallowed by
  `clippy.toml`; the app's setup (`app/tauri/src/lib.rs`) and the sandbox generator's binary
  (`crates/sandbox/src/main.rs`) each carry one `#[expect]`, so a test or a module cannot reach the real home. They are
  the only exception to the single deletion point's guard (`HOME_LOOKUPS` in `crates/cleaner/tests/deletion_point.rs`):
  the expectation must give a reason starting "the home folder is looked up once" and sit right above a
  `let home = ...home_dir()...;` statement.
- **Release profile**: thin LTO, one codegen unit, stripped symbols.

### TypeScript (the UI)

`eslint.config.ts` is the `prod-forge/backend` config (itself `@rockpack/codestyle`), adapted for React and Vitest:

- `typescript-eslint` `strictTypeChecked` + `stylisticTypeChecked`
- `@eslint-react/eslint-plugin` (`recommended-type-checked`, the rules of hooks included) and `eslint-plugin-jsx-a11y-x`
  — React correctness and accessibility (ESLint 10 compatible; see [`hacks/`](./hacks/README.md))
- `eslint-plugin-import-lite` — no default exports (config files excepted)
- `eslint-plugin-perfectionist` — sorted imports and object keys
- `eslint-plugin-sonarjs` — bug and code smell rules, cognitive complexity ≤ 20
- `eslint-plugin-unicorn` — picked best-practice rules
- `eslint-plugin-regexp` — regular expressions
- `eslint-plugin-check-file` — kebab-case file and folder names
- `eslint-plugin-no-only-tests`, `@vitest/eslint-plugin`, `eslint-plugin-testing-library`, `eslint-plugin-playwright`
  — test files
- `eslint-plugin-package-json`, `@eslint/json` — `package.json` and JSON files
- `eslint-plugin-prettier` — Prettier as a lint rule
- `@typescript-eslint/no-restricted-imports` — the UI's import boundaries, one config per folder: only `ipc/` talks to
  the Tauri API, only `design/icons/` imports `lucide-react`, `design/` knows nothing of the product, `product/` takes
  its data by props and only types from the IPC bindings, the gallery never reaches the state or the core
  ([`architecture/`](./architecture/README.md), "UI")
- `tsc --noEmit` — the type check
- knip (`knip.json`) — unused files, exports and dependencies

The design system has guards of its own, unit tests that run with the UI's tests (`npm run test:cov`, so in the
pre-push hook and CI): the **CSS guard** (`app/ui/src/test/css-guard.test.ts`) reads every stylesheet of `src/` and
fails on a feature newer than Safari 16.0 (`color-mix()`, `@property`, nesting, `linear()`, `@starting-style`, …), on
`backdrop-filter` without its `-webkit-` twin, and on a transition or keyframe of a property outside the motion budget
(compositor-only, plus the named paint-only exceptions); the **contrast test**
(`app/ui/src/design/tokens/contrast.test.ts`) holds every text and level pair of the tokens to WCAG AA; the **motion
tokens' mirror** keeps `motion.css` and `motion.ts` equal ([`testing.md`](./testing.md)).

### Whole repository

| Tool                  | What it stops                                      | Command                 |
| --------------------- | -------------------------------------------------- | ----------------------- |
| gitleaks              | keys, tokens and passwords in staged changes       | `npm run check:secrets` |
| typos (`_typos.toml`) | misspellings in code and docs (Rust, TS, Markdown) | `npm run check:typos`   |
| staged-files guard    | any file over 1 MiB, any `*.sqlite` or `.env`      | `npm run check:staged`  |
| crate dependency guard | an edge between workspace crates outside the dependency table (`scripts/check-crate-deps.ts`); a module depending on another module; a crate root other than `fresher-macos`'s without `#![forbid(unsafe_code)]` among its leading attributes (a commented-out one does not count); a file of `modules/*/src` naming `std::process` or `std::env`, aliasing `std`, or using `extern crate`, `include!` or `#[path]` (a module runs a tool through `CommandRunner`); a module with a build script or a third-party crate outside `serde`, `serde_json`, `thiserror`, `toml`, `tracing` (its tests may use any); a `Command::new` or command-port `.run` in `crates/*/src` or `modules/*/src` (tests left out) whose program is a bare name, which the system would look up in `PATH` | `npm run check:crate-deps` |
| lock check | a `package-lock.json` package resolved from outside `https://registry.npmjs.org/` or without a `sha512` integrity, a local package that is not a workspace; a `Cargo.lock` crate from outside crates.io or without a checksum (`scripts/check-locks.ts`) | `npm run check:locks` |
| version check | a place that holds another version than the workspace's `Cargo.toml`: both `package.json`, `app/tauri/tauri.conf.json`, the workspace's entries of `package-lock.json` and `Cargo.lock`; with a version given (the release workflow), also a missing changelog section or release notes (`scripts/release-version.ts`) | `npm run check:version` |
| text check | an exclamation mark, a banned word, a text over its length, a category name of more than 4 words, a line with a full stop, a category without its six texts, a label without a description, "delete" outside a permanent delete, a lower-case "trash", a badly formed or unfinished text, in every English dictionary (`scripts/check-texts.ts`, [`writing.md`](./writing.md)) | `npm run check:texts` |

## Layer 3. Pre-commit (Husky + lint-staged)

`.husky/pre-commit` runs the repository guards (`npm run check:repo`), then lint-staged (`.lintstagedrc.cjs`), which
runs only on the staged files:

| Staged files                            | Runs                                                                        |
| --------------------------------------- | --------------------------------------------------------------------------- |
| `*.rs`                                  | `rustfmt` on the files, then Clippy once over the workspace (`-D warnings`) |
| `*.{ts,tsx}` but `*.generated.ts`       | `prettier --write`, `eslint --fix` on the files, then `tsc --noEmit` once   |
| `*.{js,cjs,mjs}`, `*.json`              | `prettier --write`, `eslint --fix`                                          |
| `*.{md,yml,yaml,css,html}`              | `prettier --write`                                                          |
| `*.toml`                                | `taplo fmt`                                                                 |
| `Cargo.toml`, `Cargo.lock`, `deny.toml` | `cargo deny check` and `cargo machete`                                      |

Clippy and `tsc` are whole-project checks by nature: lint-staged runs them once when a matching file is staged, not once
per file.

## Layer 4. Commit messages (commitlint)

`.husky/commit-msg` rejects AI attribution trailers (`Co-Authored-By: Claude`, …), then commitlint enforces
Conventional Commits: `type(scope): subject`.

- Types: `ci`, `chore`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`.
- Scopes (optional): `core`, `engine`, `modules`, `rules`, `scanner`, `store`, `cleaner`, `macos`, `testing`, `app`,
  `ui`, `e2e`, `deps`, `docs`, `repo`, `ci`.

## Layer 5. Pre-push and CI

`.husky/pre-push` runs `npm run pre-push`:

1. `npm run lint` — `cargo fmt --check`, Clippy, `tsc`, ESLint; then `npm run check:readme` (the README's generated
   table, links, images and claims; `npm run readme:generate` rewrites the table), `npm run check:notices` (the
   licence notices the app ships are up to date and every Rust and npm production licence is in `deny.toml`'s
   allowlist; `npm run notices:generate` rewrites them), `npm run check:crate-deps`, `npm run check:locks`,
   `npm run check:version` and `npm run check:texts`
2. `npm run knip` and `cargo machete`
3. `cargo deny check`
4. `npm run test:scripts` — the repository scripts' tests
5. `npm run test:cov` — Rust tests with coverage (`cargo llvm-cov nextest`, lines ≥ 90 %) and the UI unit tests with
   coverage (Vitest, ≥ 90 % on every metric)

The e2e suites (`npm run test:e2e`) are run by hand and in CI, not in the hook: the UI e2e needs a browser. The
performance spec (`npm run test:perf --workspace app/ui`) is run by hand only, on a quiet Mac: frame times on a shared
runner are noise.

The scan benches (`npm run bench`, criterion, a release build) are run by hand only, neither in the hook nor in CI:
their timings depend on the machine and its load. The deterministic bars of the scan (folders read, bytes per node,
the cache's cap, the data folder's footprint) are tests and run in the hook; see
[`testing.md`](./testing.md#performance).

CI (GitHub Actions) runs the same on every pull request and on `main`, release commits included: a tag is never the
first build of its tree. The workflows live in `.github/workflows/`; every action is pinned to a full commit SHA with its version in a
comment, and every tool is installed through `taiki-e/install-action` at an exact version, checked against its
checksum (nothing is downloaded and run unverified).

| Workflow     | Job     | Runs                                                                                           |
| ------------ | ------- | ---------------------------------------------------------------------------------------------- |
| `ci.yml`     | `lint`  | actionlint, `npm run lint`, `npm run check:readme`, `npm run check:notices` (after `cargo fetch --locked`), the crate dependency guard, the lock check, the version check, the text check, knip, cargo-machete, cargo-deny, typos, `npm audit` |
|              | `test`  | `npm run test:scripts`, `npm run test:cov`; the coverage and JUnit reports kept as an artifact |
|              | `e2e`   | `npm run test:e2e` with the Playwright browsers; the report kept on failure                    |
|              | `build` | `npm run build`: the app bundle, signed ad hoc with the hardened runtime, and its check (`npm run check:bundle`) |
| `codeql.yml` | analyze | CodeQL (`security-extended`) for the workflows, TypeScript and Rust; on pull requests, on `main` and weekly |
| `advisories.yml` | advisories | `cargo deny check advisories` and `npm audit --omit=dev`, weekly: an advisory published after the last push |
| `release.yml` | `build` | on a pushed tag `v*.*.*`, or by hand as a dry run: the tag is on `main`, the version check with the tag's version, `npm run test:scripts`, `npm run build:release` (the universal app and its disk image, and their check), `SHA256SUMS`; the files kept as an artifact. No caches |
|              | `release` | on a tag only: the files checked against `SHA256SUMS`, the build provenance attested, a draft release made with the image, `SHA256SUMS` and the release notes with the version's changelog section as its text; a person publishes it ([`workflow.md`](./workflow.md), "Releases") |

The `ci.yml` jobs run one after another on `macos-latest`. Each starts with `.github/actions/setup`: Node from `.nvmrc`,
the Rust toolchain from `rust-toolchain.toml`, the npm and cargo caches, `npm ci`. The cargo tools are installed at the
versions used locally; nextest runs with its `ci` profile (`.config/nextest.toml`). `npm audit` runs with `--omit=dev
--audit-level=high`. CodeQL compiles nothing, so it runs on Linux.

## Layer 6. Dependency locking

- Exact versions in `package.json` (`.npmrc`: `save-exact=true`), `package-lock.json` committed, `npm ci` in CI.
- `Cargo.lock` committed; every cargo command in hooks and CI runs with `--locked`, `npm run build` too (`tauri build
  -- --locked`).
- `npm run build:release` builds the release: one universal app for Apple silicon and Intel
  (`--target universal-apple-darwin`) in a disk image (`Fresher_<version>_universal.dmg`), then `npm run check:bundle
  -- --release` checks the app, both architectures (`lipo -archs`), the image (`hdiutil verify`) and the app inside it.
  It sets `CI=true`, with which Tauri's disk image script skips laying out the image's window through Finder, so a
  build on a developer's Mac opens no window and needs no automation permission.
- `npm run build` remaps the home folder to `~` in the binary's paths (`--remap-path-prefix`), so the bundle names no
  account and does not depend on where it was built; `scripts/check-bundle.ts` fails on a binary that still holds it.
- The Rust toolchain is pinned in `rust-toolchain.toml`; `rust-version` in the workspace is raised in the same change.
- The Node version is pinned exactly in `.nvmrc` (CI's version) and as a minimum in `engines`.
- Updates are taken by hand (`npx npm-check-updates`, `cargo update`), each major with its changelog read.

## Setting up a machine

1. Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`; any `cargo` command then installs the pinned
   toolchain.
2. Cargo tools: `cargo install --locked cargo-about cargo-deny cargo-machete cargo-nextest cargo-llvm-cov`
3. Repository tools: `brew install gitleaks typos-cli taplo actionlint`
4. Node (version in `.nvmrc`), then `npm install` at the root — it installs the git hooks (`prepare`).
