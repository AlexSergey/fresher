# 034/02/01 — The case files

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the categories ship their case files, negatives first, with their
golden reports and left-out expectations. Two files, since an open app is shared by every case of a file (as
`app-caches-app-open.toml`). The design: [`../00-design.md`](../00-design.md), section 7.

## Context for the implementer

- Read `fixtures/sandboxes/app-caches.toml`, `app-caches-app-open.toml` and `downloads-browser-open.toml` (the
  syntax: `[[case]]`, `kind`, `[[case.app]]` with `path`, `bundle-id`, `name`, `running`, `[[case.file]]` with
  `fill = "data"`, `[[case.symlink]]`, `[case.map]`, `[[case.left-out]]`, `[case.expect]` with `after-cleanup =
  "emptied"`), `docs/testing.md` ("Sandboxes") and `app/tauri/tests/sandboxes.rs`.
- The sandbox's apps declare no helpers: a helper's case is the fixtures' (01/03) and the core e2e's (02/02).
- **The shared world** (`npm run sandbox -- all` writes every file's cases into one sandbox): Google Chrome is open in
  `downloads-browser-open.toml`, so here Chrome appears only as the open browser, Chromium is the closed browser,
  Firefox is never installed (its case is the not-installed one), and Redis Insight is installed and closed in both
  files. Every path is apart from every other case file's.
- Each folder is written by one case only. A case that differs from the design is fixed in the module, not in the case
  file.

## Files

- `fixtures/sandboxes/app-caches-browsers.toml` (new) — `feature = "app-caches-browsers"`, `module = "app-caches"`; a
  head comment: what the file covers (the table's rows: a browser's caches per profile, a web-engine app's caches; only
  the names the table lists; emptied, the folder stays), the shared-world rule above, files small with
  `fill = "data"`. Cases, with these exact names:
  - negative, in this order:
    - "a browser's cache with no installed browser is never claimed" —
      `home/Library/Caches/Firefox/Profiles/abcd1234.default-release/cache2/entries/x`, no app;
    - "what a profile keeps beside its caches is never claimed" — `home/Library/Caches/Chromium/Profile 2/image_cache/x`
      and `home/Library/Caches/Chromium/Profile 2/Storage/ext/x`, Chromium installed (`Applications/Chromium.app`,
      `org.chromium.Chromium`, `Chromium`); `[case.map]` `your-folders` at `home/Library/Caches/Chromium/Profile 2`;
    - "a web-engine app's own data is never claimed" — `home/Library/Application Support/RedisInsight/Cookies`,
      `home/Library/Application Support/RedisInsight/Local Storage/leveldb/000003.log` and
      `home/Library/Application Support/RedisInsight/config.json`, Redis Insight installed
      (`Applications/Redis Insight.app`, `org.RedisLabs.RedisInsight-V2`, `Redis Insight`);
    - "a cache-named folder of an app not in the table is never claimed" —
      `home/Library/Application Support/OtherChat/Cache/x`, `OtherChat` installed (`com.example.OtherChat`);
    - "a link named like a cache is never claimed" — `[[case.symlink]]`
      `home/Library/Caches/Chromium/Profile 3/Cache` → `../../../../Projects/kept-cache`, a file
      `home/Projects/kept-cache/x`; after the cleanup the link and its target are unchanged;
  - positive:
    - "a browser profile's cache is emptied" — `home/Library/Caches/Chromium/Default/Cache/Cache_Data/data_0`;
      `home/Library/Caches/Chromium/Default/Cache` in `app-caches.browsers`, low, `emptied`; `[case.map]` `explained`
      at that folder;
    - "a browser profile's code cache is emptied" — `home/Library/Caches/Chromium/Default/Code Cache/js/index`; low,
      `emptied`;
    - "a second profile's cache is its own item" — `home/Library/Caches/Chromium/Profile 1/Cache/Cache_Data/data_0`;
      low, `emptied`;
    - "a web-engine app's cache is emptied" — `home/Library/Application Support/RedisInsight/Cache/Cache_Data/data_0`;
      `home/Library/Application Support/RedisInsight/Cache` in `app-caches.web-engine`, low, `emptied`.
- `fixtures/sandboxes/app-caches-browsers-open.toml` (new) — one `[[case.app]]` with `running = true` for Google
  Chrome (`Applications/Google Chrome.app`, `com.google.Chrome`, `Google Chrome`): negative "an open browser's caches
  are kept" — `home/Library/Caches/Google/Chrome/Default/Cache/Cache_Data/data_0` and
  `home/Library/Caches/Google/Chrome/Default/Code Cache/js/index`; left out `app-caches.left-out.app-open` 2 in
  `app-caches.browsers`; positive "a web-engine app's cache still goes while a browser is open" —
  `home/Library/Application Support/RedisInsight/GPUCache/data_0`, Redis Insight installed; low, `emptied`.
- `fixtures/golden/sandbox-app-caches-browsers.json`, `sandbox-app-caches-browsers-open.json` (new) —
  `UPDATE_GOLDEN=1`, reviewed: the findings with their apps, the left-out counts, the map, the `Emptied` outcomes, the
  tree after (each emptied folder still there, its entries in the sandbox's Trash).
- `app/tauri/tests/sandboxes.rs` — `app_caches_browsers => "app-caches-browsers"` and
  `app_caches_browsers_open => "app-caches-browsers-open"` in `case_files!`, sorted by file name (after
  `app_caches_app_open`).
- `docs/testing.md` — the two case files in the list of case files (after `app-caches.toml` and
  `app-caches-app-open.toml`), one paragraph in the same style: what each negative and positive shows and the
  shared-world rule.

## Produces

The case files, for 03/01 (the walks listed for the user).

## Checks

- `npm run sandbox:test` passes for both files and every other case file (their goldens unchanged).
- `npm run sandbox -- all` leaves out no file it did not already leave out; the result is written in this step's
  conclusions. If a contradiction is reported, the names here change, not the other file.
- Once per kind, by hand, written in this step's conclusions and reverted: a danger changed (`low` → `safe` on "a
  browser profile's cache is emptied") and a left-out's items changed (`2` → `1` in the open file) each make the
  autotest fail.
- `npm run sandbox -- app-caches-browsers` writes the sandbox; `EXPECTED.md` lists every case, negatives first, each
  negative "never offered by `app-caches`".
- `cargo nextest run -p fresher-app --test sandboxes`.

## Conclusions

- `cargo nextest run -p fresher-app --test sandboxes`: every case file passes, the two new ones included; the other
  goldens are unchanged.
- `npm run sandbox -- all` writes 214 cases and leaves out the same 5 files it left out before this step; the two new
  files leave out nothing.
- The hand mutations, each reverted: `low` → `safe` on "a browser profile's cache is emptied" fails
  `positive::app_caches_browsers`; left-out items `2` → `1` in the open file fails
  `positive::app_caches_browsers_open`.
- `npm run sandbox -- app-caches-browsers` writes the sandbox; `EXPECTED.md` lists the 5 negatives first, each "never
  offered by `app-caches`".
- One change from this step's text: the open file installs Redis Insight at `Applications/RedisInsight.app`, not
  `Applications/Redis Insight.app`. With the same path in both files, `all` left out the open file whole, since it
  wrote a path the first file had already written. The bundle id and the name are unchanged, so the cases are the
  same; the file's head comment and `docs/testing.md` say so.
- The tree after the cleanup lists `home/.Trash/Cache_Data` three times: the sandbox's Trash keeps same-named entries
  apart by an order prefix (`<n>-Cache_Data`), and the report drops that prefix so the golden does not depend on the
  order of the moves. Expected, not a defect.
