# Plans

Every plan of the repository lives here. Before writing a plan, read [goal.md](../goal.md), [rules.md](../rules.md),
[architecture/](../architecture/README.md) and [restrictions/](../restrictions/README.md).

## Format

- A **global plan** is a numbered folder `NNN-<name>/` with `readme.md` (header, goal, its plans in order,
  conclusions) and `00-design.md` (the design it argues from).
- Inside it, each **plan** is a folder `NN-<name>/` with `readme.md` (header, its steps in order, conclusions) and one
  file per **step**: `NN-<name>.md`.
- Every global plan, plan and step opens with the same header:

  ```
  **Status:** <status> · **Version:** <target version> · **Date:** YYYY-MM-DD · **Modules:** <what it touches>
  ```

- Statuses: 📝 draft · ✅ approved · 🔨 in progress · ⏸ hold · ✔️ done · 🔒 closed (stopped, with the reason) ·
  ❌ failed (tried, did not work; linked to `../postmortem/`).
- A plan that adds or changes a module or a category lists its case file, `fixtures/sandboxes/<feature>.toml`, among
  its files, and its finish runs `npm run dev:sandbox -- <feature>` and walks `EXPECTED.md` by hand
  ([`../testing.md`](../testing.md), "Sandboxes").
- A step is small enough for one session and one commit: the files, the interfaces it produces for later steps, the
  checks that prove it is done.
- Every global plan runs in one branch `feat/NNN-<name>` and is squash-merged into `main` once, when it is done
  ([`../workflow.md`](../workflow.md)).

## Design

`00-design.md` covers the intent (the user's task, translated into English), goals, non-goals, the design itself, and
ends with:

- **Open questions** — what cannot be decided without the user.
- **Decisions** — the user's answers, dated. Not revisited.
- **Agent's calls (to check)** — choices made without the user, each waiting for a confirm or an overrule.

Only ✅ approved plans are implemented. A plan is ✔️ done when its **Conclusions** section is written (what was
measured, decided, learned) and the user has looked at its result.

## Priority list

Rows are in priority order: the order they are implemented in — 001, then 002. The 📝 drafts below them are listed by
number; their order is set when they are approved.

**Versions.** The version was reset to 0.1.0 on 2026-11-16 and stays there while the app is built: plans do not bump
it (the user's decision: nobody has tested the app or looked for its bugs yet). The versions in the plans done before
then (0.0.1 to 0.5.5) are kept as history; the "Version" of a plan not yet done is not a target any more. Numbering and
an automated release are decided before the first release ([`../deferred/release-versioning.md`](../deferred/release-versioning.md)).

| #   | Global plan                                                          | Status         | Version | Modules                                           |
| --- | -------------------------------------------------------------------- | -------------- | ------- | ------------------------------------------------- |
| 001 | [Repository setup](./001-repository-setup/readme.md)                 | ✔️ done        | 0.0.1   | repo, ci, workspace, app                          |
| 002 | [Base architecture](./002-base-architecture/readme.md)               | ✔️ done        | 0.1.0   | core, engine, store, modules, rules, testing, app |
| 003 | [Base UI](./003-base-ui/readme.md)                                   | ✔️ done        | 0.2.0   | app/ui, app/tauri, core, engine                   |
| 004 | [Fast scan and search](./004-fast-scan/readme.md)                    | ✔️ done        | 0.3.0   | core, macos, scanner, store, engine, testing, app |
| 005 | [Settings](./005-settings/readme.md)                                 | ✔️ done        | 0.5.5   | core, engine, app, app/ui                         |
| 006 | [Permissions and privileges](./006-permissions/readme.md)            | 🔨 in progress | 0.4.0   | core, macos, engine, testing, app, app/ui         |
| 007 | [Safe development](./007-safe-development/readme.md)                 | 🔨 in progress | 0.1.0   | testing, sandbox, cleaner, core, app, app/ui, repo |
| 008 | [Safe removal and the cleanup journal](./008-safe-removal/readme.md) | ✔️ done        | 0.5.0   | core, cleaner, macos, store, engine               |
| 009 | [System knowledge](./009-system-knowledge/readme.md)                 | ✔️ done        | 0.5.5   | core, macos, system, engine, cleaner, testing     |
| 010 | [History and watch mode](./010-history/readme.md)                    | 📝 draft       | 0.6.0   | core, store, engine, app/ui                       |
| 011 | [Errors, logs and diagnostics](./011-diagnostics/readme.md)          | 📝 draft       | 0.5.0   | core, engine, app, app/ui                         |
| 012 | [The storage map](./012-storage-map/readme.md)                       | 🔨 in progress | 0.1.0   | core, macos, scanner, engine, testing, sandbox, app, app/ui |
| 013 | [Release and distribution](./013-release/readme.md)                  | 📝 draft       | 1.0.0   | repo, ci, app                                     |
| 014 | [Onboarding and the first scan](./014-onboarding/readme.md)          | 📝 draft       | 0.5.0   | engine, store, app/ui                             |
