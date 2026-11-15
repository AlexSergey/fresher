# Code quality tooling

What checks the code, when, and why. Configs live next to the code they check; this page explains them.

> **Status:** the target state, set up by plan [001](./plans/001-repository-setup/readme.md) (✅ approved). Until that
> plan is implemented, only `.editorconfig`, `.gitattributes` and `.gitignore` exist.

The system is the six-layer defence of `prod-forge/backend` (its `guide/code-quality.md`), extended with the Rust
equivalent of every TypeScript tool.

## Layer 1. Formatting and consistency

| Tool                           | Checks                                   | Command                   |
| ------------------------------ | ---------------------------------------- | ------------------------- |
| EditorConfig (`.editorconfig`) | indentation, line endings, final newline | the editor                |
| rustfmt (`rustfmt.toml`)       | Rust formatting                          | `cargo fmt --all`         |
| Prettier (`.prettierrc`)       | TS, TSX, JSON, YAML, CSS, Markdown       | `npm run format:prettier` |
| taplo (`.taplo.toml`)          | TOML formatting                          | `taplo fmt`               |

## Layer 2. Static analysis

### Rust

| Tool                                         | What it is                                                          | Command                                                          |
| -------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------- |
| `rust-toolchain.toml`                        | pins the compiler for everyone, with `rustfmt`, `clippy`, llvm      | installed by `rustup` on first use                               |
| compiler lints (`[workspace.lints.rust]`)    | stricter compiler warnings; `unsafe_code = "forbid"` in the core    | part of every build                                              |
| Clippy (`clippy.toml` + `[workspace.lints]`) | `all` + `pedantic` + picked restriction lints                       | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| cargo-deny (`deny.toml`)                     | licences, security advisories, duplicate and banned crates, sources | `cargo deny check`                                               |
| cargo-machete                                | dependencies a crate declares and never uses                        | `cargo machete`                                                  |

What the Rust lints enforce:

- **Clippy `all` + `pedantic`**; the pedantic lints that only add noise are allowed one by one in `Cargo.toml`.
- **No `unwrap()` / `expect()` / `panic!` / `todo!` / `unimplemented!` / `dbg!` / `println!` / `eprintln!`** outside
  tests: errors are returned, output goes through `tracing`.
- **No `unsafe` in the core crates**; where an FFI call is unavoidable, one unsafe operation per block with a
  `// SAFETY:` comment.
- **`#[allow(...)]` needs a reason**: `#[expect(lint, reason = "...")]`.
- **File system hygiene**: `std::fs::remove_dir_all` and `remove_file` are disallowed by `clippy.toml`
  (`disallowed-methods`) everywhere except the cleaner crate, so nothing else can delete.
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
- `tsc --noEmit` — the type check
- knip (`knip.json`) — unused files, exports and dependencies

### Whole repository

| Tool                  | What it stops                                      | Command                 |
| --------------------- | -------------------------------------------------- | ----------------------- |
| gitleaks              | keys, tokens and passwords in staged changes       | `npm run check:secrets` |
| typos (`_typos.toml`) | misspellings in code and docs (Rust, TS, Markdown) | `npm run check:typos`   |
| staged-files guard    | any file over 1 MiB, any `*.sqlite` or `.env`      | `npm run check:staged`  |

## Layer 3. Pre-commit (Husky + lint-staged)

`.husky/pre-commit` runs the repository guards (`npm run check:repo`), then lint-staged (`.lintstagedrc.cjs`), which
runs only on the staged files:

| Staged files                            | Runs                                                                        |
| --------------------------------------- | --------------------------------------------------------------------------- |
| `*.rs`                                  | `rustfmt` on the files, then Clippy once over the workspace (`-D warnings`) |
| `*.{ts,tsx}`                            | `prettier --write`, `eslint --fix` on the files, then `tsc --noEmit` once   |
| `*.{json,md,yml,yaml,css}`              | `prettier --write` (and `eslint --fix` for JSON)                            |
| `*.toml`                                | `taplo fmt`                                                                 |
| `Cargo.toml`, `Cargo.lock`, `deny.toml` | `cargo deny check` and `cargo machete`                                      |

Clippy and `tsc` are whole-project checks by nature: lint-staged runs them once when a matching file is staged, not once
per file.

## Layer 4. Commit messages (commitlint)

`.husky/commit-msg` rejects AI attribution trailers (`Co-Authored-By: Claude`, …), then commitlint enforces
Conventional Commits: `type(scope): subject`.

- Types: `ci`, `chore`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`.
- Scopes (optional): `core`, `rules`, `scanner`, `store`, `cleaner`, `app`, `ui`, `e2e`, `deps`, `docs`, `repo`, `ci`.

## Layer 5. Pre-push and CI

`.husky/pre-push` runs `npm run pre-push`:

1. `npm run lint` — `cargo fmt --check`, Clippy, `tsc`, ESLint
2. `npm run knip` and `cargo machete`
3. `cargo deny check`
4. `npm run test:cov` — Rust tests with coverage (`cargo llvm-cov nextest`, lines ≥ 90 %) and the UI unit tests with
   coverage (Vitest, ≥ 90 %)

The e2e suites (`npm run test:e2e`) are run by hand and in CI, not in the hook: the UI e2e needs a browser.

CI (GitHub Actions, macOS runners) runs the same on every pull request and on `main`, plus the e2e suites, a build of
the app bundle, `npm audit --omit=dev --audit-level=high`, and CodeQL for TypeScript and Rust.

## Layer 6. Dependency locking

- Exact versions in `package.json` (`.npmrc`: `save-exact=true`), `package-lock.json` committed, `npm ci` in CI.
- `Cargo.lock` committed; every cargo command in hooks and CI runs with `--locked`.
- The Rust toolchain is pinned in `rust-toolchain.toml`; `rust-version` in the workspace is raised in the same change.
- The Node version is pinned in `.nvmrc` and `engines`.
- Updates are taken by hand (`npx npm-check-updates`, `cargo update`), each major with its changelog read.

## Setting up a machine

1. Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`; any `cargo` command then installs the pinned
   toolchain.
2. Cargo tools: `cargo install --locked cargo-deny cargo-machete cargo-nextest cargo-llvm-cov`
3. Repository tools: `brew install gitleaks typos-cli taplo`
4. Node (version in `.nvmrc`), then `npm install` at the root — it installs the git hooks (`prepare`).
