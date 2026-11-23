# 040/04/02 — Docs and conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

The docs the rules demand, the deferred item for the first release, the plan's conclusions as built, and this plan's
section in the user's review list. The design: [`../00-design.md`](../00-design.md), section 7.

## Context

- Docs rules (CLAUDE.md, "Documentation"): a change to the developer workflow updates its doc in the same change. The
  catalogue's format (01/01), `docs/quality.md` (02/01), `docs/restrictions/repository.md` (03/01) and
  `docs/dependencies.md` (03/02) were updated by their steps; check them, do not repeat them.
- No category was added or changed in meaning; no Rust or UI code changed: the architecture docs and the coverage are
  untouched.

## Files

- `docs/testing.md` — the repository scripts' part (find `npm run test:scripts` and the scripts' table): a line for
  `scripts/readme.ts` and its three test files (`readme-catalogue.test.ts`, `readme-links.test.ts`, `readme.test.ts`),
  what they cover, and that no test reads the real `docs/` or `README.md`.
- `docs/manual-testing.md` — one walk item, in the section where repository-level walks go (or a new short section
  "The README on GitHub" before "If something looks wrong"): after a push, open the repository page on GitHub; read
  the README in the light and the dark theme and at a phone's width (the browser's responsive mode, 390 px); the logo,
  the badges, the picture (if any), the table and every link work; the "New issue" page lists the three templates and
  each form opens; the social preview shows when the link is pasted into a chat.
- `docs/deferred/readme-at-release.md` (new) and its row in `docs/deferred/README.md` (agreed 2026-11-22, due "when
  plan 013 publishes the first release"): the Install block becomes the download link to the latest release, the
  package manager's one line, the macOS version and the first-launch note; the `status: pre-release` badge becomes the
  release badge (`https://img.shields.io/github/v/release/AlexSergey/fresher`); the social image re-rendered if the
  head changed; the "Supported versions" of `SECURITY.md` names the release. In the deferred file's format (read
  another file there first).
- `docs/ideas/best-practices.md` — item 27: one line saying the README's table is generated (plan 040) and the picture
  comes from plan 038; keep the idea's text otherwise.
- `docs/plans/README.md` — row 040: status 🔨 in progress, modules `repo, scripts, ci, docs`.
- `docs/plans/040-readme/readme.md`, `00-design.md` and every plan's and step's "Conclusions" — as built: what was
  built, what changed from the design and why, the numbers (the README's lines before and after, categories in the
  table, script tests added), the by-hand mutation checks of 02/01 (each failed as it should), what of plans
  031–039 was on `main` and brought in (04/01). Statuses: every step and plan 🔨 in progress, the global plan
  🔨 in progress "awaiting the user's walk".
- `docs/deferred/user-review-2026-11-22.md` — append a section `## 040 — The README` (create the file and its row in
  `docs/deferred/README.md` only if absent; on 33d51a6c both exist), short bullets:
  - **Walks by hand**: the README on GitHub after the push (light, dark, phone width; every link; the table); the "New
    issue" page and the three forms; the social preview in a pasted link.
  - **For the user to apply on GitHub**: the description and topics with the exact `gh repo edit` command from
    03/02's conclusions; upload `assets/social-preview.png`; enable private vulnerability reporting; create the labels
    `wrong-finding`, `new-folder`, `bug` if wanted.
  - **Facts not verified**: the bundle's output folder named in Install (if not confirmed by a build); how GitHub
    renders the long table on a phone; the issue forms' validation by GitHub.
  - **Agent's calls worth a second look**: the head's wording ("Your Mac is full. See why." and the line under it);
    no picture until plan 038's GIF (and whether it landed); the explicit `**Example:**` line in every category file
    and the 80-character limit; the table all visible, sorted by danger (fold it into `<details>`?); the stars badge
    removed and one ask at the end; the "No warranty" words before plan 039's final text (if it had not landed); the
    "watch mode" ban list in the check; images only from `assets/` and shields.io; the community files in
    `.github/` as an exception to "docs only".
  - **Known gaps**: the Install block waits for plan 013 ([deferred](./readme-at-release.md)); absolute links are not
    checked (no network in a check); whatever of plans 031–039 had not landed by 04/01.

## Checks

- `npm run pre-push` passes on the branch's last commit.
- `npm run check:readme`; `npm run check:typos`.
- `git log --oneline main..HEAD -- crates modules app` prints nothing but merge commits.
- Commit: `docs(docs): plan 040 conclusions and the user's review list`.

## Conclusions

Done. `docs/testing.md` lists `scripts/readme.ts` and its three test files; `docs/manual-testing.md` has a section
"The README on GitHub"; `docs/deferred/readme-at-release.md` holds the release swap; `docs/ideas/best-practices.md`
item 27 says the table is generated; CLAUDE.md names the community files as allowed outside `docs/`; the plans table
and every status say "awaiting the user's walk"; the user's review list has its `## 040 — The README` section.
