# The user's review of 2026-11-22

**Agreed:** 2026-11-22, by the user's delegation (build autonomously, write down what the user must look at) ·
**Due at:** the user's next session

## What is postponed

Walks, unverified facts and agent's calls of the plans built on 2026-11-22. Each plan has a section; when the user has
looked at a plan's result, its section is removed, and the file and its row with the last one.

## 033 — Logs and crash reports

Plan [033](../plans/033-logs/readme.md), awaiting the user's walk.

- **Walks by hand**:
  - `rm -rf sandbox/logs && npm run dev:sandbox -- logs`; walk `EXPECTED.md` line by line, negatives first: nothing of a
    negative is offered by `logs`; the fresh, six-day, socket, system and locked cases read as kept with their reasons.
    Each positive is offered at Safe with its title and reason.
  - Select all, Review names Safe, confirm to the Trash. Every log folder is still there. Cleanups, put back.
  - The setting raised to 14 days and a rescan. Ignore one item, the category and the module; restore each.
  - `npm run dev:sandbox -- jetbrains` and `-- app-leftovers`: their logs are still offered by their own modules only.
- **The dry run on the developer's Mac** (`npm run dev`, a scan of `~`):
  - Every offered file is under `~/Library/Logs` and older than 7 days: `find ~/Library/Logs -type f -mtime +7 | wc -l`
    beside the findings' count.
  - Nothing under `Logs/JetBrains` or a `com.*`-shaped folder is offered.
  - The largest ten findings, looked into read-only, for state that is not a log.
  - How many findings a real Mac shows and whether Details stays usable (agent's call 4); the scan time beside plan
    030's (agent's call 6).
- **Facts not verified**:
  - The macOS version the reports' layout was read on (27.2.1).
  - Whether Console still lists a report after it is put back from the Trash.
  - Whether any app on the Mac writes logs deeper than six names.
- **Rulings worth a second look** (the design's agent's calls, [`00-design.md`](../plans/033-logs/00-design.md)):
  - Call 1: `logs` claims every file under `~/Library/Logs` whatever its name, at Safe: the second exception of
    "Unknown is not junk" ([`deletion-safety.md`](../restrictions/deletion-safety.md)). If you reject it, the module
    shrinks to known names.
  - Call 3: top-level dotted files (`system.log`, `install.log`) are offered, while a bundle-id-shaped folder stays
    `app-leftovers`'. Changed in a fix round: refusing every bundle-id-shaped name hid almost every top-level log.
  - Call 5: no app attribution and no running-app check. A log held open is protected only by the cleaner's open-file
    check, which does not see other users' processes.
  - Calls 2 and 7: installed apps' id-named log folders are deferred, against the draft; state and lock suffixes are
    not claimed.
- **Known gaps**: the deferred item [`logs-more-places.md`](./logs-more-places.md); the logs of other users'
  processes held open are not seen by the cleaner ([known issue](../known-issues/cleaner-other-users-processes.md)).

## 034 — Browser and web-engine caches

Plan [034](../plans/034-browser-caches/readme.md).

- **Walks by hand**:
  - `rm -rf sandbox/app-caches-browsers && npm run dev:sandbox -- app-caches-browsers`. Walk `EXPECTED.md`: no negative
    is offered; Chromium's three caches and Redis Insight's `Cache` are offered at Low, titled `Chromium, Default:
    Cache` and the like; the storage map shows `Profile 2` in your folders and the caches in Chromium's line; select
    all, confirm to the Trash, each cache folder is still there and empty, put back restores.
  - `npm run dev:sandbox -- app-caches-browsers-open`: Chrome's two caches are kept with "This app, or a part of it, is
    open…"; Redis Insight's `GPUCache` is offered.
- **Dry run on the real Mac** (`npm run dev`, a development build is a dry run): with Chrome, Chromium and Firefox
  closed, each offered cache is checked against `du -sh ~/Library/Caches/Google/Chrome/*/{Cache,"Code Cache"}
  ~/Library/Caches/Chromium/*/{Cache,"Code Cache"} ~/Library/Caches/Firefox/Profiles/*/{cache2,startupCache}`; with
  Chrome open, its caches are left out; nothing under `Application Support/Google`, `.../Firefox` or `.../Chromium` is
  offered; `~/Library/Caches/Google/Chrome-headless` is not offered. The items are 1.16 and 2.14 of
  [`../manual-testing.md`](../manual-testing.md).
