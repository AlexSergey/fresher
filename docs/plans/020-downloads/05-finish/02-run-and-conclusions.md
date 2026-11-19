# 020/05/02 — The runs by hand and the conclusions

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs, modules

A checklist for the user, then the conclusions.

## In the sandbox

- [ ] `rm -rf sandbox/downloads && npm run dev:sandbox -- downloads`. The banner says access and installs are
  simulated. Scan; walk `EXPECTED.md` line by line, negatives first: nothing of a negative is offered (the document,
  the disk image of no app, the copy, the link, the folder of yours, the big file inside it); the left-out lines read
  as written (changed lately, newer than the installed app, downloaded after it, a newer macOS, opened lately); each
  positive is offered in its category at its danger (unfinished Low with the placeholder, Safari Medium, installers
  Low, the macOS installer Low, the big file High); the installer counts in `Example`'s line of the map;
  `Downloads/Holiday` opens in the browse view.
- [ ] Select everything; Review names High; confirm to the Trash: the items land in `home/.Trash`; Cleanups → put
  back: they are back. The link's target in `home/elsewhere/clip` is untouched.
- [ ] Ignore the large-untouched category, then the module; restore.
- [ ] `npm run dev:sandbox -- downloads-denied`: the module's page shows the notice that Downloads is not allowed, with
  the way to allow it; nothing offered; the coverage names Downloads.
- [ ] `npm run dev:sandbox -- downloads-browser-open`: the left-out line names the open browser.

The check that a changed expectation fails the autotest, once per kind (a danger, a left-out's items, an access value),
was made while building (04/01's conclusions).

## On the developer's Mac (read-only)

`npm run dev`, Settings → Permissions → Downloads switched on (the system may ask: allow it), a scan of `~` only (a
development build's cleaner reaches only a sandbox, so nothing can be removed). Compare with these read-only commands:

```sh
ls -la ~/Downloads | head -50
for f in ~/Downloads/*.{crdownload,part,opdownload,partial}; do stat -f "%Sm %Sc %z %N" "$f"; done 2>/dev/null
for f in ~/Downloads/*.{dmg,pkg}; do stat -f "%Sm %Sc %N" "$f"; done 2>/dev/null
for app in /Applications/*.app; do stat -f "%Sc %N" "$app"; done | sort | tail -20
ls /var/db/receipts/*.plist | while read -r r; do plutil -extract PackageFileName raw "$r"; done
sw_vers -productVersion
mdls -name kMDItemLastUsedDate -name kMDItemDateAdded ~/Downloads/<a file you opened lately>
xattr -l ~/Downloads/<the same file>
```

- [ ] The first scan after switching Downloads on offers (01/04): no second scan is needed.
- [ ] Every offered item checked by hand: an unfinished download is old and its browser closed; an installer's app is
  installed at the installer's version or newer (by the app's About window), and it was installed after the installer
  was opened; a macOS installer is older than `sw_vers`; a big file is one you have not opened for 30 days. Nothing
  below the first level of Downloads, nothing outside it.
- [ ] Count, without names, how many `.dmg`/`.pkg` files the name rule and the receipts prove, and how many they leave
  out; two or three invented examples of what was missed, in the conclusions.
- [ ] The facts "to verify":
  - opening a file from Finder writes `com.apple.lastuseddate#PS` and moves its status change (`stat -f %Sc` before and
    after a double-click), as seen on 2026-11-19 on five files; a file opened from an app's own open panel — say what
    happens;
  - opening a disk image from Finder moves its status change (agent's call 2);
  - an app updated by its own updater during the plan has a new bundle status change; a system update moves every
    app's (seen once on this Mac: all on one day);
  - the receipts' `PackageFileName` and modification time, as 01/03 read them; whether a downloaded product archive's
    name ever equals a receipt's (install a small test package and look);
  - the browsers' bundle ids of `unfinished.toml` (each by `mdls -name kMDItemCFBundleIdentifier` on the installed
    browser, where installed) and which browser writes `.partial` (if none, the row is removed); Chrome's, Firefox's,
    Opera's suffixes by a download paused and the browser quit;
  - Safari's `.download` bundle holds `Info.plist` (pause a download, quit Safari);
  - the macOS installers' names and versions of `macos.toml` against Apple's list of installers (names after Tahoe
    added; a name that differs corrected).
  The tables, texts and category docs are corrected where they differ.
- [ ] The scan's time against `main`'s on the same home, Downloads allowed in both (rule 4: within 5 %); the scan
  bench before and after 01/02 on a quiet Mac (`npm run bench`).
- [ ] One real cleanup, by the developer, with Finder (not the app): an old installer of an installed app moved to the
  Trash by hand and put back, to confirm nothing else depends on it.

## Files

- `docs/plans/020-downloads/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built,
  what changed from the design and why, the numbers (tests, coverage, the bench, the scan's time, the installers the
  proof left out), the calls taken while building, what the real Mac showed.
- `modules/downloads/{unfinished,macos}.toml`, `locales/en.json`, `docs/categories/downloads-*.md` — corrected where
  the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Not written yet.
