# 021/03/02 — The core e2e flows

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** e2e, fixtures, docs

The module's flows on the real scanner, engine, settings, cache and journal files and the real `DiskCleaner` over a
fixture home: the projects found in the shared walk, their facts asked of the real scanner's tree after the walk, a
rescan that sees a change in a project, a restore at launch. The design: [`../00-design.md`](../00-design.md),
section 4, "Core e2e".

## Files

- `fixtures/homes/stale-projects-flow.toml` (new) — a home whose every project folder is a dated `[[dir]]`: under
  `Code/`, an active Node project (`active/`, 3 days, `node_modules/`, `src/`, `.git/refs/heads/`); a project with old
  folders and a recent commit (`committed/`); stale (400 days) `shop/` (a monorepo: `node_modules/` and
  `packages/web/node_modules/`), `tool/` (Rust: `target/debug/incremental/…`, `src/`, `.git/`), `py/` (`.venv/`),
  `site/` (Node: `node_modules/`, `dist/`); a 300-day-old `older/` (Node, `node_modules/`); an active Rust `daily/` (3
  days) with `target/debug/incremental/` 10 days old and `target/llvm-cov-target/` 3 days old; and `Documents/keep.txt`
  outside every project.
- `fixtures/golden/stale-projects-flow.json` (new) — the first scan's findings and left-outs, `UPDATE_GOLDEN=1`,
  reviewed.
- `crates/e2e/tests/stale_projects.rs` (new) and its harness `crates/e2e/tests/stale_projects_home/mod.rs` (new), on
  the shared `support/home.rs` (as `downloads_home`): the `stale-projects` module registered beside `jetbrains`,
  `temp`, `dev-caches`, `app-leftovers` and `downloads` (as the app registers them); `FakeClock`, `FakeTrash`,
  `FakeOpenFiles`, `FakeChangeJournal`, the home's stated facts. The flows, negatives first within each:
  1. `nothing_of_an_active_project_is_offered` — `active/`'s `node_modules` left out `recent`; nothing of `active/`
     offered; `src`, `.git` untouched after a cleanup of everything offered.
  2. `a_recent_commit_keeps_a_project_active` — `committed/`'s `node_modules` left out `recent`.
  3. `the_first_scan_offers_every_category` — the golden matches: `shop · node_modules` and `shop ·
     packages/web/node_modules`, `tool · target` (one finding: its incremental folder not offered again), `py · .venv`
     at Medium, `site · node_modules` (never `site/dist`), `daily · target/debug/incremental`; `daily`'s
     `llvm-cov-target` left out `cache-recent` and its `target` `recent` with its sizes less the offered cache's.
  4. `scan_confirm_trash_and_put_back` — every offered item selected: the plan's highest danger Medium; every item in
     the Trash and the journal; put back restores them byte for byte; every `src`, `.git`, `dist`, marker file and
     `Documents/keep.txt` untouched.
  5. `a_change_in_a_stale_project_keeps_it` — `site/` offered; a file added to `site/src` (its folder's time now);
     `FakeChangeJournal` reports `site/src`; a rescan (asserted `ScanKind::Rescan`) leaves `site`'s `node_modules`
     `recent`.
  6. `the_setting_moves_the_days` — `older/` (300 days) `recent` at the default 365; `stale-projects.unused-for-days`
     set to 200; the next scan offers it.
  7. `a_restore_at_launch_offers_nothing` — after a scan, a restore (`ScanKind::Restored`): every item the scan offered
     is left out `scan-again`, nothing offered.
  8. `the_storage_map_explains_the_artifacts_and_keeps_the_rest_open` — an offered `node_modules` in the explained
     part; `tool/src` in the user's folders; the browse view opens `tool/` and `daily/target/` (their children listed:
     nothing of this plan is claimed whole).
  9. `ignoring_the_build_output_keeps_it_out_of_every_total` — `stale-projects.build-output` ignored: its findings in
     no total, the others stay; restore brings it back.
  10. `a_stale_project_cleaned_once_looks_active_after` — `shop`'s `node_modules` moved to the Trash alone; the next
      scan (a rescan, the journal reporting `shop`) leaves `packages/web/node_modules` `recent` (the removal moved
      `shop`'s folder time; the design's risk, asserted so it stays known).
- `crates/e2e/Cargo.toml` — `fresher-module-stale-projects` as a dev-dependency.
- `docs/testing.md` — `tests/stale_projects.rs` in the e2e list, as `downloads.rs` is described.

## Produces

The flows, for the acceptance's "nothing claimed whole", "a commit keeps a project" and "a restore offers nothing".

## Checks

- `cargo nextest run -p fresher-e2e --test stale_projects` (the ten flows); the other e2e binaries unchanged
  (`cargo nextest run -p fresher-e2e`).
- `npm run test:e2e:core`; clippy (`--all-targets`); fmt.

## Conclusions

Ten flows on the real scanner, engine, settings, cache, journal and cleaner over `stale-projects-flow.toml`; the golden
is `stale-projects-flow.json`. Every folder is dated, `.git/refs/heads` included (preflight C10). `Documents/build` holds
a folder `drafts/notes.txt`, because the browse view lists folders only (a file child is not a row); flow 8 opens `tool/`,
`tool/target/debug/incremental`, `daily/target/` and `Documents/build` (C11). Node-modules sizes differ per project
(4096, 8192, 16384) so the left-out sums name which projects were kept. A restore offers nothing and leaves every item
`scan-again`; the module's `on_match` and `finish` meeting the same cancel token is proven by the offers appearing in the
first scan of a real engine run.
