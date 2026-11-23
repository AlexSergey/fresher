# 040/03/01 — The community files and issue templates

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, docs

`.github/CONTRIBUTING.md`, `.github/SECURITY.md` and four issue-template files; the README links them; the repository
rule names them as the exceptions to "documentation lives in `docs/`". The design:
[`../00-design.md`](../00-design.md), section 4.

## Context

- CLAUDE.md, "Documentation": every document lives under `docs/`, nothing outside it but code, its configs and
  CLAUDE.md. `README.md` at the root is the existing exception. These files are GitHub's own configuration: keep each
  short and point into `docs/` rather than repeating it.
- `npm run check:readme` (01/03) checks the relative links and anchors of `README.md` and every `.github/*.md`; a
  link from `.github/X.md` to `docs/` is written `../docs/...`. Absolute `https://` links are not checked.
- Plan 039 (step 03, "Help") may have written a wrong-finding issue template already. Before creating
  `.github/ISSUE_TEMPLATE/`, look: if a template for a wrong finding exists, keep it as it is and add only the files
  below it lacks; say so in the conclusions.
- Issue forms are YAML (`name`, `description`, `title`, `labels`, `body` with `type: markdown | input | textarea |
  dropdown | checkboxes`, `id`, `attributes`, `validations.required`). Labels used: `wrong-finding`, `new-folder`,
  `bug` (GitHub ignores a label that does not exist; creating labels is the user's, listed in 04/02).
- No personal data in any example: paths as `~/Library/Caches/com.example.App`.
- Never name another disk cleaner or reference project. English only.

## Files

- `.github/CONTRIBUTING.md` (new), at most about 40 lines:
  - one line on what Fresher is and that it deletes real files, so safety and tests come first;
  - setup: `npm install` (installs the git hooks), `npm run dev`, `npm test`;
  - read first: [`goal.md`](../docs/goal.md), [`rules.md`](../docs/rules.md),
    [`workflow.md`](../docs/workflow.md), [`architecture/`](../docs/architecture/README.md),
    [`restrictions/`](../docs/restrictions/README.md);
  - a new kind of folder: a category file in [`docs/categories/`](../docs/categories/README.md) (with its
    `**Example:**` line), negative tests first, a sandbox case file ([`testing.md`](../docs/testing.md));
  - commits: Conventional Commits, the hooks are never bypassed ([`quality.md`](../docs/quality.md));
  - "Write your own solution: code copied from another project is not accepted, whatever its licence.";
  - "A contribution is accepted under MIT OR Apache-2.0, the project's licences."
- `.github/SECURITY.md` (new), short:
  - "Supported versions": the latest `main` until the first release;
  - "What counts": anything that removes a file outside a confirmed cleanup, reaches outside the scanned folders or
    into a protected place, follows a link out of a scanned folder, sends anything over the network, or asks for more
    access than it needs; links to [`deletion-safety.md`](../docs/restrictions/deletion-safety.md) and
    [`privacy.md`](../docs/restrictions/privacy.md);
  - "How to report": privately, through the repository's Security tab → "Report a vulnerability"
    (`https://github.com/AlexSergey/fresher/security/advisories/new`); not in a public issue; what to include (macOS
    version, the commit, the steps, what was removed or reached);
  - what happens next, in one sentence: an answer, a fix on `main`, credit if wanted.
- `.github/ISSUE_TEMPLATE/wrong-finding.yml` (new) — name "Wrong finding", description "The app offered something it
  should not have", title `Wrong finding: `, label `wrong-finding`. Body: a markdown intro ("Thank you: this is the
  most useful report. Write your home folder as `~`."); inputs: the category shown (required), the path shown
  (required, placeholder `~/Library/Caches/com.example.App`), the danger level shown (dropdown Safe, Low, Medium,
  High; required); textareas: what the folder really is (required), what happened (dropdown: "I saw it, nothing was
  removed", "I removed it and put it back", "I removed it and lost something"); the macOS version and the app's
  version or commit (input).
- `.github/ISSUE_TEMPLATE/missing-folder.yml` (new) — name "A folder Fresher should know", label `new-folder`. Inputs:
  the path or pattern (required, placeholder `~/.cache/example-tool`), the tool that makes it (required), whether it
  comes back by itself and how (textarea, required), a typical size (input).
- `.github/ISSUE_TEMPLATE/bug-report.yml` (new) — name "Something else went wrong", label `bug`. The steps (required),
  what you expected, what happened (required), the macOS version (required), the app's version or commit.
- `.github/ISSUE_TEMPLATE/config.yml` (new) — `blank_issues_enabled: false`; `contact_links`: one entry "A security
  problem" → `https://github.com/AlexSergey/fresher/security/policy`, about "Please report it privately, not in an
  issue."
- `README.md` — "Help it get better": "Open an issue" becomes links to
  `https://github.com/AlexSergey/fresher/issues/new?template=wrong-finding.yml` ("Report a wrong finding") and
  `…?template=missing-folder.yml` ("Suggest a folder"); "Build and contribute" gains "How to contribute:
  [`CONTRIBUTING.md`](.github/CONTRIBUTING.md). A security problem: [`SECURITY.md`](.github/SECURITY.md)." Then
  `npm run readme:generate` (formats the file).
- `docs/restrictions/repository.md` — a new section before "Licence":

  ```markdown
  ## Documentation lives in `docs/`

  - **Rule:** every document lives under `docs/`. The exceptions are the files GitHub looks for: `README.md` at the
    root, and `.github/CONTRIBUTING.md`, `.github/SECURITY.md` and the issue templates in `.github/ISSUE_TEMPLATE/`.
    They stay short and point into `docs/`.
  - **Why:** one place for documentation (CLAUDE.md, "Documentation"); GitHub shows those files only where it looks
    for them.
  - **Enforced by:** review; `npm run check:readme` checks the README's and `.github/*.md`'s links and the README's
    generated table.
  ```

## Checks

- `npm run check:readme` passes (it now reads the two `.github/*.md` files too).
- `npx prettier --check .github README.md` passes (YAML and Markdown are Prettier-formatted by the hook).
- Each issue form parses: `node -e "for (const f of process.argv.slice(1)) require('yaml').parse(require('fs').readFileSync(f, 'utf8'))" .github/ISSUE_TEMPLATE/*.yml`
  (the `yaml` package is in `node_modules` as a transitive package; use it only here, never import it from a script,
  and add no dependency). GitHub's own form validation is seen only on the "New issue" page (04/02 lists it as a user
  walk).
- `grep -rn -i '<any other disk cleaner name>'` is a review item, not a command: no product is named.
- Commit: `docs(repo): contributing guide, security policy and issue templates`.

## Conclusions

Done. No wrong-finding template existed (plan 039 had not landed), so all four issue files were created. The README's
Help and Build sections link the templates, `CONTRIBUTING.md` and `SECURITY.md`; `repository.md` names them as the
exceptions. `scripts/readme.ts` now prints `<where>: <kind>: <message>` per problem.
