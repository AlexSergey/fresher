# Project Guidelines

Fresher: a fast macOS disk cleaner with a Rust core and a Tauri 2 shell (React + TypeScript UI). It explains where
the disk space went, in plain English, with a danger level on every category, and deletes only what the user
explicitly confirms. Read [`docs/goal.md`](./docs/goal.md) for the goal.

## Read first

Before planning or implementing anything, read:

- [`docs/workflow.md`](./docs/workflow.md) — how work is done: branches, merges, subagents, cleanup.

- **[`docs/rules.md`](./docs/rules.md) — the project rules. Every plan and change is checked against them before it is
  built.** A plan or change that breaks them is not built.
- [`docs/goal.md`](./docs/goal.md) — what the product is and what it must achieve.
- [`docs/restrictions/`](./docs/restrictions/README.md) — hard constraints (deletion safety, privacy, dependencies,
  repository), each with its reason and where it is enforced.
- [`docs/dependencies.md`](./docs/dependencies.md) — every dependency and tool with its version and licence. A change
  that adds, removes or updates a dependency updates it in the same change.
- [`docs/architecture/`](./docs/architecture/README.md) — crates, layers and data flow.
- [`docs/categories/`](./docs/categories/README.md) — the catalogue of cleanup categories and their danger levels. A
  change that adds or changes a category updates it in the same change.
- [`docs/deferred/`](./docs/deferred/README.md) — agreed work postponed for later. Before starting a new stage, check
  whether something there is due and remind the user.
- [`docs/known-issues/`](./docs/known-issues/README.md) — defects we live with, with what was tried.
- [`docs/ideas/`](./docs/ideas/README.md) — the user's ideas, not planned yet. When designing a stage an idea touches,
  mention it to the user; a new idea from the user gets its file and a row there.

## Repository map

| Folder       | What                                                                | Stack                     |
| ------------ | ------------------------------------------------------------------- | ------------------------- |
| `crates/`    | the Rust core: domain, engine, rules, scanner, store, cleaner       | Rust (Cargo workspace)    |
| `modules/`   | the feature modules: one crate each (`fresher-module-<id>`)         | Rust (Cargo workspace)    |
| `app/tauri/` | the Tauri 2 shell: thin commands that call the core                 | Rust                      |
| `app/ui/`    | the user interface                                                  | TypeScript, React, Vite   |
| `fixtures/`  | small fake file trees and golden reports used by tests              | —                         |
| `scripts/`   | repository guards run by the git hooks                              | TypeScript (Node) / shell |
| `docs/`      | ALL documentation: goal, rules, architecture, categories, plans     | Markdown                  |
| `NO_COMMIT/` | local experiments and scratch work; ignored by git, never committed | anything                  |

The layout is created by plan [001](./docs/plans/001-repository-setup/readme.md); until it is implemented some folders
do not exist yet.

## Language

- English only in every file, comment, identifier, commit message, plan and doc.
- When the user's words (answers, decisions) are recorded in the repository, they are translated into English.
- Chat language is the user's.

## Clean Room

- Never copy implementations from other projects: no code, no ported algorithms line by line, whatever their licence.
- Study other projects for their ideas only, then design our own solution and bring it to the ideal: simpler, safer
  and better tested than what inspired it.
- Never name other products or projects in the repository (code, docs, plans, commit messages); write down the idea,
  not its source. Our own dependencies and the tools the app cleans may be named.
- Research notes with names stay in chat or in `NO_COMMIT/`. Details:
  [`docs/restrictions/repository.md`](./docs/restrictions/repository.md), "Clean room".

## Important

- Write production-ready code
- Prefer small focused files
- Add comments only when necessary
- Prefer explicitness over magic
- Preserve existing architecture and conventions
- Follow existing project patterns before introducing new ones
- Prefer extending existing code over introducing new abstractions
- Reuse existing utilities before creating new ones
- Do not refactor unrelated code unless explicitly requested
- Split overly complex files into smaller modules
- Prefer minimal diffs
- Avoid reformatting unrelated code
- Prefer readable and maintainable code
- Prefer simplicity over clever abstractions
- Avoid unnecessary abstractions

---

## Safety Rules (the product deletes user files)

- Nothing is deleted without an explicit confirmation from the user for that exact set of items
- The core never deletes on its own: the scanner and the rules only report; only the cleaner deletes, and only from a
  confirmed plan
- Move to the Trash by default; a permanent delete is a separate, explicit choice
- Never follow symlinks out of a scanned root; never cross into a protected path (see
  [`docs/restrictions/deletion-safety.md`](./docs/restrictions/deletion-safety.md))
- A rule that cannot tell what a path is does not claim it; there is no "Other" category
- Tests never touch the real home folder: every root is injected and tests run on temporary fixture trees

---

## Rust Rules

