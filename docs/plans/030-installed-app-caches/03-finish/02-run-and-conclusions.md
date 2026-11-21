# 030/03/02 — The runs by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs, modules

A checklist for the user, then the conclusions.

## In the sandbox

- [ ] `rm -rf sandbox/app-caches && npm run dev:sandbox -- app-caches`. Scan; walk `EXPECTED.md` line by line,
  negatives first: nothing of a negative is offered by `app-caches` (the name that is not an id, Apple's, Fresher's,
  the removed app's, the vendor's folder, the link); the socket case reads as kept with its reason; each positive is
  offered in "Caches of your apps" at Low, titled `Scribe` and `Player: com.example.Player.ShipIt`, each reason naming
  the app and a size.
- [ ] The storage map: the offered caches in the explained part, each in its app's line; `home/Library/Caches/OtherTool`
  in your folders.
- [ ] Select everything; Review names Low; confirm to the Trash: the result reads "Moved to the Trash";
  `home/Library/Caches/com.example.Scribe` is still there and empty, `Cache.db` and `fsCachedData` are in
  `home/.Trash`. Cleanups → put back: both entries are back inside the folder.
- [ ] `npm run dev:sandbox -- app-caches-app-open`: Mailer's cache is kept "This app, or a part of it, is
  open…"; the closed Notes app's is offered.
- [ ] `npm run dev:sandbox -- app-leftovers`: the installed app's caches are offered by "Caches of your apps", and its
  negative reads "never offered by `app-leftovers`", which still holds.
- [ ] Ignore one cache, then the category, then the module; restore each.

## On the developer's Mac (a dry run)

`npm run dev`, a scan of `~` (a development build on the real Mac makes every cleanup a dry run). Compare with these
read-only commands:

```sh
du -sh ~/Library/Caches/*.* 2>/dev/null | sort -h | tail -30
ls ~/Library/Caches | grep -c '\.'
mdls -name kMDItemCFBundleIdentifier -raw /Applications/*.app 2>/dev/null | tr '\0' '\n' | sort | head -50
```

- [ ] Every offered cache checked by hand: its folder named after an installed app's id, a helper's, or an id under
  the app's; its size against `du`; its title names the right app.
- [ ] Nothing of Apple's, nothing of Fresher's own, and no removed app's cache offered by this module; a removed app's
  cache still goes to `app-leftovers` when it is proven and old.
- [ ] An open app's cache is left out; quit it and rescan: it is offered.
- [ ] The facts "to verify":
  - the largest ten caches offered: look inside each (read-only) for what does not come back (an offline library, a
    draft, a download that needs a paid account); an app that keeps such data gets an `exclude.toml` row with its
    why, and the category doc names the kind;
  - how many small findings a real Mac shows (count under 1 MB): whether a size floor is needed after all (agent's
    call 7);
  - an app with an always-running helper (a menu bar agent): whether its cache is never offered, and whether the text
    tells the user what to do;
  - an updater's cache (`<id>.ShipIt` or the like) holding a downloaded update: offered at Low, and the update
    downloads again (on a Mac where one is pending; otherwise noted).
  The tables, texts and category doc are corrected where they differ.

## Files

- `docs/plans/030-installed-app-caches/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was
  built, what changed from the design and why, the numbers (tests, coverage, what the module offered on the
  developer's Mac and how many folders), the calls taken while building, what the real Mac showed.
- `modules/app-caches/exclude.toml`, `locales/en.json`, `docs/categories/app-caches-library-caches.md` — corrected
  where the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; `git diff <plan 029's head> -- crates/core crates/scanner
  crates/cleaner crates/engine` empty; the user has looked at the result.

## Conclusions

Written without the user's walk: the automated checks and the conclusions are done, the boxes above are **unchecked on purpose** (the sandbox walks and the dry run on the real Mac are the user's). The numbers, the changes from the design and the facts "to verify" are in [the global readme](../readme.md#conclusions); each fact above is checked against the code and the texts when the user reports what the real Mac shows. The status stays 🔨 in progress until then.
