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
then (0.0.1 to 0.5.5) are kept as history; the "Version" of a plan not yet done is not a target any more. The first
release is 0.1.0, for testers (the user, 2026-11-25); how a release is cut: [`../release.md`](../release.md) (plan [013](./013-release/readme.md)).

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
| 011 | [Problems and reasons](./011-diagnostics/readme.md)                 | 🔨 in progress | 0.1.0   | core, engine, cleaner, testing, modules, app, app/ui |
| 012 | [The storage map](./012-storage-map/readme.md)                       | 🔨 in progress | 0.1.0   | core, macos, scanner, engine, testing, sandbox, app, app/ui |
| 013 | [Release and distribution](./013-release/readme.md)                  | ✔️ done (01 on hold) | 0.1.0   | repo, ci, scripts, app, app/ui, docs              |
| 014 | [Onboarding and the first scan](./014-onboarding/readme.md)          | 📝 draft       | 0.5.0   | engine, store, app/ui                             |
| 015 | [Module groups and ignoring a module](./015-module-groups/readme.md) | 🔨 in progress | 0.1.0 | core, engine, modules, testing, app, app/ui |
| 016 | [JetBrains IDE leftovers](./016-jetbrains/readme.md) | 🔨 in progress | 0.1.0 | modules, core, sandbox, testing, app, app/ui, docs |
| 017 | [Temporary files](./017-temp-files/readme.md)                        | 🔨 in progress | 0.1.0   | core, macos, system, scanner, engine, cleaner, testing, sandbox, modules, app, app/ui |
| 018 | [Developer caches](./018-developer-caches/readme.md)                 | ✅ approved    | 0.1.0   | core, scanner, testing, modules, sandbox, e2e, app, app/ui, docs |
| 019 | [Uninstalled app leftovers](./019-uninstalled-app-leftovers/readme.md) | ✅ approved  | 0.1.0   | core, scanner, macos, system, engine, testing, modules, sandbox, e2e, app, app/ui, docs |
| 020 | [Downloads and installers](./020-downloads/readme.md)               | 🔨 in progress | 0.1.0   | core, scanner, system, engine, testing, modules, sandbox, e2e, app, app/ui, docs |
| 021 | [Stale projects](./021-stale-projects/readme.md)                    | 🔨 in progress | 0.1.0   | core, engine, scanner, modules, testing, sandbox, e2e, app, app/ui, docs |
| 026 | [The development run record](./026-dev-run-record/readme.md)       | 🔨 in progress | 0.1.0   | app, app/tauri, app/ui, core, cleaner, store, scripts, docs |
| 027 | [The scan screen](./027-scan-screen/readme.md)                     | 🔨 in progress | 0.1.0   | app/ui, app/tauri, core, sandbox, e2e, docs |
| 028 | [Ignore one item](./028-ignore-item/readme.md)                     | 🔨 in progress | 0.1.0   | app/ui, e2e, docs |
| 023 | [Local AI models](./023-local-ai-models/readme.md)                 | 🔨 in progress | 0.1.0   | modules, testing, sandbox, e2e, app, app/ui, docs |
| 024 | [Unused toolchain versions](./024-unused-toolchains/readme.md)     | 🔨 in progress | 0.1.0   | core, testing, sandbox, modules, e2e, app, app/ui, docs |
| 029 | [Ollama's models](./029-ollama-models/readme.md)                   | 🔨 in progress | 0.1.0   | core, engine, cleaner, store, testing, modules, sandbox, e2e, app, app/ui, docs |
| 030 | [Installed app caches](./030-installed-app-caches/readme.md)       | 🔨 in progress | 0.1.0   | modules, testing, sandbox, e2e, app, app/ui, docs |
| 031 | [Xcode and simulators](./031-xcode/readme.md)                      | 🔨 in progress (awaiting the user's walk) | 0.1.0   | modules, testing, sandbox, e2e, app, app/ui, docs |
| 032 | [Docker](./032-docker/readme.md)                                   | 🔨 in progress (built, awaiting the user's walk) | 0.1.0   | core, engine, testing, sandbox, modules, e2e, app, app/ui, docs |
| 033 | [Logs and crash reports](./033-logs/readme.md)                     | 🔨 in progress | 0.1.0   | modules, testing, sandbox, e2e, app, app/ui, docs |
| 034 | [Browser and web-engine caches](./034-browser-caches/readme.md)    | 🔨 in progress | 0.1.0   | modules, testing, sandbox, e2e, app, app/ui, docs |
| 035 | [The largest files](./035-large-files/readme.md)                   | 🔨 in progress | 0.1.0   | core, scanner, engine, testing, sandbox, e2e, app, app/ui, docs |
| 036 | [Uninstall an app](./036-uninstall/readme.md)                      | 🔨 in progress (awaiting the user's walk) | 0.1.0   | core, macos, system, engine, testing, sandbox, e2e, app, app/ui, docs |
| 037 | [The Trash holds the freed space](./037-trash/readme.md)           | 🔨 in progress (awaiting the user's walk) | 0.1.0   | core, macos, system, engine, testing, sandbox, e2e, app, app/tauri, app/ui, docs |
| 038 | [Demo recordings](./038-demo-recordings/readme.md)                 | 🔨 in progress (awaiting the user's walk) | 0.1.0   | sandbox, testing, app, app/tauri, app/ui, scripts, fixtures, docs |
| 039 | [The agreement, the danger zone and the wording](./039-agreement-and-wording/readme.md) | 🔨 in progress (awaiting the user's walk) | 0.1.0 | core, engine, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, repo, docs |
| 040 | [The README](./040-readme/readme.md) | 🔨 in progress (awaiting the user's walk) | 0.1.0 | repo, scripts, ci, docs |
| 042 | [A plain interface](./042-plain-interface/readme.md) | ✔️ done | 0.1.0 | core, engine, modules, testing, sandbox, e2e, app, app/tauri, app/ui, scripts, docs |
| 041 | [The security audit](./041-security-audit/readme.md) | ✔️ done | 0.1.0 | every crate and module, app, scripts, repo, docs |
