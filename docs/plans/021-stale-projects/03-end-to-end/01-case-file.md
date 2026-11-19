# 021/03/01 — The case file

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case file, negatives first, with its golden
report and left-out expectations. One file: the module needs no world of its own (no access, no running app, no
Spotlight), only dated folders, which `[[case.dir]]` already writes. The design:
[`../00-design.md`](../00-design.md), section 4, "Sandbox".

## Files

- `fixtures/sandboxes/stale-projects.toml` (new) — `feature = "stale-projects"`, `module = "stale-projects"`; a head
  comment: what the file covers; that **every folder of a project is a `[[case.dir]]` with an `age`** (a folder the
  generator makes is "changed now": a stale case that forgets one turns active and fails on its missing offer), the
  parents of files and `.git/refs/heads` included; that projects live under `home/Code/<case's own name>/` so no path
  is written twice; that positives' files are small and `fill = "data"`. Ages `"400d"` for an old project, `"3d"` for
  an active one, `"10d"` for a cache unused for a week. With these exact names:
  - negative, in this order:
    - "the source, .git and the project folder are never claimed" — `home/Code/kept-source/` (`package.json`,
      `src/index.ts`, `.git/HEAD`, `.git/refs/heads/main`, every folder 400 days) without `node_modules`;
      `[case.map]` `your-folders` at `home/Code/kept-source/src`;
    - "dist and .next are never claimed" — `home/Code/framework/` (`package.json`, `dist/app.js`, `.next/cache/x`,
      400 days) without `node_modules`;
    - "a node_modules without a project is never claimed" — `home/Code/loose/node_modules/x/index.js` (400 days);
    - "a link named node_modules is never claimed" — `home/Code/linked/package.json` (400 days),
      `home/Code/linked/node_modules` → `home/elsewhere/modules` (holding `x`); the link and its target unchanged
      after the cleanup;
    - "a node_modules in the home itself is never claimed" — `home/package.json` and `home/node_modules/left-pad/x`
      (400 days);
    - "a project in a hidden folder of the home is never claimed" — `home/.config/tool/package.json` and
      `home/.config/tool/node_modules/x` (400 days);
    - "a project inside an app bundle is never claimed" — `home/Code/Tool.app/Contents/Resources/app/package.json`
      and its `node_modules/x` (400 days: an app kept beside projects, in a walked folder);
    - "an active project's packages are kept" — `home/Code/active/` (`package.json`, `node_modules/x`, 3 days); left
      out `stale-projects.left-out.recent` 1 in `stale-projects.dependencies`;
    - "a project with a recent commit is kept" — `home/Code/committed/` (every folder 400 days, `.git/refs/heads` 3
      days, `node_modules/x`); left out `recent` 1 in `stale-projects.dependencies`;
    - "a Rust build cache used this week is kept" — `home/Code/building/` (`Cargo.toml`, `src/main.rs`, folders 3
      days; `target/debug/incremental/x-1/s-1/w` 3 days); left out `stale-projects.left-out.cache-recent` 1 in
      `stale-projects.rust-build-cache` and `recent` 1 in `stale-projects.build-output` (its `target`);
  - positive:
    - "an old Node project's packages go" — `home/Code/old-web/` (`package.json`, `src/index.ts`, `node_modules/x`,
      400 days); `home/Code/old-web/node_modules` in `stale-projects.dependencies`, low, `in-trash`; `[case.map]`
      `explained`;
    - "an old Rust project's target goes" — `home/Code/old-tool/` (`Cargo.toml`, `src/main.rs`, `target/debug/x`,
      400 days); `home/Code/old-tool/target` in `stale-projects.build-output`, low, `in-trash`;
    - "an old Gradle project's build goes" — `home/Code/old-android/` (`build.gradle.kts`, `build/x`, 400 days);
      `home/Code/old-android/build` in `stale-projects.build-output`, low, `in-trash`;
    - "an old Python project's environment goes, at Medium" — `home/Code/old-py/` (`pyproject.toml`, `.venv/x`, 400
      days); `home/Code/old-py/.venv` in `stale-projects.python-environments`, medium, `in-trash`;
    - "a monorepo's package goes under the monorepo's name" — `home/Code/shop/` (`package.json`,
      `packages/web/package.json`, `packages/web/node_modules/x`, 400 days, the monorepo's own `node_modules` absent);
      `home/Code/shop/packages/web/node_modules` in `stale-projects.dependencies`, low, `in-trash`;
    - "an active Rust project's incremental cache goes" — `home/Code/daily/` (`Cargo.toml`, `src/main.rs`, folders 3
      days; `target/debug/incremental/x-1/s-1/w` 10 days); `home/Code/daily/target/debug/incremental` in
      `stale-projects.rust-build-cache`, low, `in-trash`;
    - "an active Rust project's coverage build goes" — `home/Code/covered/` (`Cargo.toml`, `src/main.rs`, folders 3
      days; `target/llvm-cov-target/debug/x` 10 days); `home/Code/covered/target/llvm-cov-target` in
      `stale-projects.rust-build-cache`, low, `in-trash`.
  The two active Rust positives also leave their `target` out `recent` (1 each in `stale-projects.build-output`):
  written as left-out expectations of their cases.
- `fixtures/golden/sandbox-stale-projects.json` (new) — `UPDATE_GOLDEN=1`, reviewed: the findings, the left-out
  counts, the map, the cleanup's outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — `stale_projects => "stale-projects"` in `case_files!`, sorted by file name.
- `docs/testing.md` — the case file in the list of case files, as `downloads`' are described, with the line that every
  project folder is dated.

What the code does for each case is written down when the file is made; a case that differs from the design is fixed
in the module, not in the case file.

## Produces

The module's case file, for 04/02 (the run by hand).

## Checks

- `npm run sandbox:test` passes for the file and every other case file (their goldens unchanged).
- Once per kind, by hand, written in this step's conclusions: a danger changed (`medium` → `low` on the Python case)
  and a left-out's items changed (`1` → `2` on the commit case) each make the autotest fail; one folder's `age` removed
  from the old Node case makes it fail (offered nothing); then reverted.
- `npm run sandbox -- stale-projects` writes the sandbox; `EXPECTED.md` lists every case, negatives first.
- `cargo nextest run -p fresher-sandbox -p fresher-app`.

## Conclusions

The case file has the 17 cases of the design, negatives first, with the exact names. Two findings of the harness shaped
it: all cases share one world, so a folder is written by one case only; and a negative owns every folder above its
paths, so `home/Code` is dated once, in the first positive (no negative may own the folder holding another case's
offer). Every other folder of every project is a dated `[[case.dir]]`; the link's target is `../../elsewhere/modules`,
and the old Node case carries `.git/refs/heads/main` dated 400 days. The map's `explained` of that case names
`home/Code/old-web/node_modules`, since the default checks the case's first path (`.git`, never explained).

Checked by hand, once per kind, then reverted: `medium` -> `low` on the Python case ("the danger Low is below the
category's Medium"); `1` -> `2` on the commit case (the left-out counts differ); one folder's `age` removed from the old
Node case (the autotest fails: nothing offered). `npm run sandbox:test` and `cargo nextest run -p fresher-app -p
fresher-sandbox` are green (297 tests); `npm run sandbox -- stale-projects` writes 17 cases and `EXPECTED.md` lists them,
negatives first.
