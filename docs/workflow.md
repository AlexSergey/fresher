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

## Pushing

- Push only when the user asks. Committing and merging locally is part of doing the work.
- Never bypass the hooks (`--no-verify`); fix what they report.

## Agents

- **Work in parallel with subagents whenever the work allows it**: independent steps, independent research, reviews,
  separate files or crates. Each subagent gets a clear brief (goal, files, what is already decided, what to return).
- Work that depends on another result stays sequential; two agents never edit the same files at once.
- The main session keeps the decisions, reviews what the subagents return and verifies it before reporting.

## End of work

- **`cargo clean` when the work is finished** (a plan done, or the end of a session): `target/` grows by gigabytes and
  is rebuilt on the next build.
- Before ending a session, the memory is prepared for the next one: the state of the work and the next steps.

## Decisions and docs

- The user's decisions are recorded in the docs in the same change, translated into English and dated
  ([`../CLAUDE.md`](../CLAUDE.md), "Language").
- An open question is asked with clear options and a recommendation; a trade-off is explained briefly and plainly.
- Plans follow [`plans/README.md`](./plans/README.md); only approved plans are built.
