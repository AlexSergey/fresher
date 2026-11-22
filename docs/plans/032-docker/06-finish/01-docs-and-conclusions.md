# 032/06/01 — Docs, runs and conclusions

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Read first: `CLAUDE.md` ("Documentation"), `docs/plans/README.md`, [`../00-design.md`](../00-design.md) (every
section, the facts to check and the agent's calls), every step's conclusions in this plan, and as the model of
conclusions `docs/plans/030-installed-app-caches/readme.md` ("Conclusions").

## Files

- `docs/architecture/modules.md` — the `Module` trait's sink: `describe` and what an insight is (never a finding,
  never counted, never stored; a restore describes again); a paragraph **`docker` is code with a data table, and the
  example of a module that describes and never offers** (the runtimes table, the gate: app running and socket
  present, never restored; the runtime's own CLI with `--context`; the four parts and their commands; the disk file's
  figures; no category); the checklist: "A module that describes space it does not offer uses `describe`, never a
  finding."
- `docs/architecture/README.md` — the event's `insights` where `ModuleFinished` and the data flow are described; the
  UI's section and screen where the screens are listed.
- `docs/categories/README.md` — a short section "Described, not cleaned": Docker's parts and their levels (build cache
  Safe, images Low, stopped containers Medium, volumes High) are shown as wording only; no category, nothing to select
  (agent's calls 4 and 12).
- `docs/testing.md` — "Sandboxes" and "The case file": `[[case.tool]]` (program relative to the sandbox, exact
  arguments, status, inline output), sandbox mode running no program but `/usr/bin/open`, the autotest failing on an
  unstated call, `access = { other-apps-data = … }`, `written`, `[[case.insight]]` and the golden's `insights`; the
  fixture homes list gains `docker-*`; the core e2e list gains `docker.rs`.
- `docs/manual-testing.md` — walk items in sections 1.19 and 2.17 (1.15-1.18 and 2.13-2.16 are taken by plans 033,
  034, 037 and 031): `npm run dev:sandbox -- docker` (and `-stopped`, `-no-answer`, `-left-behind`):
  the section, the cards, the screen, Copy and a paste into a text editor, nothing selectable, Review unaffected, the
  banner naming "tools"; then on the developer's Mac with a runtime running: the card's numbers against `docker system
  df` run by hand.
- `docs/deferred/sandbox-world.md` — "Partly taken by plan 032": the tools (stated answers, every other call refused
  and recorded, sandbox mode running no program but `/usr/bin/open`) and other apps' data in the world's access; open
  files and the steps between scans stay deferred. Its row in `docs/deferred/README.md` updated.
- `docs/deferred/docker-other-runtimes.md` (new) and its row in `docs/deferred/README.md` — Colima, Lima, Podman
  machines and Rancher Desktop (what each needs: a sign that it is up that is not an app, the place of its CLI, its
  disk file's place and how it gives space back), one line per image or volume (`docker system df -v`), "Show in
  Finder" for a disk file, a link from the storage map's line to the insight. Due: after this plan's run on the
  developer's Mac.
- `docs/ideas/developer-environment-cleaner.md` — the Docker line: "Built by plan 032 as a description: the four parts
  with their commands and the disk file's figures; the app never cleans Docker."
- `docs/plans/README.md` — plan 032's row: 🔨 in progress (built, awaiting the user's walk).
- `docs/dependencies.md` — checked: only used-by columns changed (no new dependency).
- **This plan's conclusions**: `docs/plans/032-docker/readme.md` "Conclusions" written as built, in the shape of plan
  030's (what was built; the numbers on the last code commit: `npm run pre-push`, Rust tests and coverage,
  `modules/docker` alone, the e2e, the UI unit tests and coverage, Playwright in chromium and WebKit, the
  `git diff 97d9b314 -- crates/cleaner crates/scanner crates/store crates/rules crates/macos crates/system` check
  (empty), perf not run with the reason; what changed from the design and why; the by-hand mutation checks of 04/03;
  the calls taken while building). Status 🔨 **awaiting the user's walk** on the global readme, the design, every plan
  readme and every step whose work is built. `00-design.md` "Conclusions": a short pointer to the readme's.
- `docs/deferred/user-review-2026-11-22.md` — it exists (created by an earlier plan of the session); append to it.
  Add a section **Plan 032 — Docker** with short bullets:
  - **Walk by hand**: `npm run dev:sandbox -- docker` and the three other case files, every line of `EXPECTED.md`;
    Copy in the built app (`npm run tauri build`, not only the dev web view).
  - **Facts not verified (read-only, on a Mac with the runtime running)**: `docker --context orbstack system df --format
    json` prints the four lines in the documented shape; Docker Desktop's CLI is at `Contents/Resources/bin/docker` in
    its bundle and its context is `desktop-linux`; `Docker.raw`'s place; OrbStack's disk file
    (`~/Library/Group Containers/HUAQ24HBR6.dev.orbstack/data/data.img`) and whether its length is a limit (agent's
    call: hidden); whether a runtime's CLI starts it when it is stopped; how each runtime gives freed space back (the
    two `gives-back` sentences); `docker volume prune` removing only unnamed volumes on the installed Docker version;
    whether `Reclaimable` of images and build cache matches what `--all` frees.
  - **Calls worth a second look**: the generic `Insight` in the core (call 8); only runtimes with an app (call 2); the
    four commands and `--context` (call 3); no storage-map change (call 5); nothing asked in a restored scan (call 6);
    sandbox mode running no program but `/usr/bin/open` (call 9); the parts' levels (call 12).
  - **Known gaps**: a UI that connects after a restore shows no insight until the next scan; `npm run sandbox -- all`
    leaves three of Docker's case files out (the runtimes' fixed paths); the other runtimes deferred.

## Runs

- `npm run pre-push` on the last code commit; the numbers into the conclusions.
- `npm run sandbox:test`; `npm run test:e2e -- --project chromium --project webkit`.
- `npm run dev:sandbox -- docker` (and the three others): the agent walks what it can without a display and writes
  what is left for the user; nothing on the developer's real Mac is run beyond what `npm run dev` does by itself (no
  `docker` command by hand).

## Done when

- Every file above written; `npm run lint` (markdown and prettier) passes.
- One commit: `docs(docs): plan 032 built — docs, conclusions and the review list`.

## Conclusions

Written: the architecture, testing, manual-testing (1.19, 2.17), categories, deferred (sandbox-world, the new docker-other-runtimes, the user's review) and ideas docs, the statuses, and the plan's conclusions ([`../readme.md`](../readme.md)). Runs: `npm run pre-push`, `npm run sandbox:test`, the core e2e green; Playwright from step 05/02 (no UI change since). The sandboxes by hand and the dry run on the Mac are the user's, listed in the review file; the agent ran no `docker` command and opened no display.
