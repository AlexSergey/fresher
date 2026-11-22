# 036/05/01 — The docs, the checks and the conclusions

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

Every doc the plan's change affects, written as built (a doc an earlier step already updated is checked, not
rewritten); the branch's checks; the plan's conclusions with the status 🔨 "awaiting the user's walk"; and the plan's
section in the user's review list. English only; no other product or reference project named.

## Before you start

Read the plan's readme, `00-design.md` and every step's conclusions; `docs/plans/README.md` (format), the docs listed
below, `docs/deferred/README.md` and `docs/deferred/user-review-2026-11-22.md` (the sections of plans 033 and 034 are the
model). Run `git log --oneline <base>..HEAD` to see what was built.

## Files

- `docs/architecture/README.md` — the crates table (`fresher-core`: the apps view types; `fresher-system`: the bundle
  reader; `fresher-engine`: the Apps view and the uninstall plan); the ports table's `InstalledApps` row (`bundle`:
  the measure, Spotlight's date, Homebrew's casks; `BundleReader::stated` in sandbox mode); a section **Apps and
  uninstall** after "The storage map": the list over the last committed scan, the footprint by bundle id, the cache
  measured once per state outside the locks, the uninstall as one engine-made finding of several paths, its context
  (the apps folder as a root, refused for every other plan), the paths that went, App Management learned; the UI's
  navigation (the `apps` and `app` screens, the sidebar order) and the cleanup state's `origin`.
- `docs/architecture/permissions.md` — App Management: never probed, `NotAsked` until the first uninstall, learned for
  the session, its pane (unverified).
- `docs/architecture/removal.md` — the threat model gains a row: "An app bundle removed by a plan that is not an
  uninstall" — the protection opens only whole bundles in the apps folders; the engine's context refuses the apps
  folder for every other plan; the cleaner's roots need the plan to name it — proved by
  `crates/core/src/cleanup/protection.rs`, `crates/engine/src/cleanup/context.rs` and
  `crates/e2e/tests/uninstall.rs` (`an_ordinary_cleanup_never_moves_a_bundle`). The cleaner's code is unchanged.
- `docs/restrictions/deletion-safety.md` — "Never follow symlinks out, never touch protected paths": `/Applications`
  lets through only a whole app bundle at its top or one folder below, and only for an uninstall the user confirmed;
  nothing inside a bundle; the system's apps folder never. Where it is enforced and its tests. "No administrator
  rights": an app that needs an administrator (App Store apps) is shown and never offered, with Show in Finder.
- `docs/categories/core-uninstall-keep-data.md` and `core-uninstall-everything.md` (new) in the catalogue's format
  (Medium and High), with the parts each takes; their rows in `docs/categories/README.md`, which also says that these
  two are the core's, offered only from an app's page, never on Scan.
- `docs/testing.md` — `[case.uninstall]` and the three `[[case.app]]` keys, the two case files, the harness's
  `uninstall.rs`, `crates/e2e/tests/uninstall.rs`, the UI scenarios.
- `docs/manual-testing.md` — the plan's walks as items `1.22` (the sandbox walk) and `2.20` (the dry run on the
  developer's Mac), after plan 038's `1.21` and `2.19`, linking here.
- `docs/ideas/README.md` — the ideas "Smart Uninstaller" and "Application Footprint" marked as replaced by plan 036
  (if rows remain); `docs/ideas/app-resource-profile.md` — one line: the Apps list it would extend exists.
- `docs/deferred/uninstall-system-files.md` (new) and its row in `docs/deferred/README.md` (agreed 2026-11-22, due "when
  the user asks for it after the first release"): an app's system-wide files (`/Library/LaunchDaemons`,
  `/Library/PrivilegedHelperTools`, `/Library/Application Support/<id>`, need an administrator: shown, never removed),
  files kept under the app's name rather than its id (a table of names to ids, as the deferred
  `installed-app-caches-more-places.md` needs), package managers beyond Homebrew casks and running their own uninstall
  command, watch mode's growing apps (plan 010).
