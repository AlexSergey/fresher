# Repository

## English only

- **Rule:** every file, comment, identifier, commit message, plan and doc is in English. The user's words recorded in
  the repository (answers, decisions) are translated into English. Chat may be in any language.
- **Why:** the user's decision (2026-11-15).
- **Enforced by:** review; `typos` catches some non-English words.

## Clean room

- **Rule:** no other product or project is named in the repository — no names, links or star counts of competing or
  reference apps, in code, docs, plans or commit messages. Their implementations are never copied — no code, no
  algorithm ported line by line, whatever the licence; we study their ideas only and design our own, better solution.
  Our own dependencies (crates, packages) and the tools we clean (Xcode, Docker, …) are named where needed.
- **Why:** the user's decision (2026-11-15): the project stands on its own ideas; several of those projects are GPL.
- **Enforced by:** review.

## Licence

- **Rule:** the project is free and open source under MIT OR Apache-2.0, at the user's option (`LICENSE-MIT`,
  `LICENSE-APACHE`, `NOTICE`). Every crate and package declares `MIT OR Apache-2.0`; a contribution is accepted under
  the same terms.
- **Why:** the user's decision (2026-11-15): the standard of the Rust ecosystem — Apache-2.0's patent grant, MIT's
  compatibility with GPLv2.
- **Enforced by:** `license` in `Cargo.toml` (`[workspace.package]`) and in every `package.json`; review.

## Commit messages

- **Rule:** Conventional Commits, `type(scope): subject`; no AI attribution trailers.
- **Why:** a readable history and a generated changelog; the user's policy.
- **Enforced by:** `.husky/commit-msg` and commitlint (plan 001).

## Generated and protected files

- **Rule:** `Cargo.lock`, `package-lock.json`, `app/tauri/gen/` and `*.generated.ts` (such as
  `app/ui/src/ipc/bindings.generated.ts`, the IPC types generated from the core's Rust types) are never edited by hand. No file
  over 1 MiB, no database, no `.env`, no signing key and no personal IDE folder (`.idea/`) is committed.
- **Why:** generated files are rewritten by their tools; large or secret files do not belong in git.
- **Enforced by:** `.gitignore`; the staged-files guard and gitleaks in the pre-commit hook (plan 001).

## Hooks are never bypassed

- **Rule:** no `--no-verify`; what a hook reports is fixed.
- **Why:** the hooks are the quality gate.
- **Enforced by:** CI runs the same checks (plan 001).
