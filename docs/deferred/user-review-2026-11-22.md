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

## 037 — The Trash holds the freed space

Plan [037](../plans/037-trash/readme.md), awaiting the user's walk.

- **Walks by hand**: the sandbox walks (`trash-empty` and `trash`, each also with `--real-trash`) and the dry run on
  the real Mac, items 1.17 and 2.15 of [`../manual-testing.md`](../manual-testing.md), step
  [037/04/01](../plans/037-trash/04-finish/01-docs-and-conclusions.md).
- **Your decision kept**: the app never empties the Trash and never offers to; no button anywhere is named so, and a
  test and an e2e scenario fail on one.
- **Facts not verified**: Finder's 30-day key (`defaults read com.apple.finder FXRemoveOldTrashItems` prints `1` with
  the setting on); `open ~/.Trash` opens Finder's Trash window with and without Full Disk Access; a USB disk's
  `.Trashes/<uid>` gets its own line named after the disk; the time `get_trash` takes on a Mac with a long history
  (it reads the whole journal on each call).
- **Agent's calls worth a second look**: 3 (Show in the Trash kept on the result, Open Trash only on the line), 4 (the
  first number is the last scan's measure plus the cleanups since, stale until the next scan after the user empties
  the Trash), 5 (the date for the app's share only), 7 (every golden gains a `trash` section), 10 (refresh on focus),
  11 (the history's per-page sum); and what was decided while building: only the known Trash folders count (the home's,
  a volume's `.Trashes/<uid>`, and with `--real-trash` the Mac's home Trash), so an item that landed elsewhere is
  never a line; the scanner now shares the composition's clock (no change in the normal app, where both were the
  system clock; it keeps the scan's date and the journal's times comparable in sandbox mode); `--real-trash` shows the
  real home Trash as a second known Trash; Finder's setting is read through `defaults` and never written.
- **Known gaps**: the stale first number ([known issue](../known-issues/app-trash-waiting-stale-after-emptying.md));
  the history's per-page sum; a second volume's Trash measured only when a scan root holds it; the sandbox clock
  restarts at each launch, so after a relaunch the line can count cleanups twice
  ([known issue](../known-issues/app-sandbox-clock-restarts-each-launch.md)); sandbox dates follow the generation's
  clock ([known issue](../known-issues/app-sandbox-scan-dated-at-generation.md)).
- **A test baseline to look at**: the chromium screenshot `category-details-app-leftovers-chosen` failed even alone in
  one full run of the plan's last Playwright pass. It is the known flaky baseline (item 1.14 of the manual testing doc); on the final
  commit it passes alone and fails when its whole spec runs (the diff is the whole page drawn at another scroll
  offset, not a Trash change, as the details screen has no Trash line). It was not run on main from this branch.

## 032 — Docker, described and never cleaned

Plan [032](../plans/032-docker/readme.md), awaiting the user's walk.

- **Walk by hand**: `rm -rf sandbox/docker && npm run dev:sandbox -- docker`, and `docker-stopped`, `docker-no-answer`
  and `docker-left-behind`: every line of `EXPECTED.md`, the section "Described, not cleaned", the insight screen, Copy
  and a paste into a text editor, nothing selectable, Review unaffected, the banner naming "tools" (items 1.19 and 2.17
  of [`../manual-testing.md`](../manual-testing.md)). Copy in the built app too (`npm run tauri build`), not only the
  dev web view: the clipboard in the built app is unverified.
- **Your decision kept**: Docker is described only and never cleaned; nothing of it is selectable, counted or
  journaled, and the app runs no prune. It shows the command, the user runs it.
- **New and generic**: an `Insight` output in the core (what a module describes and never offers), the section
  "Described, not cleaned" on the scan screen and an insight screen with Copy. Only Docker Desktop and OrbStack are
  built; Colima, Lima, Podman machines and Rancher Desktop are deferred
  ([`docker-other-runtimes.md`](./docker-other-runtimes.md)).
- **How the module asks**: it runs the runtime's own CLI, `<app>/<cli> --context <ctx> system df --format json`, only
  when the app is installed and running and the walk saw its socket, never on a restored scan and never to start a
  runtime. Output over 64 KiB is refused. The `left` card ("not installed any more, its disk file is left") is said
  only on a complete installed-apps listing. The reason a read is unreadable is not logged (the module has no tracing).
