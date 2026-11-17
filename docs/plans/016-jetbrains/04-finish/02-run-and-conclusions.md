# 016/04/02 — The runs by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## In the sandbox

- [ ] `rm -rf sandbox/jetbrains && npm run dev:sandbox -- jetbrains`, with a real JetBrains IDE open on the Mac: the banner
  says open apps are simulated; the real open IDE changes nothing.
- [ ] Scan; walk `EXPECTED.md` line by line, negatives first: nothing of a negative is offered; every left-out line reads
  as written under its category; each positive is offered at its level and in WebStorm's (or Rider's) line of the
  storage map.
- [ ] Select every positive, confirm to the Trash: the Medium level and the settings' sentence are on the confirmation;
  the folders land in `home/.Trash`; Cleanups → put back: they are back. Again with `--real-trash`: Finder's Put Back
  restores them.
- [ ] Ignore the module, then the group Development, from Settings and from the module's page; restore.

## On the developer's Mac

`npm run dev`, a scan of `~` only (a development build's cleaner reaches only a sandbox, so nothing can be removed):

- [ ] `ls ~/Library/{Application\ Support,Caches,Logs}/JetBrains` against what is offered and left out; every offered
  folder is older than an installed IDE that is not open; the version in use is left out with "This is the version you
  use."
- [ ] Each installed IDE's bundle id and version (`mdls -name kMDItemCFBundleIdentifier -name kMDItemVersion
  /Applications/<IDE>.app`, and in `~/Applications` for Toolbox installs): the products table, the `-EAP` ids and the
  Toolbox app's id, the Toolbox folders, a folder's age as its own modification time, and the agent's calls 1, 2, 5, 6,
  13, 14 and 19 confirmed or corrected; `products.toml`'s comment says which ids were seen.
- [ ] An IDE opened, then a rescan: its folders are left out with "The IDE is open…"; quit it, rescan: offered again.
- [ ] The scan's time and the module's share of it noted (rule 4: the module adds three narrow interests; the time is
  compared with a scan of `main`).

## Files

- `docs/plans/016-jetbrains/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built, what
  changed from the design and why, the measured numbers (tests, coverage, the scan's time), the agent's calls taken
  while building, what the real Mac showed.
- `modules/jetbrains/products.toml`, `docs/categories/jetbrains-*.md` — corrected where the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Waits for the user: nothing above is ticked. The user asked on 2026-11-17 to merge the plan into `main` before the run;
the results (the sandbox walk, the `mdls` check of every bundle id and the Toolbox paths, the scan's time, the
correction of `products.toml` and the category files if the Mac differs) are written here, on `main`, when the
checklist is done. The plan's conclusions ([`../readme.md`](../readme.md)) say what was built and what is open.
