# 021/04/02 — The runs by hand and the conclusions

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs, modules

A checklist for the user, then the conclusions.

## In the sandbox

- [ ] `rm -rf sandbox/stale-projects && npm run dev:sandbox -- stale-projects`. Scan; walk `EXPECTED.md` line by line,
  negatives first: nothing of a negative is offered (the source, `.git`, `dist`, `.next`, the loose `node_modules`, the
  link, the home's own `node_modules`, the hidden folder's and the app bundle's projects); the left-out lines read as
  written (the project changed lately, a build used it this week); each positive is offered in its category at its
  danger (packages and build output Low, the Python environment Medium, the Rust caches Low); the monorepo's package
  reads "shop · packages/web/node_modules"; each reason names a date and a command.
- [ ] The storage map: the offered folders in the explained part; `home/Code/kept-source/src` in your folders; the
  browse view opens `home/Code/old-tool/target` (not one opaque entry).
- [ ] Select everything; Review names Medium; confirm to the Trash: the items land in `home/.Trash`; Cleanups → put
  back: they are back. The link's target in `home/elsewhere/modules` is untouched.
- [ ] Ignore the build-output category, then the module; restore.

The check that a changed expectation fails the autotest, once per kind (a danger, a left-out's items, a folder's age),
was made while building (03/01's conclusions).

## On the developer's Mac (read-only)

`npm run dev`, a scan of `~` only (a development build's cleaner reaches only a sandbox, so nothing can be removed).
Compare with these read-only commands (`<project>` is one the scan offers or keeps):

```sh
find <project> \( -name node_modules -o -name target -o -name .git -o -name build -o -name .venv \) -prune \
  -o -type d -print0 | xargs -0 stat -f "%m %N" | sort -n | tail -3
find <project>/.git/refs/heads -type d -print0 | xargs -0 stat -f "%Sm %N"
stat -f "%Sm %N" <project>/.git/logs/HEAD
du -sh <project>/node_modules <project>/target 2>/dev/null
ls target/debug target/llvm-cov-target 2>/dev/null; du -sh target/*/incremental target/llvm-cov-target 2>/dev/null
```

- [ ] Every offered item checked by hand: its project's newest folder and its `refs/heads` are older than the days;
  the folder is the project's `node_modules`, `target`, `build`, `.gradle`, `.build`, `.dart_tool`, `.venv` or `venv`;
  nothing under `~/Library`, an app, a hidden folder of the home or the home itself; no `src`, no `.git`, no `dist`.
- [ ] Every project kept `recent` that you think is old: why (which folder or branch is new); kept `unread`: which
  folder was not read.
- [ ] **This repository's `target/`** (built daily, covered on every push): what the cache category offers and keeps,
  and its sizes; `du -sh target/debug/{deps,incremental,build}` and `target/llvm-cov-target` today and again after a
  week of work, to say which part grows (the risk of decision 3.4; a later idea if `deps` is the part).
- [ ] The facts "to verify":
  - an editor that saves in place: open a small file of a project in each editor you use (a JetBrains IDE, Xcode and
    the others), change one line, save; `stat -f %Sm` the file's folder before and after. An editor that moves no
    folder is written in the category docs (agent's call 2);
  - `refs/heads` against `logs/HEAD` on your repositories, as on 2026-11-19 (13 of 15 the same day): a commit, a pull,
    a merge, a checkout of an existing branch — which move `refs/heads` (agent's call 1);
  - Finder browsing a project (a `.DS_Store` written): does it move the project's folder time;
  - a `node_modules` made of hard links into a package manager's shared store: the finding's reclaimable size against `du` (the scanner counts
    a hard-linked file's bytes once);
  - Cargo's incremental folders: a build that recompiles moves `target/debug/incremental`'s folder times (the cache's 7
    days).
  The table, texts and category docs are corrected where they differ.
- [ ] The scan's time against `main`'s on the same home (rule 4: within 5 %), and the `subtrees` bench group's time
  (`npm run bench`, on a quiet Mac).

## Files

- `docs/plans/021-stale-projects/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was
  built, what changed from the design and why, the numbers (tests, coverage, the bench, the scan's time, what the
  module offered on the developer's Mac by category, this repository's `target/` parts over a week), the calls taken
  while building, what the real Mac showed.
- `modules/stale-projects/artifacts.toml`, `locales/en.json`, `docs/categories/stale-projects-*.md` — corrected where
  the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Not written yet.
