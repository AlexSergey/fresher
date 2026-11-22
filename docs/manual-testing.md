# Manual testing: every run by hand, in one list

One checklist of everything that still waits for a person at a real Mac, in the order that costs least. Each item has a
checkbox, a link to the plan step it comes from, the commands, what to look for, and where to write the result (the
step's `## Conclusions`, on `main`). Items already done and written down are not here. Last compiled 2026-11-21.

Read the step only when an item is not clear: each one is meant to be run from here.

Plans 027, 028, 023, 024, 029 and 030 were squash-merged into `main` on 2026-11-21 before their walks, at the user's
request; they stay 🔨 until their items here (1.9, 1.11 to 1.14, 2.8, 2.10 to 2.12) are run and written down. The agent's
calls taken while building them are in each plan's `00-design.md`.

## 0. Before you start

### Prerequisites

- [ ] Node 24.15 or newer, npm 11 or newer, and `node_modules` present: `node -v && npm -v && ls node_modules | head -1`;
      if missing or stale, `npm ci`.
- [ ] The Rust toolchain of `rust-toolchain.toml` (1.99.1): `rustc --version`; `cargo nextest --version` (the tool is
      installed once with `cargo install --locked cargo-nextest`).
- [ ] Playwright browsers for the UI e2e, once: `npx playwright install webkit chromium`.
- [ ] Free disk space: 20 GB or more (a release build, the bench trees under `target/bench-trees/`, a second worktree
      for the bench baselines).
- [ ] A quiet Mac for section 4 only: on power, no other build or sync running, `uptime` load average under 2.
- [ ] Note down the state to start from, so each item can be read against it: `sw_vers -productVersion`, and whether
      the terminal app that starts the dev build (Terminal, iTerm, the editor) has Full Disk Access (System Settings,
      Privacy and Security, Full Disk Access). Write both at the top of the first Conclusions you fill.

### Start the app

- `npm run dev`: the development build on the real Mac. It always runs as `io.github.alexsergey.fresher.dev`, shows a
  development banner, makes every cleanup a dry run (every check runs, nothing is moved: each item ends "a dry run") and
  records the run in `NO_COMMIT/dev-runs/`. It takes no argument.
- `npm run dev:sandbox -- <name>`: the same build in sandbox mode on `sandbox/<name>/`. It generates the folder when it
  is missing, so a clean start is `rm -rf sandbox/<name> && npm run dev:sandbox -- <name>`. The banner names the
  sandbox; the scan, the settings, the history and the Trash are the sandbox's. `--real-trash` sends items to the
  system's Trash instead of `sandbox/<name>/home/.Trash`. It cleans for real inside the sandbox and is recorded too
  (`..._sandbox-<name>.json`).
- Both commands share one identifier and the app is single-instance: **quit the running window before starting
  another**, or the new launch is handed to the old window and keeps its mode (the banner shows which).
- `sandbox/<name>/EXPECTED.md` is the checklist of a sandbox: one box per case, negatives first. Walk it in order:
  nothing of a negative may be offered; every left-out line reads as written; each positive is offered at its stated
  danger. The folder is gitignored, so ticking boxes in it costs nothing. Details:
  [`testing.md`](./testing.md#sandboxes).
- A sandbox's dates count from its generation (`world.toml` `now`), so a scan may say "made days ago" in a sandbox that
  is older than the run: regenerate with `rm -rf` to avoid it (known issue, see section 6).

### Autotests first (all should be green before you test by hand)

```sh
npm run sandbox:test                      # every case file of fixtures/sandboxes/ against its goldens
cargo nextest run --workspace --locked    # the Rust tests
npm run test:e2e --workspace app/ui       # the UI Playwright suite (npm run test:e2e adds the core e2e)
```

`npm run pre-push` runs the whole gate (lint, guards, Rust and UI tests with coverage) if you want one command; see
[`quality.md`](./quality.md). A red autotest is a finding of its own: stop and record it before the hand runs.
Two known flakes are listed in section 6.

### Safety

- A development build never moves anything outside a sandbox: on the real Mac a cleanup is a dry run. A release build
  (`npm run build`) can, and is used by hand only in sections 3 and 5.
- Every real-Mac command in this list is read-only (`ls`, `stat`, `du`, `mdls`, `mdfind`, `mdutil -s`, `plutil -p`,
  `log show`). The exceptions are marked: a pane address opened with `open`, `tccutil reset`, and one item that mounts a
  disk image.
- Record counts, times and sizes in the Conclusions, never a real path or a file name from your Mac.
- A sandbox cleanup with `--real-trash` puts items into the real Trash: only sandbox items, and you put them back.

## 1. Sandbox runs

Each item: run, scan, walk `EXPECTED.md` (negatives first), then the action listed. Every sandbox is independent; do
them in this order because the first ones teach the screens used by the rest.

### 1.1 The trial sandbox (plans 007, 011, 015)

- [ ] **Plan 007, the trial run** ([007/05/02](./plans/007-safe-development/05-finish/02-run-and-conclusions.md)).
      Record under its Conclusions.
  1. `rm -rf sandbox/trial && npm run dev:sandbox -- trial`: the banner shows; scan; walk `EXPECTED.md` case by case.
     Select and confirm a cleanup to the sandbox's Trash; Cleanups screen, put back: the items return.
  2. Quit. `npm run sandbox -- trial` again (regenerate over the existing folder): the removed items come back and
     nothing else changes.
  3. `npm run dev:sandbox -- trial --real-trash`: one cleanup to the real Trash; put it back (the app, then once
     with Finder's Put Back on another item).
  4. `npm run dev` (quit the sandbox window first): the development banner, no sandbox name; the data folder is the
     development one (`ls ~/Library/Application\ Support/io.github.alexsergey.fresher.dev`).
  5. The release-build part of this step is in section 5.
  6. On a category's details, ignore one item from its row's menu, by pointer and then by keyboard: the row leaves and
     the focus stays in the list; Settings, Ignored, Ignored items lists the path; Restore brings the row back
     without a rescan; a cleanup prepared after the ignore does not list the item.
- [ ] **Plan 011, problems and reasons** ([011/03/01](./plans/011-diagnostics/03-finish/01-docs-and-conclusions.md)).
  1. `npm run dev` (the demo module): scan; the demo category shows "1 item left out (150 MB): The app that made it is
     open. Quit the app, then scan again."
  2. Quit and start again: the restored scan shows no left-out line until the next scan (expected: left-outs are
     event-only).
  3. `FRESHER_SANDBOX=relative npm run dev`: a dialog with a plain sentence and the sandbox's refusal under it; no main
     window flashes; Cmd-Q waits for the dialog; after dismissing, the app exits with code 1 (`echo $?` when started
     from the terminal); nothing was written.
  4. A tool's failure is not reachable by hand yet: the UI e2e screenshot of a failed item with "Details" stands in.
  5. The autotests of section 0 are the "checks" of this step.
- [ ] **Plan 015, module groups** ([015/03/02](./plans/015-module-groups/03-finish/02-run-and-conclusions.md)).
  1. `npm run dev:sandbox -- trial`: scan; Scan's cards of `demo` and `trial` sit in their danger zones.
  2. Settings, Ignored groups, Debug on: Scan updates at once, nothing of the two is offered in
     Review. Off: everything is back with no rescan. Repeat with the module `trial` alone (Ignore module from a card's
     settings menu) and restore it in Settings, Ignored modules.
  3. Quit and open again: the ignored lists are kept. A cleanup prepared before an ignore is refused after it.
  4. `EXPECTED.md` of `trial` walked as in 007: nothing changed in what is offered.
  5. On the real Mac, `npm run dev` shows the group tabs. They read well with VoiceOver (Cmd-F5) and answer the arrow
     keys. (The note that only Debug shows is out of date: the real modules are registered now, so expect Development,
     System and Your files as well.)

### 1.2 The storage map ([012/06/02](./plans/012-storage-map/06-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/storage-map && npm run dev:sandbox -- storage-map`; every line of `EXPECTED.md` holds,
      including the map's lines (the ring and its legend; the legend's "i" explains each part).
- [ ] Ignore and restore a category: its bytes move to "Your folders" and back, the total unchanged, no rescan.
- [ ] During a scan: "Explained" grows as findings arrive; "Explained" plus "Not scanned yet" stays the used space;
      at the end the full map replaces it in one move. Quit and reopen: the restored map shows at once.

### 1.3 JetBrains ([016/04/02](./plans/016-jetbrains/04-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/jetbrains && npm run dev:sandbox -- jetbrains`. Have a real JetBrains IDE open on the Mac if you
      can: the banner says open apps are simulated, and the real open IDE changes nothing.
- [ ] Scan; walk `EXPECTED.md`: nothing of a negative offered; each left-out line reads as written under its category;
      each positive offered at its level and counted in WebStorm's (or Rider's) line of the map.
- [ ] Select every positive, confirm: the Medium level and the settings' sentence are on the confirmation; the folders
      land in `sandbox/jetbrains/home/.Trash`; Cleanups, put back: they return. Repeat with `--real-trash`: Finder's Put
      Back restores them.
- [ ] Ignore the module, then the group Development, from Settings and from a card's settings menu (Ignore module); restore in Settings, Ignored.
- [ ] The two extra case files also run clean: `npm run dev:sandbox -- jetbrains-unknown-app-installed` and
      `jetbrains-unknown-app-open` (their `EXPECTED.md` lists the expected left-outs; not in the step's list, but each has
      its autotest and a look costs a minute).

### 1.4 Temporary files ([017/04/02](./plans/017-temp-files/04-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/temp && npm run dev:sandbox -- temp`. Scan; walk `EXPECTED.md`: negatives not offered; left-out
      lines read as written under "Old temporary files"; each positive offered at Safe; the map lists "Your temporary
      folder" and "Shared temporary folder". The fresh file and the unpacked folder are young by their real status
      change, not by a stated one.
- [ ] Select every positive, confirm: Safe on the confirmation; items land in `home/.Trash`; put back from Cleanups. The
      link's target in `home/Documents/linked` is untouched.
- [ ] Settings, Temporary files: enter 6 (refused: "Enter a whole number from 7 to 30."), then 14 (saved); rescan: the
      10-day item is left out as `recent`; set back to 7.
- [ ] Ignore the module, then the group System; restore.

### 1.5 Developer caches ([018/04/02](./plans/018-developer-caches/04-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/dev-caches && npm run dev:sandbox -- dev-caches`. Scan; walk `EXPECTED.md`: negatives not offered;
      left-out lines read as written under "Xcode build data"; each positive offered at its danger (seven Low; Go's and
      the project Safe); the project counts in Xcode's line of the map.
- [ ] Select every positive, confirm: Low on the confirmation; items land in `home/.Trash`; put back. Cargo's `index`,
      Homebrew's links and the link's target in `home/elsewhere/Link` are untouched.
- [ ] Ignore npm's category, then the module; restore.
- [ ] `npm run dev:sandbox -- dev-caches-unsigned`: nothing offered, nothing left out.
- [ ] `npm run dev:sandbox -- dev-caches-xcode-open`: the project is left out ("Its app is open..."); npm's cache is
      offered.

### 1.6 Uninstalled app leftovers ([019/04/02](./plans/019-uninstalled-app-leftovers/04-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/app-leftovers && npm run dev:sandbox -- app-leftovers`. Scan; walk `EXPECTED.md`: nothing of a
      negative offered (Notes, Apple's files, the tool's, the installed Editor's, the link, the group container, the
      container); left-out lines read as written (running, changed lately, elsewhere, a socket, the system's file); each
      positive offered at its danger (caches Safe, settings Medium, support folder High); the support folder counts in
      its id's line of the map, "no longer installed".
- [ ] On the details of a card of the module (Details): **Keep settings** selects only the caches and Review names Safe; **Remove everything**
      selects the three categories and Review names High. Confirm: items land in `home/.Trash`; put back. The link's
      target in `home/elsewhere/Linked`, the group container and the container are untouched.
- [ ] Ignore the data category, then the module; restore.
- [ ] `npm run dev:sandbox -- app-leftovers-spotlight-off`: nothing offered; the left-out line names Spotlight.

### 1.7 Downloads and installers ([020/05/02](./plans/020-downloads/05-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/downloads && npm run dev:sandbox -- downloads`. First turn on Settings → Permissions →
      Downloads (without Full Disk Access the switch decides, even for a granted folder: 020's decision 12). The banner
      says access and installs are simulated. Scan; walk `EXPECTED.md`: negatives not offered (the document, the disk image of no app, the copy, the
      link, the folder of yours, the big file inside it); left-out lines read as written (changed lately, newer than the
      installed app, downloaded after it, a newer macOS, opened lately); each positive at its danger (unfinished Low
      with the placeholder, Safari Medium, installers Low, the macOS installer Low, the big file High); the installer
      counts in `Example`'s line of the map; `Downloads/Holiday` opens in the browse view.
- [ ] Select everything: Review names High; confirm: items land in `home/.Trash`; put back. The link's target in
      `home/elsewhere/clip` is untouched.
- [ ] Ignore the large-untouched category, then the module; restore.
- [ ] `npm run dev:sandbox -- downloads-denied`: the card's Details show the notice that Downloads is not allowed, with
      the way to allow it; nothing offered; the coverage names Downloads.
- [ ] `npm run dev:sandbox -- downloads-browser-open`: the left-out line names the open browser.

### 1.8 Stale projects ([021/04/02](./plans/021-stale-projects/04-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/stale-projects && npm run dev:sandbox -- stale-projects`. Scan; walk `EXPECTED.md`, negatives
      first: nothing of a negative is offered (the source, `.git`, `dist`, `.next`, the loose `node_modules`, the link,
      the home's own `node_modules`, the hidden folder's and the app bundle's projects, a project inside another's
      `build/`); left-out lines read as written (the project or the folder changed lately, a build used it this week);
      each positive at its danger (packages and build output Low, the Python environment Medium, the Rust caches Low);
      the monorepo's package reads "shop · packages/web/node_modules"; each reason names a date and a command.
- [ ] The storage map: the offered folders in the explained part; `home/Code/kept-source/src` in your folders; the
      browse view opens `home/Code/old-tool/target` (not one opaque entry).
- [ ] Select everything: Review names Medium; confirm: items land in `home/.Trash`; Cleanups → put back. The link's
      target in `home/elsewhere/modules` is untouched.
- [ ] Ignore the build-output category, then the module; restore.

### 1.9 Local AI models ([023/03/02](./plans/023-local-ai-models/03-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/local-ai-models && npm run dev:sandbox -- local-ai-models`. Scan; walk `EXPECTED.md`, negatives
      first: nothing of a negative is offered (the Space, the hub's locks, the folder without `blobs` and `snapshots`,
      the name off the shape, the loose file in LM Studio's folder, Ollama's folder without `manifests`, the project's
      model file, the project's link; Ollama's models are 1.12); the socket case reads as kept with its reason; each
      positive is offered in its category at its danger (Hugging Face Low, LM Studio Medium); titles read `meta/tiny`, `gpt2`, `o/words`, `pub/model-a` and
      `pub/model-b` (a model in LM Studio's older place goes); each reason names a date and a size.
- [ ] The storage map: the offered models in the explained part; `home/.ollama/models` in your folders.
- [ ] Select everything: Review names Medium; confirm to the Trash: the model folders land in `home/.Trash` with their
      `snapshots` links inside; `home/Code/linked/weights.safetensors` is still a link (now broken). Cleanups, put
      back: the folders are back and the link resolves again.
- [ ] `npm run dev:sandbox -- local-ai-models-lm-studio-open`: the LM Studio model is kept "LM Studio is open"; the
      Hugging Face model is offered.
- [ ] Ignore the LM Studio category, then the module; restore.

### 1.10 Also worth one look (no pending step)

- [ ] `npm run dev:sandbox -- cleanup-guards`: the case file of the removal refusals; its `EXPECTED.md` holds. It is
      the one sandbox with no plan step of its own, and 008's run (section 5) leans on its refusals.

### 1.11 Unused toolchain versions ([024/04/02](./plans/024-unused-toolchains/04-finish/02-run-and-conclusions.md))

The module has no page of its own (plan 027): its two categories are cards on Scan and screens of the category Details.

- [ ] `rm -rf sandbox/unused-toolchains && npm run dev:sandbox -- unused-toolchains`. Scan; walk `EXPECTED.md`,
      negatives first: nothing a pointer keeps is offered (rustup's default and override, the moving channels, the
      toolchains pinned in `Documents/dev/pinned`, `Code/legacy` and `Code/asdf`, the link, `my-custom`, nvm's default
      and newest, the alias chain, the `.nvmrc` in `Documents/dev/web`, the `.node-version`); the socket case reads as
      kept with its reason on the category's Details; each positive is offered in its category at its danger (Rust Low,
      Node Medium); titles read `Rust 1.70.0-aarch64-apple-darwin` and `Node v12.22.12`; each reason names a date and
      the command.
- [ ] Settings: the Node versions switch is on (the case file's settings). Turn it off, scan again: no Node version is
      offered, the toolchains still are. Turn it back on.
- [ ] The storage map: the offered versions in the explained part; the pins' folders still open in your folders.
- [ ] Select everything: Review names Medium; confirm to the Trash: the version folders land in `home/.Trash`;
      `settings.toml`, the alias files and every pin are untouched. Cleanups, put back: the folders are back.
- [ ] `rm -rf sandbox/unused-toolchains-unknown && npm run dev:sandbox -- unused-toolchains-unknown`:
      the toolchain is kept "could not read which version is the default"; the Node version is kept "a version
      file could not be read".
- [ ] `rm -rf sandbox/unused-toolchains-off && npm run dev:sandbox -- unused-toolchains-off`:
      the switch is off in Settings; no Node version; the toolchain goes.
- [ ] `rm -rf sandbox/unused-toolchains-documents-denied && npm run dev:sandbox -- unused-toolchains-documents-denied`:
      Scan's cards of the two categories say they need Documents,
      and their Details screen offers "Let Fresher look there"; nothing is offered.
- [ ] Ignore the Rust category from its card's menu on Scan, then the module; restore both in Settings, Ignored.

### 1.12 Ollama's models ([029/04/02](./plans/029-ollama-models/04-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/local-ai-models-ollama && npm run dev:sandbox -- local-ai-models-ollama`. Scan; walk
      `EXPECTED.md`, negatives first: the lone layer and the manifest at the wrong depth are not offered; the incomplete
      model reads as kept with its reason; `llama3:latest, llama3:8b` is one finding; the derived model's reason says
      how large it is in all, how much is freed and that the rest is shared with the base; every Ollama finding is
      Medium.
- [ ] The storage map: the offered models' layers in the explained part; the shared layers and the lone layer in your
      folders.
- [ ] Select the derived model and `mistral:7b`; the confirm dialog names Medium; open "more files" on each: exactly
      the manifest's companions the golden lists. Confirm to the Trash: the manifests and the unshared layers land in
      `home/.Trash`; the shared layers and the base's manifest stay. Cleanups, put back: every file is back.
- [ ] `npm run dev:sandbox -- local-ai-models-ollama-unreadable`: nothing of Ollama is offered, the card says a model
      list could not be read.
- [ ] `npm run dev:sandbox -- local-ai-models-ollama-download`: nothing of Ollama is offered, the card says a download
      is in progress.

### 1.13 Caches of installed apps ([030/03/02](./plans/030-installed-app-caches/03-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/app-caches && npm run dev:sandbox -- app-caches`. Scan; walk `EXPECTED.md`, negatives first:
      nothing of a negative is offered by `app-caches` (`OtherTool`, Apple's `com.apple.Maps`, Fresher's own, the
      removed app's `com.example.Lost`, the vendor's `com.example`, the link `com.example.Reader.Linked`); the socket
      case reads as kept with its reason; each positive is offered in "Caches of your apps" at Low, titled `Scribe` and
      `Player: com.example.Player.ShipIt`, each reason naming the app and a size.
- [ ] The storage map: the offered caches in the explained part, each in its app's line; `home/Library/Caches/OtherTool`
      in your folders.
- [ ] Select everything; Review names Low; confirm to the Trash. `home/Library/Caches/com.example.Scribe` is still
      there and empty, `Cache.db` and `fsCachedData` are in `home/.Trash` (the result counts them as moved to the
      Trash; the cleanup's detail row is where an item with nothing trashed reads "Emptied. The folder itself stays.").
      Cleanups, put back: both entries are back inside the folder.
- [ ] `npm run dev:sandbox -- app-caches-app-open`: Mailer's cache is kept "This app, or a part of it, is open..."; the
      closed Notes app's is offered.
- [ ] `npm run dev:sandbox -- app-leftovers`: the installed Editor's caches are offered by "Caches of your apps", and
      that file's negatives read "never offered by `app-leftovers`", which still holds.
- [ ] Ignore one cache, then the category, then the module; restore each.

### 1.15 Logs and crash reports ([033/03/01](./plans/033-logs/03-finish/01-docs-and-conclusions.md))

- [ ] `rm -rf sandbox/logs && npm run dev:sandbox -- logs`. Scan; walk `EXPECTED.md`, negatives first: nothing of a
      negative is offered by `logs`; the fresh, six-day, socket, system and locked cases read as kept with their
      reasons; each positive is offered at Safe with its title and reason.
- [ ] Select everything; Review names Safe; confirm to the Trash. Every log folder is still there. Cleanups, put back.
- [ ] Settings: raise "Offer logs and crash reports unchanged for" to 14 days and rescan; the files of 7 to 14 days are kept.
- [ ] Ignore one item, then the category, then the module; restore each.
- [ ] `npm run dev:sandbox -- jetbrains` and `-- app-leftovers`: their logs are still offered by their own modules only.

### 1.16 Browser and web-engine caches ([034/03/01](./plans/034-browser-caches/03-finish/01-docs-and-conclusions.md))

- [ ] `rm -rf sandbox/app-caches-browsers && npm run dev:sandbox -- app-caches-browsers`. Scan; walk `EXPECTED.md`,
      negatives first: nothing of a negative is offered by `app-caches`; Chromium's three caches and Redis Insight's
      `Cache` are offered at Low, titled `Chromium, Default: Cache` and the like.
- [ ] The storage map: `Profile 2` in your folders, the caches in Chromium's line.
- [ ] Select everything; confirm to the Trash. Each cache folder is still there and empty. Cleanups, put back:
      every entry is back in its own folder.
- [ ] `npm run dev:sandbox -- app-caches-browsers-open`: Chrome's two caches are kept "This app, or a part of it, is
      open..."; Redis Insight's `GPUCache` is offered.

### 1.18 Xcode and simulators ([031/03/02](./plans/031-xcode/03-finish/02-run-and-conclusions.md))

- [ ] `rm -rf sandbox/xcode && npm run dev:sandbox -- xcode`. Scan; walk `EXPECTED.md` line by line, negatives first:
      nothing of a negative is offered by `xcode` (`Notes`, the folder without symbols, the link, Xcode's key bindings,
      the fake archive, the simulator device); the socket case reads as kept with its reason; each positive is offered
      in its category at its level: `iOS 15.0 (19A346)` and `watchOS 10.0 (21R355)` Low, "Xcode previews" and
      "Simulator caches" Safe, `Shop 01-03-2026, 10.15` High; every reason names a size.
- [ ] The storage map: every offered item in Xcode's line.
- [ ] Select everything; the review names High; confirm to the Trash: the two device support folders and the archive
      are in `home/.Trash`; `Previews` and `CoreSimulator/Caches` are still there and empty. Cleanups, put back: the
      archive is back, byte for byte.
- [ ] `npm run dev:sandbox -- xcode-open`: the three items are kept "Xcode or the Simulator is open...".
- [ ] `npm run dev:sandbox -- dev-caches`: DerivedData is still `dev-caches`' and nothing of it is offered by `xcode`.
- [ ] Ignore one archive, then the category, then the module; restore each.

### 1.17 The Trash line ([037/04/01](./plans/037-trash/04-finish/01-docs-and-conclusions.md))

- [ ] `rm -rf sandbox/trash-empty && npm run dev:sandbox -- trash-empty`. Scan: no Trash line. Clean the trial cache
      to the Trash: the result says "Moved to the Trash", "You get the space back when you empty the Trash.", and the
      invitation under Show in the Trash; back on the scan screen the line reads "Waiting in the Trash: 1 MB, all of
      it from cleanups here." Open Trash opens `sandbox/trash-empty/home/.Trash` in Finder.
- [ ] `rm -rf sandbox/trash && npm run dev:sandbox -- trash`. Scan: "Waiting in the Trash: 2 MB." Clean: "Waiting in
      the Trash: 6 MB, 4 MB of it from cleanups here." Remove the cache's entry from `home/.Trash` in Finder and come
      back to the window: the share is gone, the first number stays until a new scan
      ([known issue](./known-issues/app-trash-waiting-stale-after-emptying.md)). Put back from Cleanups: the share is
      gone.
- [ ] Both again with `--real-trash`: the moved items wait in the Mac's Trash; a second line reads "Waiting in the
      Trash from cleanups here: ... The rest of the Trash was not measured."; Open Trash on it opens the Mac's Trash.
      Put the items back afterwards.
- [ ] No button anywhere says Empty; the Trash line and the result only invite.

### 1.19 Docker, described ([032/06/01](./plans/032-docker/06-finish/01-docs-and-conclusions.md))

Docker is only described: nothing of it is ever offered, selected or cleaned, and the app never runs a Docker command
beyond the one read.

- [ ] `rm -rf sandbox/docker && npm run dev:sandbox -- docker`. The banner names "tools" among what is simulated.
      Scan; walk `EXPECTED.md` line by line: nothing is offered by `docker`; the stale Docker Desktop socket and the
      disk file at no known place say nothing.
- [ ] Scan screen: under the danger zones a section "Described, not cleaned" with one OrbStack card (2 numbers, 4 parts,
      3 commands). No checkbox, nothing selectable, no Select all, and the zones, the ring and the selection bar do not
      count it. Review is unaffected (nothing of Docker's in it).
- [ ] Details opens the insight screen: the largest size, what the file takes on the disk, what is used inside; four parts
      (build cache Safe, images Low, stopped containers Medium, volumes High) each with its size, what running its
      command costs and its command with `--context orbstack`; no command for the containers part (nothing unused);
      the notes (how OrbStack gives space back, "Fresher never runs these"). Back returns to Scan.
- [ ] Copy puts a command on the clipboard and says "Copied."; paste it into a text editor and read the line (no
      `-f`, no `--force`). Check it in the built app too (`npm run tauri build`), not only the dev web view: the
      clipboard in the built app is unverified.
- [ ] `npm run dev:sandbox -- docker-stopped`: Docker Desktop installed and closed shows its disk file's two figures and
      no part; OrbStack open with its engine stopped shows no numbers. `-- docker-no-answer`: a card with no numbers and
      no command each. `-- docker-left-behind`: a "left behind" card with the disk file's numbers.
- [ ] Select everything on Scan and open Review: no Docker line; a cleanup leaves the socket and the disk file alone.

### 1.14 The scan screen and ignoring one item ([027/04/02](./plans/027-scan-screen/04-docs-and-finish/02-checks-and-walk.md), [028/02/02](./plans/028-ignore-item/02-docs-and-finish/02-checks-and-walk.md))

- [ ] `rm -rf sandbox/trial && npm run dev:sandbox -- trial` (and `app-leftovers` for the choices). Walk the boxes of 027/04/02 "By the user": the sidebar,
      the ring and its legend, the cards in danger zones, Details and Back, the selection bar, Review's zones, the
      sandbox's Trash in the banner and the result.
- [ ] On a category's Details, walk the boxes of 028/02/02 "By the user": the row's menu, the focus after an ignore,
      the keyboard path, Restore from Settings, Review without the item.
- [ ] Known flaky in the autotests, not a defect of the app: the chromium screenshot
      `category-details-app-leftovers-chosen` fails now and then in the full run and passes alone.

### 1.20 The largest files ([035/05/01](./plans/035-large-files/05-finish/01-docs-and-conclusions.md))

- [ ] `rm -rf sandbox/large-files && npm run dev:sandbox -- large-files`. Walk `EXPECTED.md` line by line, negatives
      first: each row it names withheld is listed with its sentence, each offered row can be ticked.
- [ ] Largest: the Files list, the filters (kind, place, changed), the Folders view and its way back by the breadcrumb,
      a withheld row's sentence, the Documents notice.
- [ ] Tick the disk image, the package, the archive and the folder. Review shows "Your own files" at High, nothing
      ticked by itself. Confirm to the Trash: the rows leave the list. Cleanups: put back. Rescan: they are listed again.

### 1.21 Demo recordings: the world and the check ([038/04/02](./plans/038-demo-recordings/04-finish/02-docs-and-conclusions.md))

Quit any development window first (the app is single-instance). Section 2.19 is the recording itself.

- [ ] `npm run dev:sandbox -- fixtures/demos/developer-mac.toml`: the world looks like a believable developer's Mac
      (the banner shows, as in any sandbox). The ring shows a 460 GB disk with 444 GB used; the cards are sized in
      gigabytes (Xcode build data, caches of the package managers, three apps' caches, leftovers of two removed apps,
      two models, old toolchains, Downloads, logs). Nothing names your home folder or your user name. Quit the window.
- [ ] `npm run demo:check -- all`: both scenarios, `cleanup` and `overview`, print `played in N s` with N at most 20 and
      the command ends 0. A failure names the step and the reason: write it down.
- [ ] Each run made a new folder `/private/tmp/fresher-demo/<scenario>-<n>`; nothing was removed. They stay until macOS
      clears `/private/tmp`; remove them by hand if you want the space back.

## 2. The real Mac: one recorded run

`npm run dev` records everything you do in one file; the agent checks the file against the disk with
`npm run dev-runs:check`. Do 2.0 once, then the by-hand items of 2.1 to 2.10: only what the disk cannot answer is left
here. The record is in [`testing.md`](./testing.md#the-development-run-record); it holds real paths, lives in the
git-ignored `NO_COMMIT/dev-runs/` and is never sent anywhere.

### 2.0 The run ([026/03/03](./plans/026-dev-run-record/03-checker-and-finish/03-run-and-conclusions.md))

On the real home use the Trash removal only: never choose "Delete permanently" and never empty the Trash during this
run.

1. [ ] Quit any development window. Settings, Permissions: Downloads on (the system may ask: allow it). Note whether
       the terminal has Full Disk Access (the record notes it too).
2. [ ] `npm run dev`. **First check:** the window shows the development banner. If it does not, **stop: quit the app
       without confirming anything** and tell the agent.
3. [ ] Scan `~` once; wait for the end. **Second check, before any other cleanup:** select one small offered item,
       Review, Confirm to the Trash: it must end "Development build: a dry run..." and still be where it was in Finder.
       If it ends anything else, **stop: quit the app without confirming anything more** and tell the agent.
4. [ ] On Scan, press Select all in each danger zone; Review: the dialog names the count, the size and the highest
       danger; Confirm to the Trash (not "Delete permanently"). Every item that passes its checks ends
       "Development build: a dry run..."; an item a check refuses keeps its own reason; nothing is freed. Finder: two of
       the offered folders are still where they were.
5. [ ] Cleanups: the dry runs are listed; Put back answers that nothing went to the Trash.
6. [ ] JetBrains (2.2), if an IDE is installed: open it, scan again, quit it, scan again (the record keeps all the
       scans).
7. [ ] Quit. Hand the agent the record: `ls -t NO_COMMIT/dev-runs/*_mac*.json | head -1`. The agent reads both the
       `.json` and its `.events.jsonl`: actions after the last cleanup are only in the events.
8. [ ] The agent runs `npm run dev-runs:check -- <that file>` and goes through `<file>.check.md` with you; each
       difference is written in the conclusions of the step it belongs to (a module's 04/02 or 05/02, or 026/03/03).

**What the record and the checker answer**, so you do not do it by hand: every offered path exists, is no link, is a
file or a folder, is inside the roots and no protected or whole folder, and its size matches the disk; stale projects:
the folder is an artifact or a Rust cache, no `src`, `.git`, `dist`, not in `~/Library`, an app or a hidden home folder,
the project's folders and branches older than the days, the cache older than 7 days; downloads: first level of
Downloads only; temporary files: in `T` or `/private/tmp`, unchanged for the days; the dry run moved nothing (per cleanup: every item `would-*`, `skipped`, `failed` or `not-reached`, none
`moved`, `emptied`, `deleted` or `tool-done`).

### 2.1 Storage map ([012/06/02](./plans/012-storage-map/06-finish/02-run-and-conclusions.md))

```sh
df -k ~ /
df -k /System/Volumes/VM /System/Volumes/Preboot /System/Volumes/Update 2>/dev/null
diskutil apfs list | head -60
```

- [ ] The bar's parts add up to the capacity of `df -k` for the home's volume.
- [ ] "macOS" equals the used space of `/`, Preboot and Update; "Swap and sleep image" the used space of
      `/System/Volumes/VM`; Free the available space.
- [ ] "Outside the scanned folders" lists its reasons, with the unreadable folders' count and the permission.
- [ ] The purgeable note is there, at least zero.
- [ ] The ring's legend: the cleanable part split into the four danger levels with the sizes of the cards in the zones
      below, Free as the gap, the system parts; the "i" of each row reads right, and hovering a segment and its row
      highlights both. (The by-meaning rows and the view by place are gone from the UI since plan 027.)
- [ ] The `Shares::of` time and heap on your home, restored from the app's cache. Reference: the generated LARGE tree
      gave 3.9 ms and 625 226 bytes; budget 50 ms and 5 MB. Where the number is shown (a debug log line) is in the
      step; record time and bytes.
- [ ] A root at a volume's mount point (an external drive, or a disk image mounted by hand with
      `hdiutil attach -nobrowse <image>`; detach after): its map says "Not measured", with no neighbours. A root of `/`
      is tried once and what the map says is written down.

### 2.2 JetBrains ([016/04/02](./plans/016-jetbrains/04-finish/02-run-and-conclusions.md))

The agent reads from the record that the version in use is left out ("This is the version you use.") and that an open
IDE's folders are left out with "The IDE is open..." in the second scan and offered again in the third.

```sh
ls ~/Library/{Application\ Support,Caches,Logs}/JetBrains
mdls -name kMDItemCFBundleIdentifier -name kMDItemVersion /Applications/<IDE>.app   # also ~/Applications for Toolbox
```

- [ ] Every offered folder is older than an installed IDE that is not open (the checker has no JetBrains rule).
- [ ] Each installed IDE's bundle id and version: the products table, the `-EAP` ids and the Toolbox app's id, the
      Toolbox folders, a folder's age as its own modification time, and the agent's calls 1, 2, 5, 6, 13, 14 and 19 are
      confirmed or corrected; `modules/jetbrains/products.toml`'s comment says which ids were seen.

### 2.3 Temporary files ([017/04/02](./plans/017-temp-files/04-finish/02-run-and-conclusions.md))

The checker does the place (in `T` or `/private/tmp`) and the age of every offered item. It does not read the owner
or the flags, it does not know `TemporaryItems`, and it does not compare the left-out counts with anything: those are
by hand below.

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
lsof +D "$T" 2>/dev/null | awk '{print $1}' | sort | uniq -c | sort -rn | head
```

- [ ] Every offered item is owned by you, with no flag (`stat -f '%Su %Sf'`); nothing in `C/`, `0/`, `X/` or
      `TemporaryItems` is claimed; the root-owned locked updater package is left out ("It belongs to the system or
      another user...").
- [ ] The left-out counts in the record match the commands' (young items, root's, sockets).
- [ ] The sweep's plists and log agree with the category's texts (the days, whether it runs at start).
- [ ] Agent's calls 12 and 13: make a scratch file in `$TMPDIR`, move it to the Trash **with Finder** and use Finder's
      Put Back (it lands in `~/.Trash`); the sweep's log answers whether it runs at start. Correct the category's
      texts (`docs/categories/temp-old-files.md`, `modules/temp/locales/en.json`) if the Mac shows otherwise.

### 2.4 Developer caches ([018/04/02](./plans/018-developer-caches/04-finish/02-run-and-conclusions.md))

The checker compares every offered item's size with the disk; it has no dev-caches rule, so what each item is and what
must never be claimed is by hand, with what only the tools can say.

```sh
ls ~/.npm/_cacache ~/Library/Caches/Yarn ~/Library/Caches/Homebrew ~/Library/Caches/pip 2>/dev/null
head -c 43 ~/.gradle/caches/CACHEDIR.TAG; echo; head -c 43 ~/.cargo/registry/CACHEDIR.TAG; echo
ls ~/Library/Caches/go-build 2>/dev/null | head; ls ~/Library/Caches/CocoaPods 2>/dev/null
ls ~/Library/Developer/Xcode/DerivedData 2>/dev/null; ls ~/Library/Developer/Xcode/DerivedData/*/info.plist 2>/dev/null
mdls -name kMDItemCFBundleIdentifier /Applications/Xcode.app 2>/dev/null
find ~/.npm/_cacache ~/.gradle/caches ~/.cargo/registry ! -user "$USER" 2>/dev/null | head
find ~/.npm/_cacache ~/.gradle/caches ~/.cargo/registry \( -type s -o -type p \) 2>/dev/null | head
```

- [ ] Every offered item is one of the table's places with its sign; nothing of Cargo's `index`, `git` or `bin`, npm's
      `_npx` or `_logs`, Gradle's `wrapper`, or Homebrew's `api` is claimed.
- [ ] The facts "to verify" (design, research, agent's calls 2 and 3): Go's `README` and `trim.txt`, CocoaPods' `Pods`,
      pip's `http-v2`, derived data's `<project>-<28 lowercase letters>` and `info.plist`, Xcode's bundle id
      `com.apple.dt.Xcode`: on this Mac where the tool is there, otherwise from the tool's own documentation. Correct
      `modules/dev-caches/tools.toml`, `locales/en.json` and `docs/categories/dev-caches-*.md` where they differ.
- [ ] One real cleanup with **Finder** (not the app): move `~/.npm/_cacache` to the Trash, run `npm install` in a
      project (it makes the cache again); the old folder stays in the Trash (do not empty it during the run). Skip if
      you do not want to lose the cache for a minute.

### 2.5 Uninstalled app leftovers ([019/04/02](./plans/019-uninstalled-app-leftovers/04-finish/02-run-and-conclusions.md))

The checker does not know the installed apps, so this one stays by hand. For each offered id `<id>` (the record lists
them):

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
      (its saved state or container is there); none starts with `com.apple.`; nothing under `Group Containers` is
      offered.
- [ ] Count the ids with leftovers that the proof leaves out (named after an id, not installed, no saved state and no
      container): the number and two or three examples (invented ones, no real names) in the conclusions.
- [ ] Shared frameworks' ids among the offered ones (an updater or a crash reporter that writes an id-named cache for
      many apps): each one goes into `modules/app-leftovers/exclude.toml` with its `why`.
- [ ] The facts "to verify": `mdutil -s` says `Indexing enabled.` for an indexed volume; `/` and the data volume are
      browsable as `mounted_volumes` reads them; the places of `modules/app-leftovers/places.toml` are readable without
      Full Disk Access (one that is not is dropped or made optional, with its reason).
- [ ] **Mounts an image, read-only on the Mac otherwise.** A disk image Spotlight does not index
      (`hdiutil attach -nobrowse <image>`; `sudo mdutil -i off <mount>` for a test image): `mdutil -s` says what for the
      unindexed one; a rescan offers nothing from `app-leftovers`, and a removed JetBrains IDE's data (if any) is left
      out with the Spotlight reason. Detach after.
- [ ] A Time Machine or other backup disk attached (if you have one): what `mdutil -s` says of it, and that
      `app-leftovers` then keeps everything (known issue
      [an unindexed volume](./known-issues/app-unindexed-volume-turns-off-removed-app-detection.md)).
- [ ] The module's `finish` time on the same scan (from the record), and the number of ids asked (up to three `mdfind`
      per id that reaches the Spotlight row, plus one `mdutil -s` per volume): record both.
- [ ] One real cleanup with **Finder**: install a small test app, open it (a window, so its saved state exists), quit,
      drag it to the Trash, move its leftovers to the Trash by hand, put it all back.

### 2.6 Downloads and installers ([020/05/02](./plans/020-downloads/05-finish/02-run-and-conclusions.md))

The checker does the place (first level of Downloads) and the sizes. By hand, what the disk does not say:

```sh
ls /var/db/receipts/*.plist | while read -r r; do plutil -extract PackageFileName raw "$r"; done
sw_vers -productVersion
mdls -name kMDItemLastUsedDate -name kMDItemDateAdded ~/Downloads/<a file you opened lately>
xattr -l ~/Downloads/<the same file>
```

- [ ] The first scan after switching Downloads on offers items: no second scan is needed (01/04).
- [ ] An offered installer's app is installed at the installer's version or newer (the app's About window) and was
      installed after the installer was opened; a macOS installer is older than `sw_vers`; a big file is one you have
      not opened for 30 days; an unfinished download is old and its browser closed.
- [ ] Count, without names, how many `.dmg`/`.pkg` files the name rule and the receipts prove and how many they leave
      out; two or three invented examples of what was missed, in the conclusions.
- [ ] Facts "to verify": opening a file from Finder writes `com.apple.lastuseddate#PS` and moves its status change
      (`stat -f %Sc` before and after a double-click; was seen on five files on 2026-11-19; also a file opened from an
      app's own open panel: say what happens); opening a disk image from Finder moves its status change (agent's call
      2); an app updated by its own updater has a new bundle status change; the receipts' `PackageFileName` and
      modification time (install a small test package and see whether a downloaded product archive's name ever equals a
      receipt's); the browsers' bundle ids of `unfinished.toml` (`mdls -name kMDItemCFBundleIdentifier` on each
      installed browser) and which browser writes `.partial` (if none, the row goes), Chrome's, Firefox's and Opera's
      suffixes by pausing a download and quitting the browser; Safari's `.download` bundle holds `Info.plist` (pause a
      download, quit Safari); the macOS installers' names and versions of `modules/downloads/macos.toml` against Apple's
      list. Correct the tables, texts and `docs/categories/downloads-*.md` where they differ.
- [ ] One real cleanup with **Finder**: an old installer of an installed app moved to the Trash and put back.

### 2.7 Stale projects ([021/04/02](./plans/021-stale-projects/04-finish/02-run-and-conclusions.md))

The checker does the folder kind, the paths kept out, the days of the project's folders and branches, and the sizes.
By hand, what a Mac can show only after an action:

```sh
du -sh target/debug/{deps,incremental,build} target/llvm-cov-target 2>/dev/null   # in this repository
stat -f %Sm <folder of a file you save>                                           # before and after a one-line save
```

- [ ] Every project kept `recent` that you think is old: which folder or branch is new; kept `unread`: which folder
      was not read.
- [ ] **This repository's `target/`**: what the Rust cache category offers and keeps, with sizes (from the record); the
      `du` line today and again after a week of work: which part grows (decision 3.4).
- [ ] Facts to verify: an editor that saves in place moves no folder time (`stat -f %Sm` of the file's folder before
      and after a one-line save, in each editor you use); which git actions move `refs/heads` (commit, pull, merge,
      checkout; `stat -f "%Sm %N" <project>/.git/refs/heads`); does Finder browsing (a `.DS_Store`) move a project's
      time; a `node_modules` of hard links into a shared store: the finding's size against `du`; a recompiling build
      moves `target/debug/incremental`'s times.

### 2.8 Local AI models ([023/03/02](./plans/023-local-ai-models/03-finish/02-run-and-conclusions.md))

The checker does the existence, the folder kind, the roots and the sizes. By hand, what the disk cannot answer:

```sh
ls ~/.cache/huggingface/hub ~/.lmstudio/models/* ~/.cache/lm-studio/models/* 2>/dev/null
du -sh ~/.cache/huggingface/hub/models--* ~/.cache/huggingface/hub/datasets--* 2>/dev/null
du -sh ~/.lmstudio/models/*/* ~/.cache/lm-studio/models/*/* 2>/dev/null
osascript -e 'id of app "LM Studio"'
```

- [ ] Every offered model: a repository folder with `blobs` and `snapshots`, or a folder two levels down in LM Studio's
      models folder; its size against `du`; its title against the folder's name.
- [ ] Nothing offered outside the three places; a Space or a non-model folder of the hub is not offered.
- [ ] Facts to verify: LM Studio's bundle id (the `osascript` line) against `stores.toml`, and with LM Studio open its
      models are kept; its older place `~/.cache/lm-studio/models` on a Mac that has it (or note that none was
      found); whether LM Studio records when a model was last used, for the date in the reason; a Hugging Face
      download in progress (what the scan offers, what the cleanup's rehearsal says); an LM Studio model imported by
      hand reads as a download, at Medium, with the "added yourself" sentence. Correct `stores.toml`, the texts and
      `docs/categories/local-ai-models-*.md` where they differ.

### 2.11 Ollama's models ([029/04/02](./plans/029-ollama-models/04-finish/02-run-and-conclusions.md))

`npm run dev`, a scan of `~` (a dry run on the real Mac). Compare with these read-only commands:

```sh
ollama list
find ~/.ollama/models/manifests -type f | sed 's|.*/manifests/||'
find ~/.ollama/models/manifests -type f -exec wc -c {} + | sort -n | tail -3
ls ~/.ollama/models/blobs | grep -c partial
du -sh ~/.ollama/models/blobs
```

- [ ] Every offered model is in `ollama list` with the same tags; aliases (the same id) are one finding; each freed
      size checked against the layers its manifest names that no other manifest names.
- [ ] Select an Ollama model, confirm: the dry run checks every part and ends "a dry run"; with a model loaded
      (`ollama run <model>` in another window), the item is skipped as in use.
- [ ] Facts to verify, corrected in the code and the texts where they differ: the largest manifest against the 4 KiB
      limit; the default host and namespace in `ollama list`'s names; partial download names while `ollama pull` runs
      and that Ollama's server deletes layers no manifest names at its start (both in a throwaway `OLLAMA_MODELS`
      folder, never the real store); what a model loaded while its files move does (the same folder).

### 2.12 Caches of installed apps ([030/03/02](./plans/030-installed-app-caches/03-finish/02-run-and-conclusions.md))

`npm run dev`, a scan of `~` (a dry run on the real Mac). Compare with these read-only commands:

```sh
du -sh ~/Library/Caches/*.* 2>/dev/null | sort -h | tail -30
ls ~/Library/Caches | grep -c '\.'
mdls -name kMDItemCFBundleIdentifier -raw /Applications/*.app 2>/dev/null | tr '\0' '\n' | sort | head -50
```

- [ ] Every offered cache is named after an installed app's id, a helper's, or an id under the app's; its size matches
      `du`; its title names the right app.
- [ ] Nothing of Apple's, nothing of Fresher's own, and no removed app's cache is offered by this module; a removed
      app's cache still goes to `app-leftovers` when it is proven and old.
- [ ] An open app's cache is left out; quit it and rescan: it is offered.
- [ ] Facts to verify, corrected in the code, the texts and `docs/categories/app-caches-library-caches.md` where they
      differ: the largest ten caches offered, looked at read-only for what does not come back (an app that keeps such
      data gets an `exclude.toml` row with its why); how many small findings a real Mac shows (under 1 MB), and
      whether a size floor is needed; an app with an always-running helper (a menu bar agent): whether its cache is
      never offered; an updater's cache holding a pending update.

### 2.13 Logs and crash reports ([033/03/01](./plans/033-logs/03-finish/01-docs-and-conclusions.md))

`npm run dev`, a scan of `~` (a dry run on the real Mac). Compare with:

```sh
find ~/Library/Logs -type f -mtime +7 | wc -l
find ~/Library/Logs -type f -mtime +7 -exec ls -lS {} + | head -10
```

- [ ] Every offered file is under `~/Library/Logs` and older than 7 days; the count is near the `find` count (fewer by
      links, state and lock files, `JetBrains`, the bundle-id-shaped folders' files and files changed (ctime) within
      the week).
- [ ] Nothing under `Logs/JetBrains` or a `com.*`-shaped folder is offered by `logs`.
- [ ] The largest ten findings, looked at read-only: none is state an app reads back.
- [ ] How many findings the Mac shows and whether Details stays usable; the scan time beside plan 030's.

### 2.14 Browser and web-engine caches ([034/03/01](./plans/034-browser-caches/03-finish/01-docs-and-conclusions.md))

`npm run dev`, a scan of `~` (a dry run on the real Mac), with Chrome, Chromium and Firefox closed. Compare with this
read-only command:

```sh
du -sh ~/Library/Caches/Google/Chrome/*/{Cache,"Code Cache"} ~/Library/Caches/Chromium/*/{Cache,"Code Cache"} \
  ~/Library/Caches/Firefox/Profiles/*/{cache2,startupCache} 2>/dev/null
```

- [ ] Every offered browser cache is checked against `du`; its title names the right browser, profile and folder.
- [ ] With Chrome open, its caches are left out; quit it and rescan: they are offered.
- [ ] Nothing under `Application Support/Google`, `Application Support/Firefox` or `Application Support/Chromium` is
      offered; `~/Library/Caches/Google/Chrome-headless` is not offered.
- [ ] Facts not verified here: Redis Insight's caches with the app open and closed; whether a browser accepts its
      emptied `Cache` folder on the next start without a warning (a release build, once, Chrome and Firefox);
      Firefox's `startupCache` emptied while the profile is closed.

### 2.15 The Trash line ([037/04/01](./plans/037-trash/04-finish/01-docs-and-conclusions.md))

`npm run dev`, a scan of `~` (a dry run on the real Mac).

- [ ] With Full Disk Access: the line's first number against `du -sh ~/.Trash`; without it: the line shows only a
      share, or nothing.
- [ ] Finder, Settings, Advanced, "Remove items from the Trash after 30 days" on: `defaults read com.apple.finder
      FXRemoveOldTrashItems` prints `1`; the line shows a date only when a share is there.
- [ ] With a USB disk holding something in its Trash and scanned: a line of its own, named after the disk.
- [ ] Open Trash opens Finder's Trash window (`open ~/.Trash` does the same, with and without Full Disk Access).

### 2.16 Xcode and simulators ([031/03/02](./plans/031-xcode/03-finish/02-run-and-conclusions.md))

`npm run dev`, a scan of `~` (a dry run on the real Mac). Compare with these read-only commands (never run `simctl`):

```sh
du -sh ~/Library/Developer/Xcode/* ~/Library/Developer/CoreSimulator/* 2>/dev/null
ls ~/Library/Developer/Xcode/*DeviceSupport 2>/dev/null
ls ~/Library/Developer/Xcode/Archives/* 2>/dev/null
ls ~/Library/Developer/Xcode/UserData/Previews 2>/dev/null
```

- [ ] Every offered item checked by hand: its folder, its size against `du`, its title.
- [ ] Nothing of `UserData` but `Previews`, nothing of `DerivedData` (that is `dev-caches`), no simulator device.
- [ ] With Xcode open: everything left out; quit it and rescan: offered.
- [ ] The facts to verify, each corrected in `places.toml`, the names, the texts and the category docs where they
      differ: the device support folders' names (with or without a model prefix) and their `Symbols` sign, on a Mac
      that has connected a device; the previews' folder and its `Simulator Devices` sign on a recent Xcode; an
      archive's `Info.plist` and the dated folder's name; whether `CoreSimulator/Caches` holds anything but `dyld`.

### 2.17 Docker, described ([032/06/01](./plans/032-docker/06-finish/01-docs-and-conclusions.md))

`npm run dev`, a scan of `~` on a Mac with Docker Desktop or OrbStack **running** (a development build is a dry run;
the module asks only a runtime that is open with its socket present). Compare the card with the runtime's own numbers,
run by you, by hand:

```sh
docker --context orbstack system df      # or --context desktop-linux
ls -l ~/Library/"Group Containers"/HUAQ24HBR6.dev.orbstack/data/data.img
du -sh ~/Library/"Group Containers"/HUAQ24HBR6.dev.orbstack/data/data.img
```

- [ ] Each part's unused size matches `Reclaimable` of its line (images, containers, volumes, build cache); the disk
      file's figures match `ls` (its length) and `du` (what it takes).
- [ ] With the runtime closed: its card shows the disk file's figures only, and no `docker` program was started by the
      scan (Activity Monitor).
- [ ] With the runtime open but its engine stopped: no numbers, no error.
- [ ] A restored scan (quit and start the app): the card shows nothing until a scan (known gap) and no runtime is
      asked.
- [ ] The facts to verify, corrected in `runtimes.toml`, the texts and the category docs where they differ: see the
      plan's list in [`deferred/user-review-2026-11-22.md`](./deferred/user-review-2026-11-22.md), "Plan 032".

### 2.18 The largest files ([035/05/01](./plans/035-large-files/05-finish/01-docs-and-conclusions.md))

Open Largest after a scan of `~` (`npm run dev`: a development build is a dry run). Check the ten largest rows against
the shell, run by you:

```sh
find ~ -xdev -size +100M -not -path "$HOME/Library/*" -not -path "*/.*" 2>/dev/null | head -50
du -sh <a path from the list>
```

- [ ] The ten largest rows are the biggest things the `find` shows (sizes on disk, `du -sh`; allocated size ranks).
- [ ] No row is under `~/Library`, the Trash, a hidden folder or an app; a module's folder is not listed.

### 2.19 Demo recordings: the recording on the screen ([038/04/02](./plans/038-demo-recordings/04-finish/02-docs-and-conclusions.md))

Run after 1.21. This is the one run that captures your screen; no private data is on it (the world is made up), but it
needs a permission and a plain desktop.

- [ ] Give the terminal the Screen Recording permission (System Settings, Privacy & Security, Screen & System Audio
      Recording) and set a plain wallpaper (the sidebar's blur shows it). Quit any development window.
- [ ] `npm run demo:record -- overview`: the window opens at the top left, plays, closes; the command prints the GIF's
      and the video's paths. If the capture fails (`avfoundation` on macOS 27 is not verified), write the message
      down: the fix is to switch `captureCommand` to `screencapture -v`.
- [ ] Open `assets/demo/overview.gif` and `NO_COMMIT/demo/overview.mp4`: only the window, no cursor of the Mac, the
      drawn pointer, no banner, no real path or name, sizes in gigabytes, the world's disk in the ring, the GIF under
      5 MiB, the corners and the title bar cropped cleanly.
- [ ] If the GIF is good, commit it (plan 040 places it in the README); if it is over budget or ugly, write down why.
- [ ] Optional: `npm run demo:record -- cleanup` and look at the cleanup's result and its freed-space line.

### 2.9 Scan time on the real home (rule 4: within 5 %)

Scan time is compared once per module generation, not per plan; do it with the baseline worktrees of section 4.2, since
`main` already holds every module. Quit all other apps, scan `~` three times in each build (the first warms the cache;
use the later two and the median), same build kind, same state of the Mac.

- [ ] Plans 016/017 ([016/04/02](./plans/016-jetbrains/04-finish/02-run-and-conclusions.md),
      [017/04/02](./plans/017-temp-files/04-finish/02-run-and-conclusions.md)): `main` against baseline `f76e1c1`
      (plan 015, no real module). Record each module's share where the app shows it.
- [ ] Plan 018 ([018/04/02](./plans/018-developer-caches/04-finish/02-run-and-conclusions.md)): `main` against `645e04b`
      (the commit before plan 018). dev-caches is registered in every build, so this exercises the held-names path.
- [ ] Plan 019 ([019/04/02](./plans/019-uninstalled-app-leftovers/04-finish/02-run-and-conclusions.md)): `main` against
      `4ad2282`.
- [ ] Plan 020 ([020/05/02](./plans/020-downloads/05-finish/02-run-and-conclusions.md)): `main` against `70447c5`, with
      Downloads allowed in both.
- [ ] Plan 021 ([021/04/02](./plans/021-stale-projects/04-finish/02-run-and-conclusions.md)): `main` against `c2bdd44` (the commit before plan 021).

### 2.10 Unused toolchain versions ([024/04/02](./plans/024-unused-toolchains/04-finish/02-run-and-conclusions.md))

`npm run dev`, a scan of `~` with Full Disk Access (a dry run on the real Mac); turn the Node switch on and scan again.
Compare with these read-only commands:

```sh
rustup toolchain list; cat ~/.rustup/settings.toml
ls ~/.rustup/toolchains ~/.nvm/versions/node 2>/dev/null; ls -R ~/.nvm/alias 2>/dev/null
du -sh ~/.rustup/toolchains/* ~/.nvm/versions/node/* 2>/dev/null
find ~ -xdev \( -name rust-toolchain.toml -o -name rust-toolchain -o -name .nvmrc -o -name .node-version \
  -o -name .tool-versions \) -not -path '*/node_modules/*' -not -path '*/Library/*' 2>/dev/null
find ~ -xdev \( -name .nvmrc -o -name .node-version \) -type l 2>/dev/null
```

- [ ] Every offered version checked by hand: not the default, not an override or alias, not named by any pin the
      `find` lists; its size against `du`.
- [ ] Nothing offered outside the two places; no moving channel, no newest Node version.
- [ ] Facts to verify: whether the walk is complete on this Mac (if every version is left out `partial-scan`, write the
      numbers and open a deferred item); a removed toolchain comes back with `rustup toolchain install <name>` while
      `update-hashes/<name>` remains (in a throwaway `RUSTUP_HOME`, never the real one); linked pins (the second
      `find`): how many, and whether they block nvm's offers. Correct the table, the texts and
      `docs/categories/unused-toolchains-*.md` where they differ.

## 3. Permissions and probes (plan 006)

Plan 006 is built on documentation answers; this section turns them into measured ones
([006 readme, "By hand"](./plans/006-permissions/readme.md#by-hand),
[probes 01/01](./plans/006-permissions/01-probe-the-system/01-prompts-and-checks.md) and
[01/02](./plans/006-permissions/01-probe-the-system/02-panes-and-reasons.md)). The result of a probe changes constants
in `crates/macos/src/privacy/` (`PLACES`, `FDA_CANDIDATES`, `ADDRESSES`, the panes) and "Probe results" in
[`plans/006-permissions/00-design.md`](./plans/006-permissions/00-design.md) in the same commit; "Reopen Fresher" goes
if a grant applies at once. Write the macOS version beside each answer.

macOS ties a grant to the app's code signature: an ad-hoc rebuild loses grants, and under `npm run dev` access is
attributed to the terminal app. For the acceptance part use a built app and reset it between runs, on a development
Mac only: `tccutil reset All io.github.alexsergey.fresher` (built app) or `...fresher.dev` (`npm run dev`).

### 3.1 Probes (plan 01)

The probes are throwaway and never committed. A binary built in `NO_COMMIT/006-probes/` (its own `Cargo.toml`, outside
the workspace) opens each place read-only, lists nothing, closes it, and prints the `errno`; it is ad-hoc signed with a
fixed bundle id and started from Finder (not a terminal), so macOS attributes access to the probe itself. No such
binary exists yet: writing it is part of the run (about 40 lines of `std::fs::File::open` and `raw_os_error()`), or use
the app's own access check in Settings, Permissions for the Files and Folders and Full Disk Access rows.
`NO_COMMIT/probes/` holds only plan 004's probes.

- [ ] Which places prompt and which only refuse, with Full Disk Access off, then on: `Desktop`, `Documents`,
      `Downloads`, `~/Library/Containers/<another app>`, `~/Library/Group Containers/<another group>`, iCloud Drive
      (`~/Library/Mobile Documents`), the Photos library, `~/Library/Mail`, `~/Library/Safari`, `~/Library/Messages`, the
      Trash; whether listing `~/Library/Containers` itself prompts or only entering one container; a removable and a
      network volume as a scan root; after denying a prompt, whether the next access is a silent `EPERM`; whether
      `EPERM` (privacy) is always told apart from `EACCES` on a folder; whether any place needs an FFI call beyond
      `open`. Write each as prompts / refuses with `EPERM` / opens.
- [ ] The Full Disk Access check: the first candidate that does not answer "missing" is the right one and never
      prompts. Candidates today: `~/Library/Application Support/com.apple.TCC/TCC.db`, then `~/Library/Safari`
      (`crates/macos/src/privacy/check.rs`).
- [ ] A Files and Folders grant applies at once to the running app; a Full Disk Access grant applies at once or only
      after a relaunch. Record which.
- [ ] The three pane addresses open the right pane, with the network off (Wi-Fi off), on this macOS and, if you have a
      macOS 13 machine or VM, there too. This one runs `open` (opens System Settings, changes nothing):

```sh
open "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles"
open "x-apple.systempreferences:com.apple.preference.security?Privacy_FilesAndFolders"
open "x-apple.systempreferences:com.apple.preference.security?Privacy"
```

- [ ] The merged `Info.plist` of a bundle holds exactly the five reason strings (`NSDesktopFolderUsageDescription`,
      `NSDocumentsFolderUsageDescription`, `NSDownloadsFolderUsageDescription`, `NSRemovableVolumesUsageDescription`,
      `NSNetworkVolumesUsageDescription`) and no other usage key: after `npm run build`,
      `plutil -p target/release/bundle/macos/Fresher.app/Contents/Info.plist | grep -i usage`. The five appear in the
      prompts; data from other apps (macOS 14+) has a fixed text of its own, no key.
- [ ] Under `npm run dev`, access is attributed to the terminal app (grant Full Disk Access to it to see a real grant);
      an ad-hoc rebuild loses a grant.
- [ ] Where a row disagrees: change the constants and update "Probe results" in the same commit (ask the agent).

### 3.2 Acceptance on the built app (plan 006 "By hand")

`npm run build` first (output `target/release/bundle/macos/Fresher.app`), then run that app. It is a release build: it
can remove things, so only look and scan here, and do not confirm any cleanup on real data.

- [ ] A first scan shows no system prompt.
- [ ] The Full Disk Access notice opens the right pane of System Settings.
- [ ] Granting Full Disk Access while the app is open updates the notices (or offers "Reopen Fresher", per 3.1).
- [ ] Opting into Desktop in Settings prompts once, inside the next scan, with the reason string; allowing it shows
      Desktop findings; denying it shows a notice. (Downloads works the same way: see 2.6.)
- [ ] A category that needs a missing permission is limited (a tooltip says what it misses); a category whose tool is
      not installed is unavailable with a plain reason.
- [ ] An item the app may not remove (for example one owned by root) is shown with its badge and no checkbox, and "Show
      in Finder" reveals it.
- [ ] The Permissions group of Settings lists every state truthfully and "Check again" updates it.
- [ ] A volume opt-in: a removable drive, then a network volume, added as a scan root prompts once inside the next
      scan with its reason string; allowing shows its findings; denying shows a notice whose "Open System Settings"
      opens Files and Folders.
- [ ] On a Mac before macOS 14 (only if you have one): other apps' data is "Not needed on this Mac" in the Permissions
      group, with no notice.

## 4. Benches (a quiet Mac)

Run when nothing else loads the machine (section 0). Earlier runs of 018, 019 and 020/01/02 were taken on a busy Mac
(load average 19 to 26, the same build spread by about 8 %, 24 to 48 threads by 10 %) and are inconclusive; this one
run settles them. It is read-only on your data (the generated tree lives under `target/bench-trees/`).

### 4.1 After: `main`

```sh
uptime                                              # load average under 2, on power
FRESHER_BENCH_TREE=large npm run bench 2>&1 | tee NO_COMMIT/bench-main.txt
```

### 4.2 Before: baselines

A git worktree per baseline in `NO_COMMIT/` (ignored by git; worktrees build into the main `target`, so each switch
rebuilds the bench). Run the same command in each, back to back with 4.1, and remove them after.

```sh
git worktree add NO_COMMIT/bench-base f76e1c1       # plan 015, before any real module
(cd NO_COMMIT/bench-base && FRESHER_BENCH_TREE=large npm run bench 2>&1 | tee $OLDPWD/NO_COMMIT/bench-f76e1c1.txt)
git worktree remove --force NO_COMMIT/bench-base
# repeat with 645e04b (before 018), 4ad2282 (before 019, after 018), 70447c5 (before 020, after 019),
# c2bdd44 (before 021, after 020)
```

- [ ] **Plan 018** ([018/04/02](./plans/018-developer-caches/04-finish/02-run-and-conclusions.md)): `main` against
      `645e04b`. Only the empty-holds path is measured (the bench has no interest with held names); say so. Known
      costs: `Hit` 16 to 24 bytes, +4.6 % tree heap in the state bench.
- [ ] **Plan 019** ([019/04/02](./plans/019-uninstalled-app-leftovers/04-finish/02-run-and-conclusions.md)): `main`
      against `4ad2282`, including the shaped interest 01/01 added. Earlier: -0.9 % at 12 threads, -0.3 % at 1 thread on
      a busy Mac. Retake with a shaped interest over many names. Known costs: one shape check per name under a shaped
      node; the whole claims of id-named folders (fewer nodes in the tree).
- [ ] **Plan 020, step 01/02** ([020/01/02](./plans/020-downloads/01-core-prerequisites/02-suffixed-names.md); the
      numbers go to
      [020/05/02](./plans/020-downloads/05-finish/02-run-and-conclusions.md)): `main` against `70447c5`, the suffixed
      interest (`Data/*` with `.download` and `.app`). Earlier: `matcher/entry` +0.6 %, `scan/full` 1 thread -0.8 %,
      12 threads -1.0 %; 24 to 48 threads varied by 10 %.
- [ ] **Plan 021** ([021/04/02](./plans/021-stale-projects/04-finish/02-run-and-conclusions.md)): `main` against `c2bdd44`; the new `subtrees` group's time (earlier: `subtrees/projects/small`
      11.6 µs on a busy Mac) and `scan/full` unchanged within 5 %.
- [ ] For every bench group, compare medians (criterion prints `change:`): within rule 4's 5 %? Write the table of
      groups (`matcher/entry`, `scan/full` at 1 and 12 threads, `scan/restore`, the state bench's tree heap) with the
      load average beside it. A bar that fails is not raised quietly: stop and tell the agent.

## 5. Release-blocking deferred runs (due before the first release)

Plan 004 and plan 008 were merged without their runs by hand
([`deferred/`](./deferred/README.md)); both are due before the first release. The other items due then (licence notices,
release versioning, own-data clearing, session log, bug-report file, diagnostics corner) are work to build, not runs.

### 5.1 The fast scan on your home ([deferred](./deferred/fast-scan-home-run.md), [004/08/03](./plans/004-fast-scan/08-queries-and-finish/03-budget-run.md))

A release build scanning your real home, read-only. Record counts, times and sizes, never a path. Fill the blanks in
plan 004's conclusions.

- [ ] `npm run build`; start `target/release/bundle/macos/Fresher.app`. The first scan of `~`: time, peak memory
      (Activity Monitor), the sizes of the two cache files and the data folder's footprint (a debug log line; the file
      sizes with `ls -l ~/Library/Application\ Support/io.github.alexsergey.fresher`), tree heap per node if logged.
      Target: 10 s on a warm tree.
- [ ] Optionally the same scan at two threads per core: time and peak memory.
- [ ] A rescan after a minute, and again after a day of ordinary use: time, changed folders, fallback if any.
- [ ] Quit and start again: the last result shows at once, with its date.
- [ ] Clones: the scan's total allocated bytes against `du -sk` of the same roots; within the bar, no shared block
      counted twice or lost.
- [ ] Once after a reboot, before anything else reads the disk: a cold first scan; record it, no target.
- [ ] Confirm or overrule the decisions taken as recommended on 2026-11-16: the 10 s target on a warm tree, one thread
      per core, the tree-heap bar kept on its test; say whether a cold-scan target is wanted.
- [ ] Optional, the generated big tree: `cargo nextest run -p fresher-e2e --release --run-ignored only --no-capture
      ten_scans_of_the_large` (the footprint after ten scans stays within its budget).

### 5.2 The first real cleanups, in a sandbox ([deferred](./deferred/removal-sandbox-run.md), [008/08/02](./plans/008-safe-removal/08-finish/02-sandbox-run.md))

Step 2 of the deferred item (the scan) is held by `npm run sandbox:test`. The numbered steps below keep the deferred
file's order; the real Trash, the real process table, an image and the real window are what only a hand can check.

- [ ] **Prepare**: `rm -rf sandbox/trial && npm run dev:sandbox -- trial --real-trash` for the trial folders. By hand,
      add what the generator does not make: a `held/` folder with its own marker and a trial folder held open from a
      second terminal (`cd` into it, or `tail -f` a file in it); a read-only APFS image with a trial folder in a
      `sandbox/` folder beside a marker (FAT32 and exFAT images no longer mount); a `~/FresherNoMarker` with a marker
      file that lacks the signature, a trial folder, and a bare trial folder with no marker. Details of each: the
      deferred file, step 1.
- [ ] **Move to the Trash**: the confirmation lists what will not be removed and why (the held folder in use, naming its
      holder; the image's folder; the unmarked folder as "Development build"); progress; the result "Moved to the
      Trash", the measured space about 0, the Trash sentence; the items are in the Trash.
- [ ] **Put back**: Cleanups shows "In the Trash: n of n"; one item put back from the app, another with Finder's own Put
      Back; the history then shows it no longer in the Trash.
- [ ] **Never touched**: the held folder, the image's folder (skipped, no Trash), the unmarked folder, the look-alike
      and the bare folder are all still in place.
- [ ] **Permanent delete** of the trial folders left: "Freed", the measured space next to the estimate; nothing in the
      Trash.
- [ ] **Quit during a long cleanup** (a few thousand small trial folders): Cmd-Q asks, **Keep cleaning** changes
      nothing, **Stop and quit** stops after the current item and the history shows the cleanup as stopped; closing
      the window asks the same; the Dock's Quit does not ask (known issue
      [`app-quit-during-cleanup`](./known-issues/app-quit-during-cleanup.md)) and the next start shows the cleanup as
      interrupted.
- [ ] **The release build in sandbox mode**: `npm run build`, then
      `FRESHER_SANDBOX=$PWD/sandbox/trial target/release/bundle/macos/Fresher.app/Contents/MacOS/<executable>`
      (the executable's name is the one `ls target/release/bundle/macos/Fresher.app/Contents/MacOS/` shows): no banner, the variable is ignored, it
      scans the real home and offers nothing from `demo` or `trial` (neither is registered); confirm nothing in it.
      This also settles item 5 of [007/05/02](./plans/007-safe-development/05-finish/02-run-and-conclusions.md).
- [ ] **Afterwards**: the sandbox folders removed, the image detached, the sandbox's items emptied from the Trash by
      hand, the scan roots set back; nothing else changed. Anything unexpected becomes a fix in its plan or a known
      issue. Also wanted when at hand: a run with Full Disk Access, the sound of the Trash call, a network share, a macOS
      before 27.

## 6. If something looks wrong

First check [`known-issues/`](./known-issues/README.md): these look like bugs and are known. The ones you may meet
here:

- A sandbox scan "made days ago" (the sandbox's clock starts at its generation):
  [`app-sandbox-scan-dated-at-generation`](./known-issues/app-sandbox-scan-dated-at-generation.md). Regenerate with
  `rm -rf sandbox/<name>`.
- A category whose items are all left out shows "0 bytes / 0 items" (Xcode while Xcode is open):
  [`ui-category-all-left-out-shows-zero`](./known-issues/ui-category-all-left-out-shows-zero.md).
- The scan offers an item inside the Trash:
  [`app-scan-offers-an-item-in-the-trash`](./known-issues/app-scan-offers-an-item-in-the-trash.md).
- A developer cache with a locked sub-folder is not offered, with no reason:
  [`scanner-unreadable-folder-in-a-claimed-match`](./known-issues/scanner-unreadable-folder-in-a-claimed-match.md).
- A removed-app detection that keeps everything while a USB stick or an installer image is mounted:
  [`app-unindexed-volume-turns-off-removed-app-detection`](./known-issues/app-unindexed-volume-turns-off-removed-app-detection.md).
- "Reopen Fresher" forgets itself after the card is rebuilt:
  [`ui-reopen-button-state-is-local`](./known-issues/ui-reopen-button-state-is-local.md); relaunch right after a
  confirmed cleanup cuts it short:
  [`app-relaunch-after-a-confirmed-cleanup`](./known-issues/app-relaunch-after-a-confirmed-cleanup.md).
- A setting changed during a scan shows no "Rescan to apply":
  [`core-settings-changed-during-a-scan`](./known-issues/core-settings-changed-during-a-scan.md).
- Flaky autotests: the WebKit gallery part 3 times out under load
  ([`ui-axe-gallery-part-3-webkit-timeout`](./known-issues/ui-axe-gallery-part-3-webkit-timeout.md); rerun it alone).
- The `all` sandbox's checklist is wrong where worlds interact
  ([`repo-sandbox-all-worlds-interact`](./known-issues/repo-sandbox-all-worlds-interact.md)); use one case file at a time.
- `npm run dev` and `npm run dev:sandbox` share one window: a launch while one is open shows the old window's mode.

If it is not known, capture and send:

1. A screenshot of the window, with the banner visible (it says the mode and the sandbox).
2. The module and the category, the case line of `EXPECTED.md` (its text), and what you expected against what you saw.
3. The scan's counts and times, and macOS (`sw_vers`) and the Full Disk Access state of the terminal app.
4. The terminal output of `npm run dev` (the log) and, for a sandbox, the `sandbox/<name>/world.toml` and `EXPECTED.md`
   as they were. For a failed autotest, the nextest output of the failing test. There is no bug-report file yet
   ([deferred](./deferred/bug-report-file.md)).
5. Never include real paths or file names from your Mac: counts, sizes and invented examples.

## 7. Plan step, status, section

Status as of the plan files on 2026-11-19: all are waiting for the run (no item above is ticked on the plan side).

| Plan step                                                                                                    | Status                         | Section       |
| ------------------------------------------------------------------------------------------------------------ | ------------------------------ | ------------- |
| [006 readme "By hand"](./plans/006-permissions/readme.md#by-hand)                                            | pending                        | 3.2           |
| [006/01/01 prompts and checks](./plans/006-permissions/01-probe-the-system/01-prompts-and-checks.md)         | pending (approved, not run)    | 3.1           |
| [006/01/02 panes and reasons](./plans/006-permissions/01-probe-the-system/02-panes-and-reasons.md)           | pending (approved, not run)    | 3.1           |
| [007/05/02 run and conclusions](./plans/007-safe-development/05-finish/02-run-and-conclusions.md)            | pending, in progress           | 1.1, 5.2      |
| [011/03/01 docs and conclusions](./plans/011-diagnostics/03-finish/01-docs-and-conclusions.md)               | pending, in progress           | 1.1           |
| [012/06/02 run and conclusions](./plans/012-storage-map/06-finish/02-run-and-conclusions.md)                 | pending                        | 1.2, 2.1      |
| [015/03/02 run and conclusions](./plans/015-module-groups/03-finish/02-run-and-conclusions.md)               | pending, in progress           | 1.1           |
| [016/04/02 runs and conclusions](./plans/016-jetbrains/04-finish/02-run-and-conclusions.md)                  | pending, in progress           | 1.3, 2.2, 2.9 |
| [017/04/02 runs and conclusions](./plans/017-temp-files/04-finish/02-run-and-conclusions.md)                 | pending, in progress           | 1.4, 2.3, 2.9 |
| [018/04/02 runs and conclusions](./plans/018-developer-caches/04-finish/02-run-and-conclusions.md)           | pending                        | 1.5, 2.4, 2.9, 4 |
| [019/04/02 runs and conclusions](./plans/019-uninstalled-app-leftovers/04-finish/02-run-and-conclusions.md)  | pending                        | 1.6, 2.5, 2.9, 4 |
| [020/01/02 suffixed names bench](./plans/020-downloads/01-core-prerequisites/02-suffixed-names.md)           | step done, bench re-run pending | 4             |
| [020/05/02 runs and conclusions](./plans/020-downloads/05-finish/02-run-and-conclusions.md)                  | pending, in progress           | 1.7, 2.6, 2.9, 4 |
| [021/04/02 runs and conclusions](./plans/021-stale-projects/04-finish/02-run-and-conclusions.md) | pending | 1.8, 2.7, 2.9, 4 |
| [026/03/03 run and conclusions](./plans/026-dev-run-record/03-checker-and-finish/03-run-and-conclusions.md) | pending | 2.0 |
| [Deferred: the fast scan on your home](./deferred/fast-scan-home-run.md) (004/08/03)                         | deferred, due before release   | 5.1           |
| [Deferred: the first real cleanups](./deferred/removal-sandbox-run.md) (008/08/02)                           | deferred, due before release   | 5.2           |
| [023/03/02 runs and conclusions](./plans/023-local-ai-models/03-finish/02-run-and-conclusions.md)            | pending                        | 1.9, 2.8      |
| [024/04/02 runs and conclusions](./plans/024-unused-toolchains/04-finish/02-run-and-conclusions.md)            | pending                        | 1.11, 2.10 |
| [029/04/02 runs and conclusions](./plans/029-ollama-models/04-finish/02-run-and-conclusions.md)            | pending                        | 1.12, 2.11 |
| [030/03/02 runs and conclusions](./plans/030-installed-app-caches/03-finish/02-run-and-conclusions.md)      | pending                        | 1.13, 2.12    |
| [034/03/01](./plans/034-browser-caches/03-finish/01-docs-and-conclusions.md) | pending | 1.16, 2.14 |
| [027/04/02 checks and the walk](./plans/027-scan-screen/04-docs-and-finish/02-checks-and-walk.md)           | pending                        | 1.14          |
| [028/02/02 checks and the walk](./plans/028-ignore-item/02-docs-and-finish/02-checks-and-walk.md)            | pending                        | 1.14          |
| [031/03/02 runs and conclusions](./plans/031-xcode/03-finish/02-run-and-conclusions.md)                    | pending                        | 1.18, 2.16    |
| [032/06/01 docs and conclusions](./plans/032-docker/06-finish/01-docs-and-conclusions.md)                  | pending                        | 1.19, 2.17    |
| [035/05/01 docs and conclusions](./plans/035-large-files/05-finish/01-docs-and-conclusions.md)               | pending                        | 1.20, 2.18    |
| Sandbox `cleanup-guards` (no pending step)                                                                   | optional                       | 1.10          |

Writing the results down: for each item, ticked on the plan side means the box in that step's `## Conclusions` ticked and
the numbers written under it, on `main`. Then tell the agent: a plan moves to done only when you have looked at its
result ([`plans/README.md`](./plans/README.md)).
