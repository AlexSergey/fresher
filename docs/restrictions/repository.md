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

## Documentation lives in `docs/`

- **Rule:** every document lives under `docs/`. The exceptions are the files GitHub looks for: `README.md` at the
  root, and `.github/CONTRIBUTING.md`, `.github/SECURITY.md` and the issue templates in `.github/ISSUE_TEMPLATE/`.
  They stay short and point into `docs/`.
- **Why:** one place for documentation (CLAUDE.md, "Documentation"); GitHub shows those files only where it looks
  for them.
- **Enforced by:** review; `npm run check:readme` checks the README's and `.github/*.md`'s links and the README's
  generated table.

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
  `app/ui/src/ipc/bindings.generated.ts`, the IPC types generated from the core's Rust types) and `*.generated.json`
  (`app/ui/src/screens/settings/notices.generated.json`, the licence notices, `npm run notices:generate`) are never
  edited by hand. No file
  over 1 MiB, no database, no `.env`, no signing key and no personal IDE folder (`.idea/`) is committed.
- **Why:** generated files are rewritten by their tools; large or secret files do not belong in git.
- **Enforced by:** `.gitignore`; the staged-files guard and gitleaks in the pre-commit hook (plan 001).

## Assets

- **Rule:** `assets/` holds the demo GIFs and the images the README uses, nothing else. A GIF in `assets/demo/` is
  recorded by `npm run demo:record`, within the recorder's budget (up to 5 MiB); every other committed file keeps the
  1 MiB limit.
- **Why:** the README shows the app; media belongs in one folder, apart from the code.
- **Enforced by:** the staged-files guard (docs/hacks), the recorder's budget check.

## The website

- **Rule:** `site/` holds the source of the website, getfresher.app (plan
  [046](../plans/046-website/readme.md)): static HTML and one style sheet, no framework, no build tool, no font or
  script from another site but the visit counter's. `npm run site:build` (`scripts/site.ts`) fills it from
  `docs/categories` and the latest release, copies its images from `assets/` and writes `dist/site/`, which git
  ignores; only `.github/workflows/site.yml` uploads it. The counter's site code is the one value in
  `site/config.json`; the hosting's access is in the repository's secrets, never in a file. The site names no other
  product.
- **Why:** one stable address for the download and the project; the list of kinds of files never drifts from the
  catalogue; nothing secret is committed.
- **Enforced by:** the build refuses a page with a slot left or unknown, a release without its one universal disk
  image, or a counter code that is not one (`scripts/site.test.ts`, `scripts/site-pages.test.ts`); review.

## Hooks are never bypassed

- **Rule:** no `--no-verify`; what a hook reports is fixed.
- **Why:** the hooks are the quality gate.
- **Enforced by:** CI runs the same checks (plan 001).
