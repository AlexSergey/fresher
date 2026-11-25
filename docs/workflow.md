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
- A worktree builds into the main checkout's `target/` only while nothing else builds there: run Cargo in it with
  `CARGO_TARGET_DIR=../fresher/target` (absolute path in agents' briefs). Cargo names a workspace's crates by their
  path inside the workspace, so two checkouts building into one `target/` at the same time overwrite each other's
  crates and fail with errors of the other's code. When `main` or another worktree builds meanwhile (a push's hooks,
  another agent), the worktree uses its own `target/`: a second full build, 10–50 GB, removed with the worktree
  (`git worktree remove ../fresher-NNN`).
- The coverage build (`target/llvm-cov-target`, made by `npm run test:cov`) is another full build: when the disk runs
  short, remove it with `cargo llvm-cov clean --workspace`; the next coverage run rebuilds it.

## Pushing

- Push only when the user asks. Committing and merging locally is part of doing the work.
- Never bypass the hooks (`--no-verify`); fix what they report.

## Releases

A release follows [`release.md`](./release.md): the strategy, the versions, what a release holds, signing, the
changelog and the steps to cut one.

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
