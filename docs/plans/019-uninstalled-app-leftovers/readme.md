# 019 — Uninstalled app leftovers

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules, core, system, sandbox,
testing, e2e, app, app/ui, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The next module in the user's agreed order (developer caches, then uninstalled-app leftovers, downloads and
installers, permissions): dragging an app to the Trash removes only its bundle, and its caches, settings, saved
windows, web data, support folder and container stay in `~/Library` for good. This plan finds the data of apps that
are provably no longer installed, matched by bundle identifier only, explains it in plain English and offers it to the
Trash. It generalizes what [plan 016](../016-jetbrains/readme.md) built for JetBrains IDEs (`jetbrains.uninstalled`)
to every app. The idea: [uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md).

## Goals

1. Only a folder or file in the user's `~/Library` places named after a bundle id (`BundleId::names`) is ever
   considered; a name that is not an id is never matched, whatever it looks like (`Application Support/Notes`).
2. An id is offered only when no app with it is installed, found by Spotlight or running, every check could be made,
   and the id is proven to have been an app's.
3. One category per kind of leftover, each with its danger level, so "keep settings" is a choice of categories.
4. What the module recognizes and does not offer is reported as left out with its reason, never dropped.

## Non-goals (each stays in its idea)

- Removing an installed app with its data: [smart uninstaller](../../ideas/smart-uninstaller.md).
- Launch agents, login items and helper tools (`~/Library/LaunchAgents/<id>*.plist`): removing a loaded agent needs
  `launchctl`, and a plist may start a program outside the app; a later addition with its own danger.
- Folders named after the app's name, not its id (`Application Support/<Name>`, `Logs/<Name>`): matching a name is a
  guess (rule 1); missed, never mistaken.
- Anything outside the user's `~/Library` (`/Library/Application Support`, other users' homes): another user's or the
  system's data.
- The JetBrains IDEs' own folders: plan 016's module keeps them (they are named after the product, not the id).

## Scope and danger levels

| Category                 | Place (known folder + path)                                                     | Danger | Why                                                         |
| ------------------------ | ------------------------------------------------------------------------------- | ------ | ----------------------------------------------------------- |
| `app-leftovers.caches`   | `AppCaches` `<id>`, `Logs` `<id>`, `SavedState` `<id>.savedState`                | Safe   | rebuilt by the app if it is installed again                 |
| `app-leftovers.settings` | `Preferences` `<id>.plist`, `Library` `HTTPStorages/<id>`, `WebKit/<id>`, `Cookies/<id>.binarycookies` | Medium | the app's settings and sign-ins are lost on a reinstall     |
| `app-leftovers.data`     | `AppSupport` `<id>`, `Containers` `<id>`                                         | High   | may hold the only copy of documents the app kept for itself |

High for support folders and containers: a sandboxed app keeps its documents and databases there, and nothing tells
a cache from a library of notes (rule 1: when in doubt, higher). One finding per app and category, titled with the
app's bundle id (no name is known for a removed app, and one guessed from the id could mislead); it counts in the
app's line of the storage map. Every place is data (`places.toml`), never an OS path in code (rule 5).

## How "not installed" is decided

Rows checked in order, in `finish` (all of an id's leftovers are known only after the walk); the first that holds
keeps every leftover of the id as left out with its reason:

1. **Apple's own or the system's**: an id under `com.apple.`, or in the module's exclusion table (`exclude.toml`:
   ids the system or shared frameworks write for every app); not claimed.
2. **Not an app's**: no leftover of the id is a container (`Containers/<id>`, which the system makes only for a
   sandboxed app or extension) or a saved state (`<id>.savedState`, written only by an app with windows). A
   command-line tool, a plugin or a framework also writes `Caches/<id>` and `Preferences/<id>.plist` and has no bundle
   to find; without that proof the id is never offered.
3. **The scan could not see enough**: a restored scan, or a folder of the module's places left unread (the place's
   completeness, below); an unseen leftover may be recent.
