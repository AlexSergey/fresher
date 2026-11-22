# 031/03/02 — The runs, the conclusions and the review list

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs, modules

A checklist for the user, the numbers of the last code commit, the plan's conclusions, and its section of the user's
review list. The agent runs every automated check and writes the conclusions; the boxes by hand are the user's.

## In the sandbox (the user)

- [ ] `rm -rf sandbox/xcode && npm run dev:sandbox -- xcode`. Scan; walk `EXPECTED.md` line by line, negatives first:
  nothing of a negative is offered by `xcode` (`Notes`, the folder without symbols, the link, Xcode's key bindings,
  the fake archive, the simulator device); the socket case reads as kept with its reason; each positive is offered in
  its category at its level: `iOS 15.0 (19A346)` and `watchOS 10.0 (21R355)` Low, "Xcode previews" and "Simulator
  caches" Safe, `Shop 01-03-2026, 10.15` High; every reason names a size.
- [ ] The storage map: every offered item in Xcode's line.
- [ ] Select everything; the review names High; confirm to the Trash: the two device support folders and the archive
  are in `home/.Trash`; `Previews` and `CoreSimulator/Caches` are still there and empty. Cleanups → put back: the
  archive is back, byte for byte.
- [ ] `npm run dev:sandbox -- xcode-open`: the three items are kept "Xcode or the Simulator is open…".
- [ ] `npm run dev:sandbox -- dev-caches`: DerivedData is still `dev-caches`' and nothing of it is offered by `xcode`.
- [ ] Ignore one archive, then the category, then the module; restore each.

## On the developer's Mac (a dry run, the user)

`npm run dev`, a scan of `~` (a development build on the real Mac makes every cleanup a dry run). Compare with these
read-only commands (never run `simctl`: see the design's facts):

```sh
du -sh ~/Library/Developer/Xcode/* ~/Library/Developer/CoreSimulator/* 2>/dev/null
ls ~/Library/Developer/Xcode/*DeviceSupport 2>/dev/null
ls ~/Library/Developer/Xcode/Archives/* 2>/dev/null
ls ~/Library/Developer/Xcode/UserData/Previews 2>/dev/null
```

- [ ] Every offered item checked by hand: its folder, its size against `du`, its title.
- [ ] Nothing of `UserData` but `Previews`, nothing of `DerivedData` (that is `dev-caches`), no simulator device.
- [ ] With Xcode open: everything left out; quit it and rescan: offered.
- [ ] The facts "to verify", each corrected in `places.toml`, the names, the texts and the category docs where they
  differ: the device support folders' names (with or without a model prefix) and their `Symbols` sign, on a Mac that
  has connected a device; the previews' folder and its `Simulator Devices` sign on a recent Xcode; an archive's
  `Info.plist` and the dated folder's name; whether `CoreSimulator/Caches` holds anything but `dyld`.

## Files

- `docs/plans/031-xcode/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was built, the
  numbers below, what changed from the design and why, the calls taken while building, the by-hand mutation checks of
  02/01, and "awaiting the user's walk". Statuses: the global plan, its plans and steps 🔨 in progress, "awaiting the
  user's walk" (it becomes ✔️ done only after the user has looked at the result).
- `docs/plans/README.md` — the row of 031: 🔨 in progress.
- `docs/deferred/user-review-2026-11-22.md` — create it if it does not exist (head: `# What to look at — 2026-11-22`,
  one line saying it lists what the user asked to check after the autonomous session of 2026-11-22, one section per
  plan), else append. This plan's section, `## 031 — Xcode and simulators`, short bullets:
  - **Walks by hand**: the sandbox boxes above and the dry run on the Mac (links to this step and to
    `docs/manual-testing.md`'s items).
  - **The big change from the draft**: simulator devices deferred (agent's call 1), with the facts: `xcrun simctl list`
    started `xcodebuild -runFirstLaunch` on this Mac (stopped, nothing installed); confirm or overrule; link to
    `xcode-simulator-devices.md`. Also: plan 032 now builds the sandbox's tool world.
  - **Facts not verified**: the device support, previews and archive paths, names and signs (this Mac has none).
  - **Calls worth a second look**: archives High (nothing pre-selected); the two newest device support versions
    offered at Low with their own reason; one app list (Xcode or the Simulator) for every category; every finding in
    Xcode's line; no scan bench.
  - **Known gaps**: archives in a custom folder not read; `~/Library/Caches/com.apple.dt.Xcode` not offered; simulator
    devices and runtimes; `XCTestDevices` and `DVTDownloads` not looked at.
- `docs/deferred/README.md` — the row of `user-review-2026-11-22.md` (`Agreed` 2026-11-22, by the user; `Due at` the
  user's review after the session of 2026-11-22; `What` the walks, facts and calls to check, one section per plan), if
  the file is new.

## Checks (the agent)

- `npm run pre-push` green on the last commit; numbers recorded: Rust tests and coverage (floor 90 %), the crate's own
  coverage, `fresher-e2e --test xcode`, UI tests and coverage, Playwright in chromium and webkit.
- `git diff $(git merge-base HEAD main) -- crates/core crates/scanner crates/cleaner crates/engine crates/sandbox` is
  empty, written in the conclusions.
- `npm run sandbox -- xcode` and `-- xcode-open` generate; `EXPECTED.md` read once by the agent against the goldens.

## Conclusions

The numbers, the empty diff and the changes are written in the plan's [`readme.md`](../readme.md), "Conclusions". The
boxes above are the user's and are not ticked.
