# Workflow

How work is done in this repository: branches, merges, agents, cleanup. The user's decisions of 2026-11-15.

## Branches and merges

- **One branch per global plan**: `feat/NNN-<name>` (for example `feat/002-base-architecture`), from `main`. Every plan
  and step inside the global plan (`NNN/NN`) is committed to that branch, each commit through the git hooks.
- **One squash-merge per global plan**: when the global plan is done (its conclusions written, the user has looked at
  the result), the branch is squash-merged into `main` as ONE Conventional Commit naming the plan.
- Before the squash-merge, the pre-push checks pass on the branch's last commit (`npm run pre-push`).
- **No pull requests for local development** (the user's decision of 2026-11-15): the squash-merge is done locally
  with `git merge --squash`; a pull request is opened only when the user asks for one.
- **Small changes outside any plan** (an idea, the README, a rule, a doc fix) are committed **straight to `main`**, one
  commit through the hooks, no branch.
- The merged branch is kept for two weeks, then deleted.
- **An unfinished global plan may be merged with a dependent one** when the user asks (the user's decision of
  2026-11-16, plans 006 and 008): its built parts go into `main` in the dependent plan's squash-merge, it stays
  🔨 in progress, and its rest continues on a new branch from `main`. Likewise a plan whose only open step is the
  user's run by hand may be merged when the user asks (plan 007, 2026-11-17); the run's results are written on `main`.

## Worktrees

- A global plan's branch may be worked in a sibling worktree (`git worktree add ../fresher-NNN -b feat/NNN-<name>`),
  so `main` stays free for small changes while the plan is built.
- The git hooks run the Node tools, so a new worktree needs `node_modules`: run `npm ci` in it, or link the main
  checkout's folder (`ln -s ../fresher/node_modules node_modules`; the link is ignored by git).
- A worktree builds into the main checkout's `target/`, never its own: run Cargo in it with
  `CARGO_TARGET_DIR=../fresher/target` (absolute path in agents' briefs). A second `target/` is a second full build,
  10–50 GB. Remove the worktree when its plan is merged (`git worktree remove ../fresher-NNN`).
- The coverage build (`target/llvm-cov-target`, made by `npm run test:cov`) is another full build: when the disk runs
  short, remove it with `cargo llvm-cov clean --workspace`; the next coverage run rebuilds it.

## Pushing

- Push only when the user asks. Committing and merging locally is part of doing the work.
- Never bypass the hooks (`--no-verify`); fix what they report.

## Releases

A release is cut by the user from `main` (plan [013](./plans/013-release/readme.md)). A pushed tag `vX.Y.Z` starts
`.github/workflows/release.yml`, which builds the universal disk image, writes `SHA256SUMS`, attests the build
provenance and makes a **draft** release; a person checks the draft and publishes it. Versions follow semantic
versioning; 1.0.0 comes only after real testing (the user's decision of 2026-11-25).

1. **Version.** On `main`, `npm run release:version -- X.Y.Z` writes the version into every place and refreshes the
   lock files (nothing to do when the version already is X.Y.Z, as for 0.1.0).
2. **Changelog.** In [`changelog.md`](./changelog.md), the entries of "Unreleased" become `## [X.Y.Z] - YYYY-MM-DD`
   with the day of the tag, written for the people who use the app; the links at the bottom get the new version.
3. **Release notes.** `docs/releases/X.Y.Z.md`, the text of the GitHub release: what it is, the supported Macs,
   install, first launch, Full Disk Access, how to check the download, no warranty, where to report, the changelog.
   `.github/SECURITY.md` names the supported versions.
4. **Check and commit.** `npm run check:version -- X.Y.Z` passes; the change is one commit on `main`
   (`chore(repo): release X.Y.Z`) through the hooks.
5. **Dry run (optional).** Actions → Release → "Run workflow" on `main` builds the same files without a release and
   keeps them as the `release` artifact for 14 days.
6. **Push and wait.** `git push origin main`; CI passes on that commit (a tag is never the first build of its tree).
7. **Tag.** `git tag -a vX.Y.Z -m "Fresher X.Y.Z"` on that commit, then `git push origin vX.Y.Z`. The workflow
   refuses a tag that is not on `main` or that differs from the version in any place.
8. **Check the draft.** On the Releases page: the image `Fresher_X.Y.Z_universal.dmg`, `SHA256SUMS` and the notes are
   there; download the image, run `shasum -a 256 -c SHA256SUMS` and `gh attestation verify <image> --repo
   AlexSergey/fresher`; install it as the notes say and open it on the oldest supported macOS (13) and the newest,
   on Apple silicon and on Intel where a Mac is at hand: the first launch, About's version, a scan. From the second
   release on, open the new version over the data folder of the previous one (its settings, ignore list and cleanups
   are kept).
9. **Publish** the draft. Then the README's release items of
   [`deferred/readme-at-release.md`](./deferred/readme-at-release.md) are due.

A broken draft is deleted with its tag (`git push --delete origin vX.Y.Z`), fixed on `main` and tagged again. A
published release is never replaced: the fix is the next patch version.

## Agents

- **Work in parallel with subagents whenever the work allows it**: independent steps, independent research, reviews,
  separate files or crates. Each subagent gets a clear brief (goal, files, what is already decided, what to return).
- Work that depends on another result stays sequential; two agents never edit the same files at once.
- The main session keeps the decisions, reviews what the subagents return and verifies it before reporting.

## End of work

- **`cargo clean` only when a global plan is done** (the user's decision of 2026-11-15): `target/` grows by gigabytes,
  but cleaning it after every step or session makes every next check a cold build of the whole workspace.
- Before ending a session, the memory is prepared for the next one: the state of the work and the next steps.

## Decisions and docs

- The user's decisions are recorded in the docs in the same change, translated into English and dated
  ([`../CLAUDE.md`](../CLAUDE.md), "Language").
- An open question is asked with clear options and a recommendation; a trade-off is explained briefly and plainly.
- Plans follow [`plans/README.md`](./plans/README.md); only approved plans are built.