4. **The apps could not be listed in full**: `installed_apps_complete()` fails; an unread bundle may be the app.
5. **Installed**: an installed app owns the id (`InstalledApp::owns`: its own or a helper's), or the id is an
   installed app's id followed by a dot, or the other way round (`com.example.App.Helper`, an app and its extension).
6. **Running**: a running app has the id: an app run from a disk image, from Downloads, or translocated.
7. **Used lately**: a leftover changed within the setting's days (30 by default, as plan 016), or at a time the walk
   could not read.
8. **Elsewhere on the Mac**: `find_apps` finds a bundle with the id anywhere Spotlight knows (another volume, a
   custom folder); Spotlight cannot be asked or its index is off; or a mounted volume is not indexed (below).
9. Otherwise every leftover of the id is offered, each in its kind's category.

What stays uncertain, and the answer to each: an app on a drive that is not connected now cannot be found by any
means (the leftovers waited 30 days unchanged, the finding says "If this app is on a drive that is not connected, keep
these", the user confirms each, and they go to the Trash); an app whose bundle id changed between versions leaves the
old id's data, which is a true leftover; an iOS app on Apple silicon or a web app wrapper that the listing cannot read
makes the list partial (row 4), never "not installed".

**Never offered** (left out with a reason, or not claimed): a link at a place (not claimed); a folder read in part
(`inside.complete` false); one holding an entry not the user's, locked or flagged (`inside.foreign`); one holding a
socket or a pipe; anything in `Group Containers` (below); any name that is not a bundle id. An item a process holds
open is skipped by the cleaner's in-use check; a container the system refuses to move is skipped with its reason
([`deletion-safety.md`](../../restrictions/deletion-safety.md)).

## Core and system changes

- **Interests that match only bundle-id names** (`Pattern::Under` with an id-shaped `*`, checked by the walk): a
  whole-folder claim of `Caches/*` must never swallow `Caches/JetBrains` or the developer caches' places of plan 018.
- **The completeness of a module's places**: `cx.is_walk_complete()` is false on any Mac without Full Disk Access (a
  protected folder elsewhere is unread), which would make the module offer nothing; a per-interest answer says
  whether every folder at the module's places was read. The `jetbrains` module keeps its global check.
- **A removed app's name** ([deferred](../../deferred/removed-app-name-in-map.md)): `NewFinding::for_app_named(id,
  name)`, carried to the map's `AppRef` when no installed app has the id; `jetbrains` passes its product's name; this
  module has no name to pass and keeps `for_app(id)`.
- **A volume Spotlight does not index**
  ([known issue](../../known-issues/app-spotlight-misses-an-excluded-volume.md)): before offering, the system reader
  asks `mdutil -s` for each mounted local volume through `CommandRunner`, with a time limit; a volume not indexed, a
  network volume or a timeout keeps everything (row 8). Both modules gain it; the known issue is closed.

## Group, settings and the "keep settings" choice

A new group **Apps** ("What apps leave behind"), where the smart uninstaller and downloads will sit later. One setting,
`app-leftovers.uninstalled-after-days` (30, as `jetbrains`), and the module's switch. The user's one-click choice of
the idea is a selection shortcut on the module's page, per app, with the texts of rule 7: _Remove everything_ —
"Deletes all files this app left behind. If you install it again, it starts fresh." and _Keep settings_ — "Deletes
only caches and logs. Your settings stay, in case you install the app again." It selects; the confirmation still
lists the items and names the highest danger level.

## Testing

- **Unit**: the decision, every row, negatives first; `places.toml` and `exclude.toml` parsed and checked; the
  id-shaped pattern; the per-interest completeness; `for_app_named` in the map.
- **Fixtures**: `fixtures/homes/app-leftovers-*.toml` under an injected root: look-alike names (`com.example.Appx`,
  `Notes`), `com.apple.*`, an id with caches and preferences only (no proof), an installed app, a helper's id, a
  running app, an app found elsewhere, Spotlight off, a volume not indexed, a partial list, a recent leftover, a link,
  a foreign entry, a group container; then each kind; golden `fixtures/golden/app-leftovers.json`; the contract test
  with `Catalogue::Required`.
- **Sandbox**: `fixtures/sandboxes/app-leftovers.toml`, negatives first; `world.toml` gains what Spotlight answers
  (an app found elsewhere, the index off), part of [the sandbox world](../../deferred/sandbox-world.md), item 2;
  `npm run dev:sandbox -- app-leftovers` and `EXPECTED.md` walked by hand.
- **Core e2e**: scan, confirm, to the Trash and put back on a fixture home, on the shared home harness (below).
- **UI e2e**: the Apps group, the three categories, the left-out reasons, the two choices, the confirmation's danger.
- **By hand**: a scan of the developer's Mac (scan only): every offered id is checked with `mdfind` and by hand to be
  no installed app; nothing under `com.apple.` or `Group Containers` is offered; the scan time stays within 5 % of
  `main` (rule 4); then one real cleanup of a removed test app's leftovers and its put-back.

## Plans (after approval)

| #   | Plan        | What                                                                                                  |
| --- | ----------- | ----------------------------------------------------------------------------------------------------- |
| 01  | The core    | id-shaped interests, the places' completeness, `for_app_named`, the volume index check, the Apps group |
| 02  | The module  | the crate, `places.toml`, `exclude.toml`, the decision and its left-outs, texts, category docs, fixtures |
| 03  | End to end  | the sandbox's Spotlight answers and case file, the shared e2e home and this module's flows, the UI e2e |
| 04  | Finish      | the docs, the runs by hand, the conclusions                                                           |

01 and 02 in parallel once 01's API is fixed (02 builds on fakes); 03 needs both. Depends on plans 016, 017 and 018
merged.

## Risks

- **The worst false positive is a High container of an app the check missed**: rows 2 to 8 all must fail to keep it,
  the user confirms it with High shown, and it goes to the Trash.
- **Containers are protected by macOS** (the "data from other apps" prompt): reading them may prompt or be refused;
  plan 006's permissions decide when the walk enters them, and an unread container is never offered (row 3).
- **Too few proven ids**: apps that are not sandboxed and never saved a window are missed; the run by hand counts how
  much the proof leaves out, and a later proof (the app's name in its support folder) is weighed then.
- **Overlap with `jetbrains` and `dev-caches`**: one claim per byte; the id-shaped pattern keeps them apart, and
  fixtures with both modules' places check it.

## Ideas and deferred items it touches

- [Uninstalled app leftovers](../../ideas/uninstalled-app-leftovers.md): becomes this plan, the vendor-uninstaller
  question included (the same rows cover it). [Smart uninstaller](../../ideas/smart-uninstaller.md) stays an idea.
- [A removed app's name in the storage map](../../deferred/removed-app-name-in-map.md): due here, done in plan 01.
- [The module harnesses of the core e2e share their code](../../deferred/e2e-module-harness-shared.md): due at this
  harness, done in plan 03 before this module's home is added.
- [The sandbox world](../../deferred/sandbox-world.md): partly taken (Spotlight's answers); open files and steps stay
  deferred. [Why an item matched](../../deferred/match-explanations.md) is not due (code, not rules).
  [A module names its app's data folders](../../deferred/module-names-app-data.md) is not due (these folders are
  named after the id already).

## Open questions

1. **Group containers** (`~/Library/Group Containers/<team>.<name>`), often large, are shared by a developer's apps
   and not named after one app's id. Leave them out of this plan (recommended), or offer one, High, when no installed
   or running app has its team id?
2. **Support folders and containers at High**: offer them in this plan (recommended: the gigabytes are there, the
   danger is shown), or ship caches and settings first and add the High category after a release of use?

## Decisions

None yet.

## Agent's calls (to check)

1. **Bundle id only, never a name**: a folder not named after an id is not matched (rule 1).
2. **An id must be proven an app's** (a container or a saved state): tools and frameworks write id-named caches too.
3. **Three categories by kind: Safe caches, Medium settings, High data**: "keep settings" is a choice of categories.
4. **"Not installed" needs every check to answer**: a full app list, no owner by id or helper, not running, not in
   Spotlight, every volume indexed; any unknown keeps everything.
5. **Unchanged for 30 days, a setting**, as `jetbrains`: a just-removed app may be installed again.
6. **`com.apple.*` and an exclusion table never offered.**
7. **Group containers and launch agents left out**: shared or loaded; each a later addition.
8. **Per-place completeness in the core**: the global check would block the module on every Mac without Full Disk
   Access.
9. **Id-shaped interests in the core**: a `*` claim must not swallow other modules' places.
10. **The volume index check closes the known issue for both modules.**
11. **`for_app_named` done here, for `jetbrains`**: the deferred item is due; this module shows the id, no guessed name.
12. **"Keep settings" as a selection shortcut**: the confirmation is unchanged.
13. **One finding per app and category**: the user sees each app once per kind.
14. **The Trash only**: no app's or the system's own removal is run.
15. **Module id `app-leftovers`, a new group Apps.**
16. **The e2e home harness shared in plan 03**: its deferred item is due at this harness.

## Conclusions

Not written yet.
