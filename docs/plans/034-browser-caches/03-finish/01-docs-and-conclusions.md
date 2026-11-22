# 034/03/01 — Docs, review list and conclusions

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Every doc the plan's change affects, as built (a doc a step already updated is checked, not rewritten); the deferred
item narrowed; the section of this plan in the user's review list; the conclusions. The runs by hand are the user's:
this step lists them, it does not tick them.

## Context for the implementer

- Read the plan's readme, `00-design.md`, every step's file and the commits of the branch
  (`git log $(git merge-base HEAD main)..HEAD`), then the docs below. English only; never name another disk-cleaner
  product or a reference project; apps whose caches are cleaned may be named.
- Plan 030's docs are the model: `docs/plans/030-installed-app-caches/03-finish/` and the sections of
  `docs/manual-testing.md` (1.13, 2.12) and `docs/architecture/modules.md` it wrote.

## Files

- `docs/architecture/modules.md` — in the `app-caches` paragraph: the second table, `apps.toml`, ties a place named
  after an app to the app by bundle id and names the exact cache folders (rules as data); a `*` in it stands for a
  profile; a match of a table row finds its owner by the row's id, the rest of the decision is the same rows; one
  finding per cache folder, emptied. The deferred link now names what stays deferred.
- `docs/architecture/README.md` — the `app-caches` entry of the crate list: "and the cache folders a table ties to an
  installed browser or web-engine app".
- `docs/categories/app-caches-browsers.md`, `app-caches-web-engine.md` — as built (the rows, the texts, the decision);
  their rows in `docs/categories/README.md` checked; in the danger table, "a browser's cache" among the Low examples.
- `docs/testing.md` — the case files (02/01), `app_caches_browsers.rs` (02/02), the UI scan (02/03): checked.
- `docs/manual-testing.md` — two items, in the style of 1.13 and 2.12, linked to this step: "1.x Browser and
  web-engine caches" (the sandbox walks below) and "2.x Browser and web-engine caches" (the dry run below); the
  step's row in the table of runs, `pending`.
- `docs/ideas/installed-app-caches.md` — its line on plan 034: what was built, what is still deferred.
- `docs/deferred/installed-app-caches-more-places.md` — narrowed: the rows "Web-engine caches inside
  `~/Library/Application Support/<name>/`" and "Browsers' caches" now say the table exists (plan 034) and what remains:
  rows for apps not checked yet, the browsers' support-folder graphics caches (protected without Full Disk Access),
  `Service Worker/CacheStorage` and `CachedData` (names with a path or a meaning not yet checked). Its row in
  `docs/deferred/README.md` updated in the same words.
- `docs/deferred/user-review-2026-11-22.md` — create it if absent (title "What to review after the session of
  2026-11-22", a one-paragraph head: what the file is, one section per plan, short bullets, removed when the user has
  looked) with its row in `docs/deferred/README.md` (agreed 2026-11-22, due "the user's review of 2026-11-23"); append
  otherwise. The section `## 034 — Browser and web-engine caches` (link to the plan), short bullets:
  - **Walks by hand**: `rm -rf sandbox/app-caches-browsers && npm run dev:sandbox -- app-caches-browsers` (walk
    `EXPECTED.md`: no negative offered; Chromium's three caches and Redis Insight's `Cache` offered at Low, titled
    `Chromium, Default: Cache` and the like; the storage map shows `Profile 2` in your folders and the caches in
    Chromium's line; select all, confirm to the Trash, each cache folder still there and empty, put back restores);
    `npm run dev:sandbox -- app-caches-browsers-open` (Chrome's two caches kept with "This app, or a part of it, is
    open…"; Redis Insight's `GPUCache` offered).
  - **Dry run on the real Mac** (`npm run dev`, a development build is a dry run): with Chrome, Chromium, Firefox
    closed, each offered cache checked against `du -sh ~/Library/Caches/Google/Chrome/*/{Cache,"Code Cache"}
    ~/Library/Caches/Chromium/*/{Cache,"Code Cache"} ~/Library/Caches/Firefox/Profiles/*/{cache2,startupCache}`; with
    Chrome open, its caches left out; nothing under `Application Support/Google`, `.../Firefox` or `.../Chromium`
    offered; `~/Library/Caches/Google/Chrome-headless` not offered.
  - **Facts not verified here**: Redis Insight's caches with the app open and closed (not run on this Mac); whether a
    browser accepts its emptied `Cache` folder on the next start without a warning (expected: yes; check Chrome and
    Firefox once after a real cleanup in a release build); Firefox's `startupCache` emptied while the profile is
    closed.
  - **Candidate rows to check on a real install** (each needs the bundle id from `Info.plist`, the folders with `ls`,
    then a row): Microsoft Edge (`com.microsoft.edgemac`, expected `~/Library/Caches/Microsoft Edge/<profile>/`),
    Brave (`com.brave.Browser`, `~/Library/Caches/BraveSoftware/Brave-Browser/<profile>/`), Vivaldi, Arc; Visual
    Studio Code (`com.microsoft.VSCode`, `~/Library/Application Support/Code/`), Slack, Discord, Cursor; all
    expected, none checked.
  - **Agent's calls worth a second look**: one finding per cache folder rather than per profile (design call 5); only
    the four checked rows ship (call 4); no browser support-folder caches (call 6); Firefox's `startupCache` in,
    `safebrowsing` and `thumbnails` out (call 7); the title shows the raw folder name (`Code Cache`, `cache2`) (call
    8); the calls taken while building (from the conclusions).
  - **Known gaps**: whatever the conclusions list (a flake, a skipped check).
- `docs/plans/034-browser-caches/` — each step's, each plan's and the global Conclusions written as built (what was
  built, what changed from the design and why, the numbers: tests, coverage of the crate and of the workspace, the
  `sandbox -- all` result, the by-hand mutation checks of 02/01, the calls taken while building); statuses 🔨 in
  progress "awaiting the user's walk" on the global readme, the plans and this step; the steps that are fully built
  ✔️ only if nothing of theirs waits for the user.
- `docs/plans/README.md` — the row of 034: 🔨 in progress.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves.
- `npm run pre-push` on the branch's last commit (its numbers go into the conclusions);
  `git diff $(git merge-base HEAD main) -- crates/core crates/scanner crates/cleaner crates/engine` empty.

## Conclusions

Built: the architecture (`modules.md`, the crate list), the categories (two files as built, their rows, "a browser's
cache" among the Low examples), `testing.md` (checked: the case files, `app_caches_browsers.rs`, the UI scan),
`manual-testing.md` (items 1.16 and 2.14 and the step's row, `pending`), the idea, the deferred item narrowed and its
row, and the review list `docs/deferred/user-review-2026-11-22.md` with this plan's section. The runs by hand are the
user's and are listed, not ticked. `git diff` of `crates/core`, `crates/scanner`, `crates/cleaner` and `crates/engine`
against the merge base is empty. `npm run pre-push` on the branch passed: 3887 Rust tests passed, workspace line coverage 97.54 %, UI line coverage 98.60 %.
