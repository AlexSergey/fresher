# Apps you uninstall, keeping their data

**Danger level:** Medium · **Rule:** none (the core's code, `crates/engine/src/apps/uninstall.rs`; made by no module: the
user chooses it on an app's page) · **Since:** 0.1.0

**Example:** `<name>.app` in `/Applications` and its `~/Library/Caches/<id>`

## What it is

One installed app, chosen on its page with "Remove the app, keep its settings and data" (the default choice): the
bundle with the caches, logs, saved windows and launch agents (login items) the app made. One finding of one item, whose
paths move to the Trash together.

| Text          | Key                              | Wording |
| ------------- | -------------------------------- | ------- |
| Name          | `core.uninstall-keep-data.name`        | Apps you uninstall, keeping their data |
| What it is    | `core.uninstall-keep-data.what`        | An app you chose to uninstall from its page, with the caches, logs, saved windows and login items it made. |
| Made by       | `core.uninstall-keep-data.made-by`     | You installed the app; the app made the rest while it ran. |
| If removed    | `core.uninstall-keep-data.if-removed`  | The app and those files go to the Trash together. Its settings and the files it keeps for you stay, so it starts where it left off if you install it again. |
| Comes back    | `core.uninstall-keep-data.comes-back`  | Only if you install the app again. To undo it, put it back from the Trash or from Cleanups. |

The finding is titled by the app's name and gives the reason "{app} and the caches, logs, saved windows and login items
it made: {size}. Its settings and files stay." (`core.uninstall.reason.keep-data`).

## Where it lives

The paths of one installed app on its page (the Apps screen), never on Scan: the app's bundle (`<name>.app` directly in
`/Applications` or `~/Applications`, or one folder below) and the first-level folders and files named exactly by its
bundle id or a helper's in `~/Library`:
`~/Library/Caches/<id>`, `~/Library/Logs/<id>`, `~/Library/Saved Application State/<id>.savedState` and
`~/Library/LaunchAgents/<id>.plist` (or `<id>.<x>.plist`). Its settings, website data, support folder, containers and
group containers are listed on the page and stay.

## What happens if you remove it

The bundle and those files go to the Trash as one item and come back as one item (Cleanups, or Put Back). The app is
gone from the Apps list; its settings, sign-ins and files stay, so installing it again starts where it left off.
Medium, not Safe: the app itself goes, and a launch agent it installed goes with it.

## How the app decides

It does not decide: the user chooses. The engine builds the finding from the app's footprint at that moment
(`prepare_uninstall`), never in a scan and never selected on Scan. Only an offered app can be chosen (not Apple's, not a
package manager's, not one that needs an administrator or is open, not Fresher's own); a part another installed app may
share (the same bundle id, a group container of a shared team), a link, a part not read in full and an ignored path are
kept and never in the plan. More than 64 paths are refused, never cut. The plan is judged again at prepare and at
confirm (the app must not have been opened, the bundle must be as listed); it always goes to the Trash: the
confirmation offers no permanent delete and the engine refuses one. See
[`core-uninstall-everything.md`](./core-uninstall-everything.md) for the other choice.
