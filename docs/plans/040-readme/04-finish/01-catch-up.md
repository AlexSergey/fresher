# 040/04/01 — Catch up with `main`

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, docs

Plans 031, 032, 035, 036, 037, 038 and 039 are planned to merge into `main` before this plan ends. Bring their
results into the README: their category files' example lines, the regenerated table, plan 038's picture, plan 039's
agreement words and the issue template it may have written, plan 032's Docker sentence. Whatever has not landed yet
is written down, not waited for. The design: [`../00-design.md`](../00-design.md), section 6.

## Context

- The README (02/01, 03/01) has a generated block between `<!-- what-it-finds:start … -->` and
  `<!-- what-it-finds:end -->`; `npm run readme:generate` rewrites it from `docs/categories/` and formats the file;
  `npm run check:readme` refuses a category file without its `**Example:** <text>` line (one line, at most 80
  characters, no `|`, true of the file's "Where it lives"), dead relative links and anchors, images outside `assets/`,
  remote images other than shields.io, and the words "watch mode".
- Never merge by hand `Cargo.lock`, `package-lock.json` or `*.generated.ts`: take `main`'s and regenerate if needed.
- Never run `xcrun`, `simctl`, `xcodebuild`, a state-changing `docker` command, or `gh repo edit`.

## Do

1. `git merge main` into `feat/040-readme` (a merge commit, not a rebase: the branch is squash-merged later). Resolve
   conflicts in `README.md` by keeping this branch's text and regenerating; in `docs/categories/*.md` by keeping both
   sides' lines (the example line and `main`'s changes); in `package.json`'s `pre-push` by keeping every check of both.
   Record which plans' commits came in (`git log --oneline <old main>..main`).
2. **The category files that arrived**: `node scripts/readme.ts check` lists each file with `no-example`; give each an
   example line by 01/01's rules (read its "Where it lives"). Examples of the kind expected, to correct against the
   files: Xcode's simulators `~/Library/Developer/CoreSimulator/Devices/<id>` of a removed runtime; Docker — if plan
   032 ships a category file, its example says what it shows (Docker's own space is never offered); the largest files
   — "a file over 1 GB anywhere in your home"; the Trash, uninstall — only if they ship category files.
3. `npm run readme:generate`; read the table: every row's example is true; the count sentence matches the catalogue.
4. **The picture** (plan 038): if `assets/demo/overview.gif` exists, add it under the head, centred, width 800:
   `<p align="center"><img src="assets/demo/overview.gif" width="800" alt="Fresher scanning a Mac made of generated
   sample data, then showing what it found by danger level"></p>` and under it the line "The recording shows generated
   sample data, not a real Mac." (plan 038, agent's call 5). If `assets/demo/cleanup.gif` exists too, do not add it
   (one picture in the first screen); note it for the user. If neither exists: no picture; the conclusions and the
   user's review list say so.
5. **The words** (plan 039): if its agreement text has landed (search `app/ui/src/locales/en.json` and
   `crates/core/locales/en.json` for the agreement's keys, and plan 039's readme or steps for the final text), make
   the README's "No warranty" bullets say the same things in the same order, and its voice follow `docs/writing.md` if
   that exists (read it; apply its glossary words: "remove", "move to the Trash", "delete permanently"). If plan 039
   wrote a wrong-finding issue template, keep one of the two (plan 039's if it is referenced from the app or its docs;
   otherwise ours) and fix the README's template link to it.
6. **Docker** (plan 032): if the module `docker` is on `main`, the "What it does not do" sentence "It never removes
   anything of Docker's." becomes "It shows what Docker takes and the command that frees each part, and never runs
   it." — true only if the module does so; read its category doc or plan conclusions first.
7. **The Trash** (plan 037) and **uninstall** (plan 036): if they landed, read their conclusions; the README's "Trash
   first" line and "What it does not do" stay true (the app never empties the Trash). If uninstall shipped and is
   reachable in the app, add one sentence after the generated block of "What it finds": "It can also remove an app you
   choose, together with what it left, after showing you everything first." Otherwise add nothing.
8. Read the whole README once more against `main`: every claim is true of what is on `main` now (no watch mode, no
   release, no feature of a plan not merged).

## Checks

- `npm run check:readme` passes; `npx prettier --check README.md .github`.
- `npm run test:scripts`; `npm run lint`.
- `git diff 33d51a6c -- crates modules app` shows only what came from `main` (none of this branch's commits touch
  code): `git log --oneline main..HEAD -- crates modules app` prints nothing but the merge.
- Commit (after the merge commit): `docs(repo): bring the README up to date with the merged plans`.

## Conclusions

- `main` (b9cafed1) merged in (merge commit `7da9539f`): plans 037 (the Trash), 032 (Docker), 038 (demo recordings),
  035 (the largest files), 036 (uninstall) and 039 (the agreement and the wording) and two dependency fixes; plan 031
  was already in the branch's base.
- Conflicts: `package.json`'s `pre-push` and `docs/quality.md` keep every check of both (`check:readme` after
  `npm run lint`, then `check:crate-deps`, then `check:texts`); `docs/dependencies.md` keeps both rows (`ffmpeg`,
  `rsvg-convert`); `docs/plans/README.md` takes `main`'s rows for 032–039. Plan 039's
  `.github/ISSUE_TEMPLATE/wrong-finding.yml` is kept (the app's "Removed something you need?" help links to it), ours
  dropped; the README's link already pointed at that name. `config.yml` keeps this plan's design (no blank issues, a
  contact link to the security policy) over plan 039's `blank_issues_enabled: true`: for the user to confirm.
- Example lines for the three category files that arrived (`core-own-file`, `core-uninstall-keep-data`,
  `core-uninstall-everything`); Docker and the Trash ship no category file. The table counts 44 kinds.
- No picture: plan 038 committed no `assets/demo/*.gif` (the recording is made by hand by the user); the README has no
  image under the head until it does.
- "No warranty" says the agreement's five lines (`core.agreement.line.*`) in their order; "Trash first" says "deleting
  permanently" (the glossary of `docs/writing.md`); the head says "removes"; Docker's sentence is plan 032's; one
  sentence on uninstalling an app follows the table; the wrong-finding ask follows plan 039's form.
