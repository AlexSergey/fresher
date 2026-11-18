# 018/03/01 — The sandbox case files

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. Three files, since every tool's place is fixed and the open apps are a file's own
(design, agent's call 12).

## Files

- `fixtures/sandboxes/dev-caches.toml` (new) — `feature = "dev-caches"`, `module = "dev-caches"`; a head comment
  saying what the file covers and why the two others exist; the eight negative cases, then the nine positive ones, of
  the design's section 6, "Sandbox", with these exact names (the link's target, `home/elsewhere/Link/`, unchanged
  after the cleanup):
  - negative: "look-alike folders are never claimed", "a tool's settings, credentials and programs are never claimed",
    "stores that are not plain caches are never claimed", "Homebrew's own data beside its downloads is never
    claimed", "derived data that is no project is never claimed", "a link or a file in a project's place is never
    claimed", "a project holding a socket is kept", "a project holding the system's file is kept";
  - positive: "npm's cache goes to the Trash", "Yarn's cache goes", "Homebrew's downloads go, its links stay",
    "Cargo's downloaded crates are cleaned, the unpacked sources stay offered", "CocoaPods' cache goes", "Gradle's caches go", "pip's cache goes",
    "Go's build cache goes", "a closed Xcode's project build data goes, in Xcode's line".
  Xcode installed and closed (`[[case.app]]` `Applications/Xcode.app`, `com.apple.dt.Xcode`, in the first positive
  that needs it); valid tags with `fill = "cachedir-tag"`; the positives' files `fill = "data"`, so the map's numbers
  are blocks; `[[case.left-out]]` for `special` and `not-yours` (category `dev-caches.xcode-derived-data`, one item
  each); `[case.map]` `your-folders` at `home/.npm/_cacache-old` and `home/.cargo/registry/index`, `explained` at
  `home/.npm/_cacache` and at the closed Xcode's project.
- `fixtures/sandboxes/dev-caches-unsigned.toml` (new) — `feature = "dev-caches-unsigned"`, `module = "dev-caches"`:
  negatives only, one per tool, each place present without its sign: "npm's cache without its index is not claimed"
  (`content-v2` only), "Yarn's cache of another version is not claimed" (`v5`), "Homebrew's downloads without its API
  folder are not claimed", "Cargo's crates without the registry's tag are not claimed", "CocoaPods' folder without its
  pods is not claimed" (`Specs` only), "Gradle's caches without a tag are not claimed", "pip's folder without its
  download cache is not claimed" (`wheels` only), "Go's folder without its own files is not claimed" (a `01/` folder
  only), "a project's folder without Xcode's file is not claimed". No file path is one `dev-caches.toml` writes
  (`all` joins them); the invalid tag, a link and a file at a place are the fixtures' and the core e2e's (design,
  section 6).
- `fixtures/sandboxes/dev-caches-xcode-open.toml` (new) — `feature = "dev-caches-xcode-open"`, `module =
  "dev-caches"`; `[[case.running]]` Xcode (`com.apple.dt.Xcode`); negative "a project's build data is kept while Xcode
  is open" (`…/DerivedData/Open-cdefghijklmnopqrstuvwxyzabcd/`, left out `app-open`); positive "npm's cache still goes
  while Xcode is open" (its files named apart from `dev-caches.toml`'s).
- `fixtures/golden/sandbox-dev-caches.json`, `sandbox-dev-caches-unsigned.json`, `sandbox-dev-caches-xcode-open.json`
  (new) — `UPDATE_GOLDEN=1`, reviewed: the findings with their app, the left-out counts, the map, the cleanup's
  outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — the three files in `case_files!`, sorted by file name.
- `docs/testing.md` — the three case files in the list of case files, and that their worlds are their own: the
  autotest runs each alone, `all` joins them for a look by hand only, so none writes a path another writes.

What the code does for each case is written down when the file is made; a case that differs from the design is fixed
in the module, not in the case file.

## Produces

The module's sandboxes: their autotests and the runs by hand (04/02).

## Checks

- `npm run sandbox:test` passes the three files; it fails, once each by hand, when a positive's danger, a left-out's
  `items`, and a positive's sign (`index-v5` removed from npm's cache) are changed on purpose (written in the
  conclusions).
- `npm run sandbox -- dev-caches` (and the two others) writes `sandbox/<feature>/` with `world.toml` (the apps and the
  stated entries) and an `EXPECTED.md` that names every stated fact; `npm run sandbox -- all` writes them beside the
  other case files without a clash.
- `npm run lint`.

## Conclusions

Built (075589c): `fixtures/sandboxes/dev-caches.toml` (eight negatives, then nine positives, the brief's names; the
link in a project's place untouched after the cleanup; left-outs `special` and `not-yours`; the map's your-folders and
explained lines), `dev-caches-unsigned.toml` (nine negatives, one per tool, file paths apart from the first file's),
`dev-caches-xcode-open.toml` (a project left out `app-open`, npm's cache still offered), the three goldens
`fixtures/golden/sandbox-dev-caches*.json` (reviewed: ten findings, seven Low, Go and the project Safe), the three
`case_files!` entries in `app/tauri/tests/sandboxes.rs` and the case files' paragraph in `docs/testing.md`.

Deviations and calls:

- Homebrew's positive takes its `api/` sign from the negative case's file (F13); the file's head comment says so.
- A positive case has one `[case.expect]`, so Cargo's positive expects and cleans `registry/cache` only; `registry/src`
  is offered (in the golden) and stays. The core e2e (03/02) removes and puts back both folders.
- Xcode is declared installed in the first positive that needs it and so for the whole file.
- `Fill::CachedirTag` was not changed to sync (no allocated-size instability in six runs).

Tests: `npm run sandbox:test`: 40 of 40 passed, six runs, stable, goldens unchanged;
`every_registered_module_has_a_case_file` passes again. `npm run sandbox -- dev-caches`, `-unsigned`, `-xcode-open` and
`all` write 17, 9, 2 and 98 cases with no clash. Three by-hand failure checks, each restored: a lowered danger, a
changed left-out item count and a renamed sign each fail the autotest with the case named. Parked minors: the Cargo
positive's name overstates (only `registry/cache` is cleaned), and the Xcode-closed case's name says "in Xcode's line",
which is unasserted.
