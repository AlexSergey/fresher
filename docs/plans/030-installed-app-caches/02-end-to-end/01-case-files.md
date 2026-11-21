# 030/02/01 — The case files

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. Two files, since an open app is shared by every case of a file (as
`dev-caches-xcode-open.toml`). And the checklist's negative sentence names the case file's module (agent's call 11).
The module is registered in the
app here, not in 01/03: `every_registered_module_has_a_case_file` in `app/tauri/tests/sandboxes.rs` fails for a
registered module without a case file.
The design: [`../00-design.md`](../00-design.md), section 7.

## Files

- `fixtures/sandboxes/app-caches.toml` (new) — `feature = "app-caches"`, `module = "app-caches"`; a head comment: what
  the file covers, that an installed app is a `[[case.app]]` (the sandbox's apps declare no helpers, so a helper's
  folder is the fixtures' and the core e2e's), that files are small with `fill = "data"`, that a removed app's cache
  here has no saved state, so `app-leftovers` would not claim it either. With these exact names:
  - negative, in this order:
    - "a name that is not an app's id is never claimed" — `home/Library/Caches/OtherTool/x`; `[case.map]`
      `your-folders` at `home/Library/Caches/OtherTool`;
    - "Apple's caches are never claimed" — `home/Library/Caches/com.apple.Maps/x`;
    - "Fresher's own cache is never claimed" — `home/Library/Caches/io.github.alexsergey.fresher.dev/x`;
    - "a removed app's cache is not this module's" — `home/Library/Caches/com.example.Lost/Cache.db`, no app;
    - "a vendor's folder above an installed app's id is never claimed" — `home/Library/Caches/com.example/x`, with
      `com.example.Writer` installed (`Applications/Writer.app`);
    - "a link named like an installed app's id is never claimed" — `[[case.symlink]]`
      `home/Library/Caches/com.example.Reader.Linked` → `../../Projects/kept`, a file `home/Projects/kept/x`, with
      `com.example.Reader` installed; after the cleanup the link and its target are unchanged;
    - "a cache holding a socket is kept" — `home/Library/Caches/com.example.Busy/x` and a socket
      `home/Library/Caches/com.example.Busy/s` (`kind = "socket"`), `com.example.Busy` installed; left out
      `app-caches.left-out.special` 1 in `app-caches.library-caches`;
  - positive:
    - "an installed app's cache is emptied" — `home/Library/Caches/com.example.Scribe/Cache.db` and
      `home/Library/Caches/com.example.Scribe/fsCachedData/a`, `com.example.Scribe` installed
      (`Applications/Scribe.app`); `home/Library/Caches/com.example.Scribe` in `app-caches.library-caches`, low,
      `emptied`; `[case.map]` `explained` at that folder;
    - "an updater's cache under the app's id is emptied" — `home/Library/Caches/com.example.Player.ShipIt/update.zip`,
      `com.example.Player` installed; low, `emptied`.
- `fixtures/sandboxes/app-caches-app-open.toml` (new) — one `[[case.app]]` with `running = true`
  for `com.example.Mailer` (an installed, open app; a `[[case.running]]` means open and not installed); negative "an
  open app's cache is kept" — `home/Library/Caches/com.example.Mailer/Cache.db`;
  left out `app-caches.left-out.app-open` 1 in `app-caches.library-caches`; positive "a closed app's cache still goes
  while another app is open" — `home/Library/Caches/com.example.Notes/Cache.db`, `com.example.Notes` installed; low,
  `emptied`. File paths apart from `app-caches.toml`'s.
- `fixtures/golden/sandbox-app-caches.json`, `sandbox-app-caches-app-open.json` (new) — `UPDATE_GOLDEN=1`, reviewed:
  the findings with their apps, the left-out counts, the map, the cleanup's `Emptied` outcomes, the tree after (each
  emptied folder still there, its entries in the sandbox's Trash).
- `crates/sandbox/src/expected.rs` — `sentence(case)` takes the file's module: a negative of a file that names one
  reads "never offered by `<module>`: …"; a file with no module (`cleanup-guards`, `storage-map`) keeps "never
  offered: …". Its tests: one sentence of each form, negatives first.
- `app/tauri/tests/sandboxes.rs` — `app_caches => "app-caches"` and `app_caches_app_open => "app-caches-app-open"` in
  `case_files!`, sorted by file name.
- `app/tauri/Cargo.toml`, `app/tauri/src/compose.rs` (`AppCaches(#[from] …)` in the compose error and the module in
  `modules()`, after `app-leftovers`), `app/tauri/src/commands.rs` and the expected-id lists of the composition tests —
  the registration.
- `docs/testing.md` — the two case files in the list of case files; the checklist's sentence ("never offered by the
  case file's module") where `EXPECTED.md` is described.

Each folder is written by one case only (one world). What the code does for each case is written down when the file is
made; a case that differs from the design is fixed in the module, not in the case file.

## Produces

The module's case files, for 03/02 (the run by hand).

## Checks

- `npm run sandbox:test` passes for both files and every other case file (their goldens unchanged: the sentence is in
  `EXPECTED.md` only, not in the reports).
- `npm run sandbox -- all` leaves out no file it did not already leave out (`unused-toolchains*` and
  `local-ai-models-ollama-*`).
- Once per kind, by hand, written in this step's conclusions: a danger changed (`low` → `safe` on the Scribe case) and
  a left-out's items changed (`1` → `2` on the socket case) each make the autotest fail; then reverted.
- `npm run sandbox -- app-caches` writes the sandbox; `EXPECTED.md` lists every case, negatives first, each negative
  "never offered by `app-caches`"; `npm run sandbox -- app-leftovers` reads "never offered by `app-leftovers`".
- `cargo nextest run -p fresher-sandbox -p fresher-app -p fresher-module-app-caches`.

## Conclusions

Built with the names of preflight F2 (`OtherTool`, `com.apple.Maps`, `com.example.Lost`, `Scribe`, `Reader`), so `npm run sandbox -- all` leaves out nothing new. The registration in `fresher-app` and its three id lists landed here (F1). The open Mailer is an installed app with `running = true`, not a separate running entry (that one means open and not installed). The generated EXPECTED.md names the module in a negative's sentence ("never offered by `app-caches`"); a case file with no module keeps the plain form. A positive's map case states `part = "explained"` (F9). By-hand mutations, each reverted: a positive's danger changed from Low to Safe failed the autotest ("the danger Safe is below the category's Low"); the socket case's left-out items changed from 1 to 2 failed it ("expected 2 items, got 1").