- **Facts not verified (read-only, on a Mac with the runtime running; the answer's format was taken from the
  documentation and is mocked)**:
  - `docker --context orbstack system df --format json` prints the four lines in the documented shape (string `Type`,
    `TotalCount`, `Active`, `Size`, `Reclaimable`; sizes in powers of 1000; ` (N%)` after `Reclaimable`).
  - Docker Desktop's CLI is at `Contents/Resources/bin/docker` in its bundle and its context is `desktop-linux`;
    `Docker.raw`'s place (`~/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw`).
  - OrbStack's disk file (`~/Library/Group Containers/HUAQ24HBR6.dev.orbstack/data/data.img`) and whether its length
    is a limit (agent's call: hidden).
  - Whether a runtime's CLI starts it when it is stopped (the module asks only a running one).
  - How each runtime gives freed space back (the two `gives-back` sentences).
  - `docker volume prune` removing only unnamed volumes on the installed Docker version.
  - Whether `Reclaimable` of images and build cache matches what `--all` frees.
- **Calls worth a second look** ([`00-design.md`](../plans/032-docker/00-design.md)): the generic `Insight` in the core
  (call 8); only runtimes with an app (call 2); the four commands and `--context` (call 3); no storage-map change
  (call 5); nothing asked in a restored scan (call 6); sandbox mode running no program but `/usr/bin/open` (call 9;
  the autotest exempts only the trial tool's two reads); the parts' levels (call 12). Decided while building: the
  `left` card only on a complete listing, and the output cap.
- **Known gaps**: a UI that connects after a restore shows no insight until the next scan; `npm run sandbox -- all`
  leaves three of Docker's four case files out (the runtimes' fixed paths); the other runtimes are deferred; the
  unreadable reason is not logged.

## 038 — Demo recordings

Plan [038](../plans/038-demo-recordings/readme.md), awaiting the user's walk.

- **Walks by hand** (items 1.21 and 2.19 of [`../manual-testing.md`](../manual-testing.md)):
  - quit any development window; `npm run demo:check -- all`: both scenarios print `played in N s` (N at most 20);
  - give the terminal the Screen Recording permission (System Settings, Privacy & Security, Screen & System Audio
    Recording); set a plain wallpaper (the sidebar's blur shows it); `npm run demo:record -- overview`;
  - look at `assets/demo/overview.gif` and `NO_COMMIT/demo/overview.mp4`: the window only, no cursor of the Mac, the
    drawn pointer, no banner, no real path or name, sizes in gigabytes, the world's disk in the ring, under 5 MiB;
  - commit the GIF if it is good (plan 040 places it in the README);
  - `npm run dev:sandbox -- fixtures/demos/developer-mac.toml`: the world looks believable by hand (the banner shows
    there, as in any sandbox).
- **Facts not verified** (nothing was played or captured while building):
  - `avfoundation` screen capture on macOS 27 with the terminal's Screen Recording permission (if it fails, switch
    `captureCommand` to `screencapture -v`); that `"Capture screen 0"` is the main display's device name;
  - that `tauri dev` ends when the app exits; the crop of the window's rectangle (rounded corners, the overlay title
    bar); the cleanup's result and the freed-space line with a stated volume; how long the first build takes before the
    window appears;
  - that the finder and the scroll step behave in the real window as they do in the unit tests.
- **Agent's calls worth a second look** ([`00-design.md`](../plans/038-demo-recordings/00-design.md)):
  - the GIF committed with a 5 MiB exception in the staged-files guard ([`../hacks/`](../hacks/README.md)), or hosted
    as a release asset;
  - the stated `allocated` fact and the stated volume, which apply only in a sandbox (the volume is not named in the
    banner); the vibrancy kept; the two demo command names in the release bundle's command table;
  - the player exists only in development builds (the recorder runs `tauri dev`);
  - demo folders are new on every run, `/private/tmp/fresher-demo/<scenario>-<n>`, and nothing is ever removed (no
    second deletion point): old folders stay until macOS clears `/private/tmp`;
  - the player refuses at run time any control named like a permanent delete, `Delete` or `Remove`, by prefix: it may
    over-refuse a harmless control (safer);
  - the demo world leaves out xcode, docker, trash, jetbrains and Ollama (reasons in its head comment).
- **An accessibility gap worth a fix**: every category card's "Details" button has the same accessible name, so a
  scenario cannot open one category and a screen reader hears the same name again and again. A name such as "Details
  of {name}" fixes both; it is a UI change outside this plan.
