# 017/04/02 — The runs by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## In the sandbox

- [ ] `rm -rf sandbox/temp && npm run dev:sandbox -- temp`. Scan; walk `EXPECTED.md` line by line, negatives first:
  nothing of a negative is offered; every left-out line reads as written under "Old temporary files"; each positive is
  offered at Safe; the map lists "Your temporary folder" and "Shared temporary folder". The fresh file and the
  unpacked folder are young by their real status change, not by a stated one (03/01's conclusions).
- [ ] Select every positive, confirm to the Trash: Safe on the confirmation; the items land in `home/.Trash`; Cleanups →
  put back: they are back. The link's target in `home/Documents/linked` is untouched.
- [ ] Settings → Temporary files: 6 refused (the UI's "Enter a whole number from 7 to 30."), 14 saved; rescan: the
  10-day item is left out `recent`; back to 7.
- [ ] Ignore the module, then the group System; restore.

The check that a changed expectation fails the autotest, once per kind (a danger, a left-out's items, a stated fact, a
negative's file), was made while building (03/01's conclusions).

## On the developer's Mac (read-only)

`npm run dev`, a scan of `~` only (a development build's cleaner reaches only a sandbox, so nothing can be removed),
compared with these read-only commands from the research:

```sh
getconf DARWIN_USER_TEMP_DIR; ls -ld /tmp /var /private/tmp /private/var/tmp
ls -lO "$(dirname "$(getconf DARWIN_USER_TEMP_DIR)")"                          # T, C, 0, X and their flags
plutil -p /System/Library/LaunchDaemons/com.apple.bsd.dirhelper.plist          # the daily sweep and its 3 days
plutil -p /System/Library/LaunchDaemons/com.apple.tmp_cleaner.plist; cat /usr/libexec/tmp_cleaner
log show --last 3d --predicate 'process == "dirhelper"' --info | tail -50      # does it log, does it run at start
T=$(getconf DARWIN_USER_TEMP_DIR)
find "$T" -mindepth 1 -maxdepth 1 -mtime +7 -exec stat -f '%Su %Sf %Sm %Sc %N' {} \;
find "$T" -mindepth 1 -maxdepth 1 -ctime +7 | wc -l
find "$T" /private/tmp -maxdepth 3 \( -type s -o -type p \) 2>/dev/null       # sockets and pipes
du -sk "$T"/* 2>/dev/null | sort -rn | head -20
lsof +D "$T" 2>/dev/null | awk '{print $1}' | sort | uniq -c | sort -rn | head
```

- [ ] Every offered item is in the user's own `T` or `/private/tmp`, owned by the user, with no flag, and its newest
  change (`stat -f %Sc`, `%Sm` over its inside) more than 7 days old; nothing in `C/`, `0/`, `X/` or `TemporaryItems`
  is claimed; the root-owned locked updater package is left out "It belongs to the system or another user…".
- [ ] The left-out counts match the commands' (young items, root's, sockets).
- [ ] The agent's calls 12 and 13: a scratch file the developer makes in `$TMPDIR` is moved to the Trash **with Finder**
  and put back with Finder's Put Back (it lands in `~/.Trash`); the system sweep's log answers whether it runs at
  start. The category's texts are corrected if the Mac shows otherwise.
- [ ] The scan's time against `main`'s on the same home (rule 4: within 5 %), and the two places' share of it.

## Files

- `docs/plans/017-temp-files/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built,
  what changed from the design and why, the numbers (tests, coverage, the scan's time), the agent's calls taken while
  building, what the real Mac showed.
- `docs/categories/temp-old-files.md`, `modules/temp/locales/en.json` — corrected where the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Waits for the user: nothing above is ticked. The results (the sandbox walk, the read-only commands against the scan,
Finder's Put Back of a scratch file, the system sweep's log, the scan's time against `main`, and the category's texts
corrected if the Mac differs) are written here when the checklist is done. What was built and what is open is in the
plan's conclusions ([`../readme.md`](../readme.md)).
