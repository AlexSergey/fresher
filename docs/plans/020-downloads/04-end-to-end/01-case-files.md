# 020/04/01 — The case files

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** sandbox, fixtures, app, docs

Rule 8 of [`../../../rules.md`](../../../rules.md): the module ships its case files, negatives first, with their golden
reports and left-out expectations. Three files, each a world of its own: access granted, access denied, a browser
open. The design: [`../00-design.md`](../00-design.md), section 4, "Sandbox".

## Files

- `fixtures/sandboxes/downloads.toml` (new) — `feature = "downloads"`, `module = "downloads"`; a head comment: what the
  file covers, that it states `access = { downloads = "granted" }` and `macos = "15.4"` (on its first case; the world
  joins them for every case), that installed apps are shared by every case (each case has its own app), that the big
  files are sparse (the map counts no blocks for them: their cases check the offer and the Trash), and why the two
  other files exist. Each case its own names (no path written twice); **every entry states `age` and `changed`**
  (`"40d"`, `"400d"`, or `"3d"`/`"5d"` for a recent one); positives' small files `fill = "data"`. With these exact
  names:
  - negative, in this order:
    - "a document is never claimed" — `home/Downloads/report.pdf` (400 days); `[case.map]` `your-folders` at it;
    - "a disk image of no installed app is never claimed" — `home/Downloads/backup-2024.dmg` (400 days);
    - "a copy of an installer is never claimed" — `[[case.app]]` `Applications/Copyable.app` (`com.example.Copyable`,
      `1.0`, `changed = "10d"`), `home/Downloads/Copyable (1).dmg` (20 days);
    - "a link named like a download is never claimed" — `home/Downloads/clip.mov.part` → `home/elsewhere/clip`
      (holding `x`); the link and its target unchanged after the cleanup;
    - "a folder of yours is never claimed" — `home/Downloads/Holiday/photo.jpg` (400 days); `[case.map]`
      `your-folders` at `home/Downloads/Holiday`;
    - "a big file inside a folder of yours is never claimed" — `home/Downloads/Films/long.mov` (2 GB sparse, 400
      days);
    - "an unfinished download changed lately is kept" — `home/Downloads/fresh.zip.crdownload` (3 days); left out
      `downloads.left-out.recent` 1 in `downloads.unfinished`;
    - "an installer newer than the installed app is kept" — `[[case.app]]` `Applications/Sample.app`
      (`com.example.Sample`, `1.0`, `changed = "10d"`), `home/Downloads/Sample-2.0.dmg` (20 days); left out
      `not-older` 1 in `downloads.installers`;
    - "an installer downloaded after the app was installed is kept" — `[[case.app]]` `Applications/Later.app`
      (`com.example.Later`, `3.0`, `changed = "30d"`), `home/Downloads/Later.dmg` (5 days); left out `not-older` 1;
    - "an installer for a newer macOS is kept" — `home/Downloads/Install macOS Tahoe.app/Contents/Info.plist` (40
      days); left out `newer-macos` 1 in `downloads.macos-installers`;
    - "a big file opened lately is kept" — `home/Downloads/recent.iso` (2 GB sparse, `age = "400d"`, `changed =
      "3d"`: opened from Finder 3 days ago); left out `recent` 1 in `downloads.large-untouched`;
  - positive:
    - "an unfinished Chrome download goes" — `home/Downloads/movie.mp4.crdownload` (40 days); `downloads.unfinished`,
      low, `in-trash`; `[case.map]` `explained`;
    - "an unfinished Firefox download goes with its empty placeholder" — `home/Downloads/song.flac.part` (40 days) and
      `home/Downloads/song.flac` (`size = 0`, 40 days); both `downloads.unfinished`, `in-trash` (two expectations);
    - "a Safari download goes, at Medium" — `home/Downloads/trip.zip.download/{Info.plist, trip.zip}` (40 days, the
      folder a `[[case.dir]]` with both); `downloads.safari-resumable`, medium, `in-trash`;
    - "an installer of an installed app goes" — `[[case.app]]` `Applications/Example.app` (`com.example.Example`,
      `2.0`, `changed = "10d"`), `home/Downloads/Example-2.0.dmg` (20 days); `downloads.installers`, low, `in-trash`;
    - "a package with its receipt goes" — `home/Downloads/Tool-3.1.pkg` (20 days), `[[case.receipt]]` `package =
      "Tool-3.1.pkg"`, `installed = "10d"`; `downloads.installers`, low, `in-trash`;
    - "an old macOS installer goes" — `home/Downloads/Install macOS Sonoma.app/Contents/Info.plist` (40 days);
      `downloads.macos-installers`, low, `in-trash`;
    - "a big file untouched for a long time goes, at High" — `home/Downloads/old-backup.iso` (2 GB sparse, 400 days);
      `downloads.large-untouched`, high, `in-trash`.