- **Known gaps**: no scenario for the screens of plans 031 to 037 yet
  ([deferred](./demo-scenarios-for-new-screens.md)); no CI run of `demo:check` (it needs a window); automatic
  screenshots not built.

## 035 — The largest files

Plan [035](../plans/035-large-files/readme.md), awaiting the user's walk.

- **Walks by hand**: `rm -rf sandbox/large-files && npm run dev:sandbox -- large-files` (every line of `EXPECTED.md`,
  the filters, Folders, a withheld row's sentence, tick, Review, confirm, put back, rescan: item 1.20 of
  [`../manual-testing.md`](../manual-testing.md)); Largest on the real Mac after a scan of `~`, against `find` and `du`
  (item 2.18).
- **Facts not verified**: whether Spotlight's last-used date and the access time behave on another Mac as on this one
  (checked here on macOS 27.2.1; the last-used date is mostly empty); whether any app keeps a large, hidden-free folder
  in the home that should not be offered (a legacy sync client's folder); how many items a real home holds over 100 MB
  and whether 10 000 is enough; the items query's time on the real home (8.7 ms on a synthetic list of 10 000).
- **Rulings worth a second look** ([`00-design.md`](../plans/035-large-files/00-design.md), agent's calls):
  - 2: a picked own file is a new core category `core.own-file` (High, never pre-selected, Trash by default), judged
    with one judgement (`judge_reached`) at the pick, at prepare and at confirm;
  - 3: "Changed", not "Last opened"; 5: ranked by allocated size (cloud-only files are not listed);
  - 7: removability left to the rehearsal; 8: no sync-client marker; 9: hidden names never offered;
  - 12: a claimed place is skipped even when its category is ignored: a folder holding any claimed node is withheld;
    a folder holding an ignored path is withheld but opens;
  - 13: 10 000 items; the minimum size setting offers 10, 50, 100, 500 MB and 1 GB, default 100 MB.
  - Withheld rows: Library, the Trash, hidden names, app bundles, package interiors, claimed module folders (ignored or
    not), protected and standard folders (the protection list and the cleaner's re-check), folders holding a claimed
    node.
- **Built beyond the design**: `TextKey` accepts a number-hyphen-word segment (`10-mb`); `get_storage_place` is
  removed (the deferred place-queries item is closed); a pre-existing sandbox bug is fixed: a stated access was
  compared with paths without expanding `~`, so a denied folder was still scanned (the `downloads-denied` golden
  changed, and a granted folder whose switch is off is no longer scanned in a hand walk).
- **To confirm**: a denied guarded folder's remainder says "all files" rather than its own capability (engine
  behaviour); the confirmation has no separate High tick yet (plan 039).
- **Known gaps**: [`largest-files-more.md`](./largest-files-more.md); the folders' sizes after a cleanup stay the
  scan's until a rescan; a folder's "Changed" is its own time.

## 036 — Uninstall an app

Plan [036](../plans/036-uninstall/readme.md), awaiting the user's walk.

- **The top item to review: `/Applications` opens its whole app bundles, for an uninstall only**
  ([`deletion-safety.md`](../restrictions/deletion-safety.md), [`removal.md`](../architecture/removal.md)). The walls:
  - the protection is lexical: only `<folder>/<name>.app` and `<folder>/<sub>/<name>.app` pass; nothing inside a bundle,
    the folder itself or `<folder>/<sub>`; Fresher's own bundle never;
  - a link named `X.app` passes the protection by its name, so the listing and the uninstall plan refuse links (the
    listing reads each bundle with `lstat`), and the cleaner opens no link;
  - a bundle that matches a never or ignored place ignoring case is refused (`fresher.app` against Fresher's own);
  - the plan is made only from the listing and refused unless the bundle's canonical path equals the listed path byte
    for byte and is a real folder with `Contents/Info.plist`, at prepare and at confirm;
  - limits: no Unicode case folding or normalisation in the protection (mitigated by the canonical check), and the
    cleaner's own check compares names as written for every protected place (a pre-existing gap, not touched because
    the cleaner is frozen in this plan): a **security-audit item** for plan 041
    ([`../known-issues/core-protection-passes-a-bundle-shaped-link.md`](../known-issues/core-protection-passes-a-bundle-shaped-link.md),
    [`../known-issues/cleaner-protection-compares-names-as-written.md`](../known-issues/cleaner-protection-compares-names-as-written.md)).
- **A behaviour change outside the Apps screen**: ordinary cleanups now refuse `~/Applications` as well as
  `/Applications` (a Largest pick of a folder there is left out as protected); only an uninstall opens them.
- **Who is offered**: App Store apps (owned by root) and any root-owned app are listed and not offered, with Show in
  Finder; Homebrew casks are listed and not offered (installed by the manager, with its command to copy); an
  incomplete installed list (an unreadable folder) keeps every part of every app but the bundle, so only the bundle
  goes.
- **An uninstall always goes to the Trash** (the controller's ruling on the final review): its confirmation offers no
  "Delete permanently", and the engine refuses a permanent confirmation of it (`PlanMismatch`), nothing acted on.
- **App Management is learned passively**: `NotAsked` until the first uninstall, then `Granted` once the bundle moved
  (also when only some of the other parts went) or `Denied` after any Trash refusal of the bundle; an uninstall from
  `~/Applications` teaches nothing. That is over-broad (the cleaner maps every refusal to one failure) and macOS's
  behaviour is unverified
  ([`../deferred/app-management-denied-from-not-allowed.md`](../deferred/app-management-denied-from-not-allowed.md)).
- **`id -G` is run** (read-only, through the command runner, by full path, fails closed) to read the user's groups, so
  an app in `/Applications` owned by root and the admin group is told from one that needs an administrator.
- **The default choice**: "keep its settings and data" is Medium and also moves the launch agents (login items) with
  the caches, logs and saved windows; "everything it left" is High and adds settings, website data, support,
  containers and group containers.
- **Walks by hand**: `rm -rf sandbox/uninstall && npm run dev:sandbox -- uninstall`, walk `EXPECTED.md` line by line,
  negatives first (Chess and Notes not listed; Brewed shows Homebrew's sentence and command; Rooted shows "needs an
  administrator" and Show in Finder; Keeperx's cache untouched); uninstall Keeper with the default choice, then Purger
  with everything; read the confirmation (one item, "and N more files"); Cleanups, put back each; the Twin case.
  `npm run dev:sandbox -- uninstall-open`: Mailbox shows "open"; Sketch goes (item 1.22 of
  [`../manual-testing.md`](../manual-testing.md)).
- **The dry run on the developer's Mac** (`npm run dev`, a scan of `~`, then Apps; item 2.20): the list against
  `ls -l /Applications` (owners), `ls /opt/homebrew/Caskroom` (the Homebrew apps), the sizes against `du -sh
  /Applications/<app>.app` for three apps; the App Store apps not offered; an uninstall of one app is "a dry run"
  (nothing moves).
- **Facts not verified**: whether macOS asks for App Management, refuses silently or shows its own notice on the first
  bundle moved (a release build, one throwaway app the user installs for it); the pane's address
  `Privacy_AppBundles`; whether moving a bundle with the Trash call keeps Finder's "Put Back"; how long the first
  measure takes on a Mac with Xcode.
- **Agent's calls worth a second look** ([`00-design.md`](../plans/036-uninstall/00-design.md)): 3 (last used), 5 (App
  Store apps not offered), 7 (places as code), 8 (more than 64 paths refused), 9 (one item of several paths), 10 (the
  apps folder's opening), 14 (the selection kept). From the build: a vendor app whose group containers carry another
  team's prefix keeps them (`team-shared`), so a TeamPrefixed vendor's data stays; a part named only by a helper id
  outside the app's own id prefix is kept. From the final review: another installed app whose team id cannot be read
  never counts as the same team, so with "everything" a group container of the app's own team may go to the Trash
  while that other app still uses it (it comes back by put back).
- **Known gaps**: files under the app's name, system-wide files, containers without other apps' data, no watch mode
  ([`../deferred/uninstall-system-files.md`](../deferred/uninstall-system-files.md)); the scan folds small Library files,
  so a settings plist or a launch agent under 1 MiB may be missing from the footprint and an uninstall leaves it
  ([`../deferred/uninstall-small-library-parts.md`](../deferred/uninstall-small-library-parts.md)); an app opened
  between the confirmation and the move is not asked again
  ([`../known-issues/cleaner-app-opened-after-the-confirmation.md`](../known-issues/cleaner-app-opened-after-the-confirmation.md)).
