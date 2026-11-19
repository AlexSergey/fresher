# 026/02/03 — The launcher and the docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** scripts, app, docs

`npm run dev` becomes a launcher that names the record's folder and the build's commit, so every development run
started by hand is recorded and a bare `tauri dev` or a test is not. `npm run dev:sandbox` keeps calling `npm run
dev`, so a sandbox run is recorded too. The docs say what a development build now does on the real Mac and where its
records go, and the two deferred items say what this plan gives them. The design:
[`../00-design.md`](../00-design.md), sections 2.2 and 3 (agent's calls 14 and 15).

## Files

- `scripts/dev.ts` (new) — in the shape of `scripts/sandbox-dev.ts`, reusing its `Command`, `Spawner` and `run`
  (imported from `./sandbox-dev.ts`); the header comment: "`npm run dev [-- <tauri dev args>]` (plan 026/02/03): starts
  the development build with the development run record on: `FRESHER_DEV_RUNS`, `FRESHER_BUILD_COMMIT`,
  `FRESHER_BUILD_DIRTY`. The app checks the folder (app/tauri/src/dev_runs/)."
  - `export type Git = (args: readonly string[]) => { readonly status: number; readonly stdout: string };`
  - `export type BuildFacts = { readonly commit: string; readonly dirty: boolean };`
  - `export const buildFacts = (git: Git): BuildFacts`: `git rev-parse --short HEAD` (its first line, trimmed; a
    non-zero status gives `unknown`) and `git status --porcelain` (dirty when its output is not empty after trimming; a
    non-zero status gives clean);
  - `export const devCommand = (root: string, facts: BuildFacts, argv: readonly string[]): Command`: `npm exec --
    tauri dev --config app/tauri/tauri.dev.conf.json` (fixed arguments, no `argv`) and the env `FRESHER_DEV_RUNS: join(root, 'NO_COMMIT',
    'dev-runs')`, `FRESHER_BUILD_COMMIT: facts.commit`, `FRESHER_BUILD_DIRTY: facts.dirty ? '1' : '0'`;
  - the main block under `import.meta.main`: the root from `new URL('..', import.meta.url)`, the real `Git` through
    `spawnSync('git', args, { cwd: root, encoding: 'utf8' })`, `process.exitCode = run([command], spawnReal)` (the
    spawner as `sandbox-dev.ts`'s, exported from there if it is not yet).
  It creates no folder and writes nothing: the app does.
- `scripts/dev.test.ts` (new) — `node:test`, as `scripts/sandbox-dev.test.ts`.
- `package.json` — `"dev": "node scripts/dev.ts"`, no `dev:app` script (the launcher calls the tauri CLI directly).
  `scripts/sandbox-dev.ts` is unchanged (it runs `npm run dev`); its header comment says the run is recorded.
- `docs/testing.md` —
  - "Sandbox mode": "starts the development build (`npm run dev`)" stays true; one sentence: the run is recorded as
    `sandbox-<name>` (below).
  - A new section "### The development run record" after "Sandbox mode": what `npm run dev` sets; the file names; the
    shape in one paragraph with a link to `plans/026-dev-run-record/00-design.md`, section 2.5; never overwritten; only
    debug builds; a bare `tauri dev` and every test write nothing; a development build on the real
    Mac makes every cleanup a dry run (`architecture/removal.md`).
  - The commands table: `npm run dev` ("the development build on the real Mac, recorded; every cleanup a dry run") .
- `docs/restrictions/privacy.md` — "Local data only", a second rule: "A development build started by `npm run dev`
  writes a record of each run (real paths of the Mac, never file contents) to `NO_COMMIT/dev-runs/` of the repository,
  which git ignores; a release build has no such code. Enforced by: `#[cfg(debug_assertions)]` on `app/tauri/src/
  dev_runs/`, the variable set only by `scripts/dev.ts`, `.gitignore`, the tests of `compose.rs`
  (`a_release_build_ignores_the_dev_runs_variable`)."
- `docs/architecture/README.md` — "Rules of the layout", after the sandbox mode bullet: "**The development run
  record** (plan 026, linked to `../plans/026-dev-run-record/readme.md`, debug builds only, `app/tauri/src/dev_runs/`): when
  `FRESHER_DEV_RUNS` names a folder (set by `npm run dev`), the composition begins a run; the app subscribes it to the
  engine before the last scan is restored and wraps the generated invoke handler, so every action the UI sends and
  every event but the scan's progress go to an append-only `.events.jsonl`, and a snapshot `.json` is rewritten after
  each scan and cleanup. The core, the engine and the modules know nothing of it."
- `docs/deferred/session-log-file.md` — "What doing it needs" gains a fifth point: "Plan 026 built a development-only
  record (`app/tauri/src/dev_runs/`): a new file per run never overwritten, an append-only event file, an atomic
  snapshot, a core text in English. The opt-in log may reuse that writer; its privacy decisions (the home shortened to
  `~`, no path outside it, the location) stay this item's."
- `docs/deferred/dev-debug-panel.md` — "What is postponed": "print a dry run" is answered on the real Mac since plan
  026 (a development build's cleanups are dry runs, recorded and checked by `npm run dev-runs:check`); the panel and
  the rest of the command line stay postponed.

## Produces

`npm run dev` with the record on, for 03/03's run by hand.

## Checks

- `scripts/dev.test.ts`, negatives first:
  - `describe('negative cases')`: `a_failing_git_gives_unknown_and_clean`; `a_bare_dev_app_is_not_this_launcher`
    (the command is `npm exec -- tauri dev`, never `npm run dev`, so it cannot call itself);
  - `describe('positive cases')`: `the_record_folder_is_inside_the_repository` (`/repo/NO_COMMIT/dev-runs`);
    `the_commit_is_the_trimmed_short_head` (`"26e4cb0\n"` → `26e4cb0`); `a_changed_tree_is_dirty_and_a_clean_one_is_not`
    (`" M a\n"` → `1`, `""` → `0`); `extra_arguments_are_refused` (`['--release']` and `['--foo']` give a `UsageError`).
- `npm run test:scripts`; `npm run lint:ts`; `npx eslint scripts/dev.ts scripts/dev.test.ts`; `npm run knip`;
  `npx prettier --check` on the changed files.
- By hand: `npm run dev` starts the window; `ls NO_COMMIT/dev-runs` shows one pair named with this commit (and
  `-dirty` when the tree is); quit. `npm run dev:sandbox -- trial` writes a pair ending `_sandbox-trial`.

## Conclusions

Not written yet.