- `docs/deferred/user-review-2026-11-22.md` — a section `## 036 — Uninstall an app` appended after plan 035's (the
  file and its README row exist), short bullets:
  - **Walks by hand**: `rm -rf sandbox/uninstall && npm run dev:sandbox -- uninstall`, walk `EXPECTED.md` line by line,
    negatives first (Chess and Notes not listed; Brewed shows Homebrew's sentence and command; Rooted shows "needs an
    administrator" and Show in Finder; Keeperx's cache untouched); uninstall Keeper with the default choice, then Purger
    with everything; read the confirmation (one item, "and N more files"); Cleanups → put back each; the Twin case.
    `npm run dev:sandbox -- uninstall-open`: Mailbox shows "open"; Sketch goes.
  - **The dry run on the developer's Mac** (`npm run dev`, a scan of `~`, then Apps): the list against
    `ls -l /Applications` (owners), `ls /opt/homebrew/Caskroom` (the Homebrew apps), the sizes against `du -sh
    /Applications/<app>.app` for three apps; the App Store apps not offered; an uninstall of one app is "a dry run"
    (nothing moves).
  - **Facts not verified**: whether macOS asks for App Management, refuses silently or shows its own notice on the
    first bundle moved (a release build, one throwaway app the user installs for it); the pane's address
    `Privacy_AppBundles`; whether moving a bundle with the Trash call keeps Finder's "Put Back"; how long the first
    measure takes on a Mac with Xcode.
  - **Agent's calls worth a second look**: the design's calls 3 (last used), 5 (App Store apps not offered), 7 (places
    as code), 8 (more than 64 paths refused), 9 (one item of several paths), 10 (the apps folder's opening), 14 (the
    selection kept).
  - **Known gaps**: files under the app's name, system-wide files, containers without other apps' data, no watch mode.
- `docs/plans/036-uninstall/` — every step's and plan's conclusions written as built (checked here); the global
  readme's **Conclusions**: what was built, what changed from the design and why, the numbers (tests, coverage, the
  scan's time unchanged, the measure's time in the sandbox), the calls taken while building; statuses 🔨 in progress,
  "awaiting the user's walk". `docs/plans/README.md`: the row 🔨 in progress.

## Checks

- `npm run pre-push` on the branch's last commit (lint, guards, knip, machete, deny, the script tests, the tests with
  coverage: at or above 90 % of lines in Rust and the UI, `crates/engine/src/apps` and `crates/system/src/bundle_facts`
  at or above 90 % each).
- `npm run test:e2e` (core and UI, chromium and WebKit); `npm run sandbox:test`.
- `git diff <base> -- crates/cleaner modules` is empty.
- `npm run lint` on the docs (Markdown and links); `npx prettier --check` on the changed docs.
- No performance run: the scan's walk, the scanner and the hot paths of the UI do not change; the measure runs only
  on the Apps screen ([`../../../testing.md`](../../../testing.md), "Performance"). Say so in the conclusions.

## Conclusions

- Written: the docs listed above, as built (`architecture/README.md` the crates, the ports and a section "Apps and
  uninstall"; `removal.md` a threat-model row; `deletion-safety.md` the apps folder's two walls; two category files;
  `testing.md`; manual-testing items 1.22 and 2.20; the deferred items `uninstall-system-files.md`,
  `uninstall-small-library-parts.md` and `app-management-denied-from-not-allowed.md`; the known issue
  `cleaner-app-opened-after-the-confirmation.md`; the review list's section 036). `permissions.md` was written by 02/02
  and checked here. The two ideas the brief names have no row left; `app-resource-profile.md` says the Apps list exists.
- Every step's conclusions were written here from the build's reports (a step that had written its own was kept).
- Checks: `npm run pre-push` on the last commit (numbers in the plan's conclusions); the gallery's Playwright project
  passes against the committed baselines (the uninstall-choices markup changed in step 03/02's fix and no baseline
  moved); `git diff 7cbd6624 -- crates/cleaner modules` is empty.
- Not run: `npm run test:e2e` and `npm run sandbox:test` in full (each step ran its own specs and the sandboxes); no
  performance run, for the reason in the plan's conclusions.
- Awaiting the user's walk: items 1.22 and 2.20.
