# 019/04/02 — The runs by hand and the conclusions

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** docs

A checklist for the user, then the conclusions.

## In the sandbox

- [ ] `rm -rf sandbox/app-leftovers && npm run dev:sandbox -- app-leftovers`. Scan; walk `EXPECTED.md` line by line,
  negatives first: nothing of a negative is offered (Notes, Apple's, the tool's, the installed Editor's, the link, the
  group container, the container); the left-out lines read as written (running, changed lately, elsewhere, a socket,
  the system's file); each positive is offered in its category at its danger (caches Safe, settings Medium, support
  folder High); the support folder counts in its id's line of the map, "no longer installed".
- [ ] On the module's page: **Keep settings** selects only the caches; Review names Safe. **Remove everything**
  selects the three categories; Review names High; confirm to the Trash: the items land in `home/.Trash`; Cleanups →
  put back: they are back. The link's target in `home/elsewhere/Linked`, the group container and the container are
  untouched.
- [ ] Ignore the data category, then the module; restore.
- [ ] `npm run dev:sandbox -- app-leftovers-spotlight-off`: nothing offered; the left-out line names Spotlight.

The check that a changed expectation fails the autotest, once per kind (a danger, a left-out's items, a proof), was
made while building (03/01's conclusions).

## On the developer's Mac (read-only)

`npm run dev`, a scan of `~` only (a development build's cleaner reaches only a sandbox, so nothing can be removed),
compared with these read-only commands, for each offered id `<id>`:

```sh
mdfind "kMDItemCFBundleIdentifier == '<id>'c"
ls -d /Applications/*.app ~/Applications/*.app 2>/dev/null | while read -r app; do
  defaults read "$app/Contents/Info" CFBundleIdentifier 2>/dev/null; done | grep -i "<id>"
ls -ld ~/Library/{Caches,Logs,Preferences,Application\ Support,Containers,HTTPStorages,WebKit,Cookies}/<id>* \
  ~/Library/Saved\ Application\ State/<id>.savedState 2>/dev/null
mount
for volume in / /Volumes/*; do mdutil -s "$volume"; done
ls ~/Library/{Caches,Application\ Support,Preferences} | grep -c '\.'
```

- [ ] Every offered id is no installed app (`mdfind` answers nothing, no bundle in the apps folders) and was an app's
  (its saved state or container is there); none starts with `com.apple.`; nothing under `Group Containers` is offered;
  every size matches `du -sh`'s.
- [ ] Count the ids with leftovers that the proof leaves out (named after an id, not installed, no saved state and no
  container): the number, and two or three examples, in the conclusions; a later proof is weighed from it.
- [ ] The facts "to verify": `mdutil -s` says `Indexing enabled.` for an indexed volume (and what it says for an
  unindexed one: mount a disk image with `hdiutil attach -nobrowse` and one without, and turn indexing off for a test
  image with `sudo mdutil -i off`); `/` and the data volume are browsable as `mounted_volumes` reads them; the places
  of `places.toml` are readable without Full Disk Access (a place that is not is dropped from the table or made
  optional, with its reason). The texts and the table are corrected where they differ.
- [ ] Shared frameworks' ids among the offered ones (an updater or a crash reporter that writes an id-named cache for
  many apps): each one added to `exclude.toml` with its `why`.
- [ ] With a disk image mounted that Spotlight does not index: nothing is offered by `app-leftovers`, and a removed
  JetBrains IDE's data (if any) is left out with the Spotlight reason.
- [ ] With a Time Machine or other backup disk attached: what `mdutil -s` says of it, and that `app-leftovers` then
  keeps everything (the known issue of an unindexed volume).
- [ ] The scan's time against `main`'s on the same home (rule 4: within 5 %).
- [ ] The module's `finish` time on the same scan, measured: up to three `mdfind` per id that reaches the
  Spotlight row, plus one `mdutil -s` per volume; the time and the number of ids asked, in the conclusions.
- [ ] The scan bench before and after plan 019 on a quiet Mac (`npm run bench`), including the shaped interest 01/01
  added; 01/01's numbers say whether its runs were taken on a busy Mac. Known costs to name: one shape check per name
  under a shaped node; the whole claims of id-named folders (fewer nodes in the tree).
  01/01's bench measured no slowdown (-0.9 % at 12 threads, -0.3 % at 1 thread) on a busy Mac (load average 19 to 26,
  the same build spread by about 8 %), and its shaped `Data/*` covers only a few entries: retake `npm run bench` on a
  quiet Mac, with a shaped interest over many names.
- [ ] One real cleanup, by the developer, with Finder (not the app): a test app installed, opened (a window, so its
  saved state exists), quit and dragged to the Trash; its leftovers moved to the Trash by hand; the Trash put back.

## Files

- `docs/plans/019-uninstalled-app-leftovers/readme.md`, `00-design.md` and every plan's readme — the conclusions: what
  was built, what changed from the design and why, the numbers (tests, coverage, the bench, the scan's time, the ids the
  proof left out), the calls taken while building, what the real Mac showed.
- `modules/app-leftovers/{places,exclude}.toml`, `locales/en.json`, `docs/categories/app-leftovers-*.md` — corrected
  where the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Not written yet.