- **Facts not verified here**: Redis Insight's caches with the app open and closed (not run on this Mac); whether a
  browser accepts its emptied `Cache` folder on the next start without a warning (expected: yes; check Chrome and
  Firefox once after a real cleanup in a release build); Firefox's `startupCache` emptied while the profile is closed.
  The second sandbox case file puts Redis Insight at `Applications/RedisInsight.app` (the brief said `Redis
  Insight.app`): the shared sandbox world leaves out a case file that writes a path an earlier file wrote; the bundle
  id and the name are the same.
- **Candidate rows to check on a real install** (each needs the bundle id from `Info.plist`, the folders with `ls`,
  then a row): Microsoft Edge (`com.microsoft.edgemac`, expected `~/Library/Caches/Microsoft Edge/<profile>/`), Brave
  (`com.brave.Browser`, `~/Library/Caches/BraveSoftware/Brave-Browser/<profile>/`), Vivaldi, Arc; Visual Studio Code
  (`com.microsoft.VSCode`, `~/Library/Application Support/Code/`), Slack, Discord, Cursor; all expected, none checked.
  Only four rows ship: Chrome, Chromium, Firefox and Redis Insight.
- **Agent's calls worth a second look**: one finding per cache folder rather than per profile (design call 5); only the
  four checked rows ship (call 4); no browser support-folder caches, they need Full Disk Access to verify (call 6);
  Firefox's `startupCache` in, `safebrowsing` and `thumbnails` out (call 7); the title shows the raw folder name
  (`Code Cache`, `cache2`) (call 8); the calls taken while building, in the plan's
  [conclusions](../plans/034-browser-caches/readme.md#conclusions).
- **Known gaps**: whatever the plan's conclusions list (one webkit hover flake in the UI run, which passed on rerun; the
  chromium baseline `category-details-app-leftovers-chosen` known flaky).

## 031 — Xcode and simulators

Plan [031](../plans/031-xcode/readme.md), awaiting the user's walk.

- **Walks by hand**:
  - `rm -rf sandbox/xcode && npm run dev:sandbox -- xcode`. Walk `EXPECTED.md` line by line, negatives first: nothing of
    a negative is offered by `xcode`; the socket case reads as kept with its reason; the two device support folders are
    Low, "Xcode previews" and "Simulator caches" Safe, the archive High; every reason names a size. The storage map has
    every offered item in Xcode's line. Select all: the review names High; confirm to the Trash; the previews and the
    simulator caches are still there and empty; put back restores the archive byte for byte.
  - `npm run dev:sandbox -- xcode-open`: the three items are kept "Xcode or the Simulator is open…".
    `npm run dev:sandbox -- dev-caches`: DerivedData is still `dev-caches`'.
  - The dry run on the Mac against `du` and `ls` (items 1.18 and 2.16 of [`../manual-testing.md`](../manual-testing.md),
    step [031/03/02](../plans/031-xcode/03-finish/02-run-and-conclusions.md)). In `npm run sandbox -- all` with
    Xcode open (from `xcode-open`) the positives of `xcode` are left out as open: walk them by name.
- **The big change from the draft**: simulator devices are deferred ([`xcode-simulator-devices`](./xcode-simulator-devices.md)).
  During planning `xcrun simctl list` started `xcodebuild -runFirstLaunch` on this Mac (killed, nothing installed), so
  the module runs no command; confirm or overrule. Plan 032 now builds the sandbox's tool world.
- **Facts not verified here** (this Mac has no device support, previews or archives): the device support folders'
  names (with or without a model prefix, and whether a suffix follows the build, e.g. ` arm64e`, seen on some Xcode
  versions: such a name is not claimed today, a safe miss) and their `Symbols` sign; the previews' folder and its `Simulator Devices`
  sign on a recent Xcode; an archive's `Info.plist` and the dated folder's name; whether `CoreSimulator/Caches` holds
  anything but `dyld`. A difference is a correction in `places.toml`, the names, the texts and the category docs.
- **Agent's calls worth a second look**: archives are High (nothing pre-selected; only Safe goes in bulk); the two
  newest device support versions are offered at Low with their own reason; one list of apps (Xcode or the Simulator)
  for every category; every finding in Xcode's line; no scan bench. A file held open inside the simulator caches skips
  the whole item (`InUse`), as the cleaner does for any emptied folder; the plan's step 02/02 was corrected.
- **Known gaps**: archives in a custom folder are not read; `~/Library/Caches/com.apple.dt.Xcode` is not offered;
  simulator devices and runtimes; `XCTestDevices` and `DVTDownloads` were not looked at.