- `fixtures/sandboxes/downloads-denied.toml` (new) — `feature = "downloads-denied"`, `module = "downloads"`; a head
  comment (denied access is a world of its own; in `npm run sandbox -- all` the most restrictive access wins, so
  Downloads is then denied for every case file); negative "nothing in Downloads is read without access" with `access =
  { downloads = "denied" }`: `home/Downloads/denied.zip.crdownload` and `home/Downloads/Denied-1.0.dmg` (40 days),
  nothing offered, both untouched. `EXPECTED.md` says the module's page shows the notice that Downloads is not
  allowed.
- `fixtures/sandboxes/downloads-browser-open.toml` (new) — `feature = "downloads-browser-open"`, `module =
  "downloads"`; a head comment (an open browser is shared by every case of a file); negative "an unfinished download of
  an open browser is kept": `access = { downloads = "granted" }`, `[[case.running]]` `Google Chrome`
  (`com.google.Chrome`), `home/Downloads/open.zip.crdownload` (40 days); left out `browser-open` 1 in
  `downloads.unfinished`.
- `fixtures/golden/sandbox-downloads.json`, `sandbox-downloads-denied.json`, `sandbox-downloads-browser-open.json`
  (new) — `UPDATE_GOLDEN=1`, reviewed: the findings with their app, the left-out counts, the map, the cleanup's
  outcome, the tree after.
- `app/tauri/tests/sandboxes.rs` — the three files in `case_files!`, sorted by file name.
- `docs/testing.md` — the three case files in the list of case files, as `app-leftovers`' are described.
- `docs/known-issues/repo-sandbox-all-worlds-interact.md` — access joins as Spotlight does (the most restrictive wins
  in `all`).

What the code does for each case is written down when the file is made; a case that differs from the design is fixed
in the module, not in the case file.

## Produces

The module's case files, for 05/02 (the runs by hand).

## Checks

- `npm run sandbox:test` passes for the three files and every other case file (their goldens unchanged).
- Once per kind, by hand, written in this step's conclusions: a danger changed (`medium` → `low` on the Safari case),
  a left-out's items changed (`1` → `2`), and the access value changed (`granted` → `denied` in `downloads.toml`) each
  make the autotest fail; then reverted.
- `npm run sandbox -- downloads` writes the sandbox; `world.toml` holds `[access]`, `macos`, `[[receipt]]` and the
  apps' stated changes; `EXPECTED.md` lists every case, negatives first.
- `cargo nextest run -p fresher-sandbox -p fresher-app`; `npm run lint` (the TOML formatting check, if any).

## Conclusions

Built as written, with these calls:

- A case holds one expectation, so the Firefox case is two: `song.flac.part` and the empty `song.flac`. The placeholder
  is offered only beside the `.part` of the previous case in the same folder.
- The sparse 2 GB files made the fake volume's used space (the tree's logical size plus 64 MiB) exceed its 1 GiB, so
  `sandbox_disk` caps the tree's size to what the volume holds (`app/tauri/tests/sandboxes/map.rs`); the other goldens
  are unchanged.
- `downloads = "not-asked"` turning granted once the Settings switch is on is not provable here: the autotest never
  opts in, so the module is unavailable for `not-asked` (a case stating it failed). Left to the core e2e flows (04/02).
- By hand, each made the autotest fail and was reverted: `medium` to `low` on the Safari case, a left-out's `not-older`
  items `1` to `2`, and `granted` to `denied` in `downloads.toml`.

