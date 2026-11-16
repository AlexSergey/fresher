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

| #   | Global plan                                          | Status         | Version | Modules                  |
| --- | ---------------------------------------------------- | -------------- | ------- | ------------------------ |
| 001 | [Repository setup](./001-repository-setup/readme.md) | ✔️ done | 0.0.1   | repo, ci, workspace, app |
| 002 | [Base architecture](./002-base-architecture/readme.md) | ✔️ done       | 0.1.0   | core, engine, store, modules, rules, testing, app |
| 003 | [Base UI](./003-base-ui/readme.md)                     | ✔️ done           | 0.2.0   | app/ui, app/tauri, core, engine |
| 004 | [Fast scan and search](./004-fast-scan/readme.md)      | ✅ approved       | 0.3.0   | core, macos, scanner, store, engine, testing, app |
| 005 | [Settings](./005-settings/readme.md) | 📝 draft | 0.4.0 | core, store, engine, app/ui |
| 006 | [Permissions and privileges](./006-permissions/readme.md) | ✅ approved | 0.4.0 | core, macos, engine, testing, app, app/ui |
| 007 | [Safe development](./007-safe-development/readme.md) | 📝 draft | 0.1.0 | testing, cleaner, engine, app, repo |
| 008 | [Safe removal and the cleanup journal](./008-safe-removal/readme.md) | 📝 draft | 0.5.0 | core, cleaner, macos, store, engine |
| 009 | [System knowledge](./009-system-knowledge/readme.md) | 📝 draft | 0.5.0 | core, macos, system, engine, testing |
| 010 | [History and watch mode](./010-history/readme.md) | 📝 draft | 0.6.0 | core, store, engine, app/ui |
| 011 | [Errors, logs and diagnostics](./011-diagnostics/readme.md) | 📝 draft | 0.5.0 | core, engine, app, app/ui |
| 012 | [The storage map](./012-storage-map/readme.md) | 📝 draft | 0.5.0 | core, engine, system, app/ui |
| 013 | [Release and distribution](./013-release/readme.md) | 📝 draft | 1.0.0 | repo, ci, app |
| 014 | [Onboarding and the first scan](./014-onboarding/readme.md) | 📝 draft | 0.5.0 | engine, store, app/ui |
