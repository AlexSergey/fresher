# 018/04/02 — The runs by hand and the conclusions

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** docs

## In the sandbox

- [ ] `rm -rf sandbox/dev-caches && npm run dev:sandbox -- dev-caches`. Scan; walk `EXPECTED.md` line by line,
  negatives first: nothing of a negative is offered; the left-out lines read as written under "Xcode build data"; each
  positive is offered in its category at its danger (seven Low, Go's and the project Safe); the project counts in
  Xcode's line of the map.
- [ ] Select every positive, confirm to the Trash: Low on the confirmation; the items land in `home/.Trash`; Cleanups →
  put back: they are back. Cargo's `index`, Homebrew's links and the link's target in `home/elsewhere/Link` are
  untouched.
- [ ] Ignore npm's category, then the module; restore.
- [ ] `npm run dev:sandbox -- dev-caches-unsigned`: nothing offered, nothing left out.
- [ ] `npm run dev:sandbox -- dev-caches-xcode-open`: the project left out "Its app is open…"; npm's cache offered.

The check that a changed expectation fails the autotest, once per kind (a danger, a left-out's items, a sign), was made
while building (03/01's conclusions).

## On the developer's Mac (read-only)

`npm run dev`, a scan of `~` only (a development build's cleaner reaches only a sandbox, so nothing can be removed),
compared with these read-only commands:

```sh
du -sh ~/.npm/_cacache ~/.gradle/caches ~/.cargo/registry/{cache,src} \
  ~/Library/Caches/{Yarn,Homebrew/downloads,pip,go-build,CocoaPods} ~/Library/Developer/Xcode/DerivedData/* 2>/dev/null
ls ~/.npm/_cacache ~/Library/Caches/Yarn ~/Library/Caches/Homebrew ~/Library/Caches/pip 2>/dev/null
head -c 43 ~/.gradle/caches/CACHEDIR.TAG; echo; head -c 43 ~/.cargo/registry/CACHEDIR.TAG; echo
ls ~/Library/Caches/go-build 2>/dev/null | head; ls ~/Library/Caches/CocoaPods 2>/dev/null
ls ~/Library/Developer/Xcode/DerivedData 2>/dev/null; ls ~/Library/Developer/Xcode/DerivedData/*/info.plist 2>/dev/null
mdls -name kMDItemCFBundleIdentifier /Applications/Xcode.app 2>/dev/null
find ~/.npm/_cacache ~/.gradle/caches ~/.cargo/registry ! -user "$USER" 2>/dev/null | head
find ~/.npm/_cacache ~/.gradle/caches ~/.cargo/registry \( -type s -o -type p \) 2>/dev/null | head
```

- [ ] Every offered item is one of the table's places with its sign; its size matches `du`'s; nothing of Cargo's
  `index`, `git` or `bin`, npm's `_npx` or `_logs`, Gradle's `wrapper`, or Homebrew's `api` is claimed.
- [ ] The facts "to verify" (design, research and agent's calls 2 and 3): Go's `README` and `trim.txt`, CocoaPods'
  `Pods`, pip's `http-v2`, derived data's `<project>-<28 lowercase letters>` and `info.plist`, Xcode's bundle id
  `com.apple.dt.Xcode` — on this Mac where the tool is there, otherwise from the tool's own documentation and a Mac
  that has it. The table and the category texts are corrected where they differ.
- [ ] The scan's time against `main`'s on the same home (rule 4: within 5 %).
- [ ] The scan bench before and after plan 018 on a quiet Mac (`npm run bench`): 01/01's runs were taken on a busy
  one and were inconclusive (±10 % in both directions). Only the empty-holds path is measured, as the scanner
  bench has no interest with held names, so the real-home scan timing against `main`'s (dev-caches is registered in
  every build) is what exercises the held-names path; say so in the conclusions. Known costs: `Hit` 16 to 24 bytes,
  +4.6 % tree heap in the state bench.
- [ ] One real cleanup, by the developer, with Finder (not the app): `~/.npm/_cacache` moved to the Trash, then an
  `npm install` in a project makes it again; the Trash emptied or the folder put back.

## Files

- `docs/plans/018-developer-caches/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was
  built, what changed from the design and why, the numbers (tests, coverage, the bench, the scan's time), the calls
  taken while building, what the real Mac showed.
- `modules/dev-caches/tools.toml`, `locales/en.json`, `docs/categories/dev-caches-*.md` — corrected where the real
  Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

Not written yet.
