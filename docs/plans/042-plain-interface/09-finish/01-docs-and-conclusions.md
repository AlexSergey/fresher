# 042/09/01 — The docs, the walk and the conclusions

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** docs, e2e, sandbox

Every doc the plan affects, as built (a doc a step already updated is checked, not rewritten); every baseline green;
the demo; the walk by hand for the user; the plan's conclusions; the user's review list. The design:
[`../00-design.md`](../00-design.md).

## Files

- `docs/architecture/README.md` — the High switch (where it is set, what it hides, the refusals) in "Data" or beside
  the cleanup's preconditions; "The storage map": the five-part summary beside the parts; "The largest files": hidden
  while High is withheld; "UI": the new screens' structure (Scan's disk card and groups, Review's tiles and the tick
  model with `toCheck`, Settings' sidebar, pages and search), the design pieces of plan 02, `AppInfo.high`.
- `docs/architecture/modules.md` — six texts and an icon key (01/04 wrote it; checked).
- `docs/restrictions/deletion-safety.md` — "High is withheld in the first release" (01/01 wrote it; checked against
  the code); "Explicit confirmation only" gains: a "Look before removing" item goes only when the user ticked it by
  hand (`Selection.toCheck`, `state/selection.ts` and its tests); "A High plan needs its own tick" notes it is unused
  while High is withheld.
- `docs/categories/README.md` — the "Shown as" column (01/05) and each category file's texts against its dictionary
  (08/01; checked with a grep of each heading against the dictionaries).
- `docs/testing.md` — the `high` key of a case file and `high-withheld.toml`; `high_tests.rs`, `high_withheld.rs`; the
  e2e mock's `high` option and `withheld` helper; the text check's new rules; the UI scenarios that replaced 039's
  ("The agreement and the danger zone in the interface" renamed and rewritten).
- `docs/writing.md` — checked against the script's constants (`LIMITS`, `BANNED`, the form rules).
- `docs/manual-testing.md` — a section `1.24 A plain interface` with the walk below and a link to this step; item 1.23
  (039's) marked where its Review and Scan question are replaced.
- `docs/mockups/README.md` — unchanged unless a mockup changed (none may without the user).
- `docs/plans/README.md` — the row of 042: 🔨 in progress (awaiting the user's walk), placed above 041.
- `docs/plans/042-plain-interface/` — every step's and plan's conclusions (written as built; checked); the global
  readme's and `00-design.md`'s **Conclusions**: what was built, the numbers (Rust and UI tests and coverage, the text
  check before and after the pass, texts changed per dictionary, baselines changed), what changed from the design and
  why, the calls taken while building; statuses 🔨 "awaiting the user's walk" on the global readme and on each plan and
  step.
- `docs/plans/039-agreement-and-wording/readme.md`, `docs/plans/027-scan-screen/readme.md`, `035`, `036` — one line
  each: what 042 replaced or hid.
- Demo recordings: run `npm run demo:check` if the development app can start here; if `npm run demo:record` works,
  record and commit the GIFs it writes (look at each); otherwise add a short item to
  `docs/deferred/demo-scenarios-for-new-screens.md` (or a new deferred file and its row) saying the recordings wait
  for the new screens' walk.
- `docs/deferred/user-review-2026-11-24.md` (new) and its row in `docs/deferred/README.md` — short bullets:
  - **Walks by hand**: the walk below; the dry run on the real Mac (`npm run dev`: no High anywhere, Largest absent).
  - **Rulings worth a second look**: the design's agent's calls 1 to 20, one line each, and every call taken while
    building.
  - **Texts**: the group names and lines, the five part names and their "i" texts, the category lines; texts whose
    meaning looked wrong (08/01, 08/02), listed, not changed.
  - **The logo**: the sidebar's mark, the app icon in the Dock and Finder, the social preview (upload it by hand on
    GitHub, as plan 040 says).
  - **Known gaps**: what a step reported as not done; the progress estimate on a first scan; the demo GIFs if
    deferred.

## The walk by hand (for the user)

Each from a fresh folder: `rm -rf sandbox/<name> && npm run dev:sandbox -- <name>`.

- [ ] `trial`: Scan before a scan (the disk card, Scan in the ring); scan: the swoosh fills, the percent, "Looking
  at …", the size found; after: the disk card's five parts, each "i", hover a row and its arc; the separator; the
  groups with their lines; the tiles; readable trial names.
- [ ] `stale-projects` (Medium items): tick a Look before removing tile on Scan; the bar says "to check in Review";
  Review: the tile's items unticked, the pill amber, the chip "not checked", Move to the Trash inactive; tick one: the
  summary grows; tick all: the pill turns green; move to the Trash; put back from Cleanups.
- [ ] `app-leftovers`: no data folders anywhere (High withheld); the caches tile; its details: three answers, who makes
  it, the left-out bar, the items with the app's name first; the menus' accent hover and their order.
- [ ] `uninstall`: an app's page offers only "Keep my settings"; no Largest in the sidebar.
- [ ] Settings: the tree opens and closes; a module's page with its switch last; the search "days": three results,
  marked; Enter opens the first with its setting focused; Esc clears; drag a slider, type a number out of bounds.
- [ ] The logo beside "Fresher"; the primary buttons' sheen; the app icon in the Dock.
- [ ] Read every screen once for the voice of `docs/writing.md`.

## Checks

- `npm run pre-push` on the branch's last commit; `npm run test:e2e` all green (every baseline looked at when it
  changed); `npm run sandbox:test`; `npm run lint`; `npx prettier --check` on the changed docs; every relative link of
  the changed files resolves.
- `git diff main -- crates/scanner crates/macos crates/system crates/store crates/rules crates/cleaner` is empty;
  `git diff main -- modules` touches only `src/lib.rs` (icons), `locales/en.json`, and the trial and demo modules'
  names, tests and fixtures.

Commit: `docs(docs): plan 042 — docs, walk and conclusions`.

## Conclusions

- First, the final review's last leak: an Apps row no longer shows a part of a High kind nor counts its bytes while
  High is withheld (`list::summary` takes the switch; tests in `high_tests.rs`, the Apps mock and `apps.spec.ts`).
- The docs as built: the architecture (the High switch, the storage map's summary, Largest hidden, the UI section
  rewritten for the new screens), deletion safety (the Look items rule, the High tick unused while withheld, what
  Withheld hides), testing (the High tests, the `withheld` helper and its limit, the text check's rules, the baseline
  note), manual testing 1.24, the notes in plans 027, 035, 036 and 039, the user's review list. `docs/categories`,
  `writing.md` and `modules.md` were checked (every category heading is its dictionary's name).
- Every Playwright baseline deleted and taken again (542; 59 changed, one stale keyboard shot of the danger zone gone);
  a sample of every screen looked at in both browsers.
- The demo GIFs were not recorded: recording needs the app's window on the user's screen and the screen recording
  permission; [deferred](../../../deferred/demo-scenarios-for-new-screens.md) until the user's walk.