- Cargo workspace: one concern per crate; every crate's `Cargo.toml` has `[lints] workspace = true` and takes
  `edition`, `rust-version`, `license` and `publish` from the workspace
- Code must pass `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`
- Silence a lint only with `#[expect(lint, reason = "...")]`, never a bare `#[allow]`
- No `unwrap()` / `expect()` / `panic!` / `todo!` / `dbg!` / `println!` outside tests
- Typed errors in library crates (`thiserror`); `anyhow` only at the binary edge
- `unsafe_code = "deny"` in the workspace; `#![forbid(unsafe_code)]` at the root of every crate but `fresher-macos`,
  checked by the crate guard; in `fresher-macos` only the module `ffi` may use `unsafe`, one operation per block, each
  with a `// SAFETY:` comment stating the invariant
- Prefer enums over flags and stringly-typed values
- Prefer `&Path` / `PathBuf` over strings for paths; never build paths by string concatenation
- Keep the scan allocation-light and parallel; never block the UI thread
- Public items of a crate are the crate's API: keep them few, explicit and documented

---

## TypeScript / React Rules

- TypeScript strict mode; avoid `any`
- Prefer explicit types for public APIs
- Prefer `type` over `interface`
- Use discriminated unions when appropriate
- Prefer readonly where possible
- Avoid unnecessary generics
- Components are small and presentational; the core does the work, the UI renders it
- Every call to the Rust core goes through one typed IPC module; components never call `invoke` directly
- No business logic in the UI: sizes, danger levels and descriptions come from the core

---

## Testing Rules

- Tests must be deterministic: no wall clock, no network, no real home folder
- Keep tests readable and explicit
- Avoid duplicated setup logic
- Use descriptive test names
- Mock external dependencies when appropriate (the Docker CLI, the Trash, the clock)
- Keep unit and integration tests isolated

### Test Structure

- Negative test cases must go first
- Positive test cases must go after negative cases
- Negative and positive cases must be placed in separate blocks
- Keep test structure consistent across the project

Rust:

```rust
#[cfg(test)]
mod tests {
    mod negative {
        #[test]
        fn rejects_a_path_outside_the_root() {}
    }

    mod positive {
        #[test]
        fn finds_stale_jetbrains_settings() {}
    }
}
```

TypeScript:

```ts
describe('CategoryCard', () => {
  describe('negative cases', () => {
    it('shows nothing to clean when the size is zero', () => {});
  });

  describe('positive cases', () => {
    it('shows the danger level badge', () => {});
  });
});
```

Kinds of tests, bars and where they live: [`docs/testing.md`](./docs/testing.md).

---

## Code Quality

- Git hooks check every commit and push: see [`docs/quality.md`](./docs/quality.md)
- Commit messages follow Conventional Commits: `type(scope): subject`
- Never bypass the hooks (`--no-verify`); fix what they report

## Git Commit Policy

- Never add `Co-Authored-By` or any other Claude/AI attribution to commits or pull requests
- Commit messages describe only the actual changes
- The `commit-msg` hook rejects messages with AI attribution trailers

## Workflow

**Read [`docs/workflow.md`](./docs/workflow.md) and follow it.** In short:

- One branch per global plan (`feat/NNN-<name>`), one squash-merge into `main` when the global plan is done; small
  changes outside any plan go straight to `main`
- Push only when the user asks
- Work in parallel with subagents whenever the work allows it
- `cargo clean` only when a global plan is done

---

## Ignore Rules

Do not analyze or modify generated/dependency files.

Ignore:

- node_modules
- dist
- target
- coverage
- app/tauri/gen
- \*.generated.ts
- package-lock.json
- Cargo.lock

Never edit generated code manually.

---

## Cost Saving Rules

- Run only affected tests when possible (`cargo nextest run -p <crate>`, `npx vitest run <file>`)
- Do not run the entire test suite for small isolated changes
- If only one test file was modified, run only that test file
- Read only files relevant to the current task
- Avoid scanning the entire repository unless necessary

---

## Documentation

- Every document lives under `docs/`; nothing outside it except code, its configs and this file
- When a change affects the goal, architecture, categories, restrictions or developer workflow, update the relevant doc
  in the same change; do not defer doc updates to a follow-up
- Do not update docs for internal refactors with no observable effect

## Planning

- Before writing a new plan, read `docs/goal.md`, `docs/rules.md`, `docs/architecture/` and `docs/restrictions/`
- Plans follow [`docs/plans/README.md`](./docs/plans/README.md): a global plan (`NNN-<name>/` with `readme.md` and
  `00-design.md`), plans inside it, one file per step; every level carries status, version, date and modules
- The design ends with open questions, the user's decisions and the agent's calls to check; implement only approved
  plans
- A plan is done when its conclusions are written and the user has looked at its result
- Shortcuts go to `docs/hacks/`, known defects to `docs/known-issues/`, postponed work to `docs/deferred/`, failed
  attempts to `docs/postmortem/`
