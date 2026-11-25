# Apps and their data

**Danger level:** High · **Rule:** none (the core's code, `crates/engine/src/apps/uninstall.rs`; made by no module: the
user chooses it on an app's page) · **Since:** 0.1.0

**Example:** `<name>.app` with its `~/Library/Application Support/<id>`

## What it is

One installed app, chosen on its page with "Remove the app and everything it left": everything the keep-data choice
takes ([`core-uninstall-keep-data.md`](./core-uninstall-keep-data.md)) and the settings, the website data, the support
folder, the container and the group container the app keeps for the user. One finding of one item, whose paths move to
the Trash together.

| Text          | Key                              | Wording |
| ------------- | -------------------------------- | ------- |
| Name          | `core.uninstall-everything.name`        | Apps you uninstall with everything they left |
| Line          | `core.uninstall-everything.line`        | The app and everything it left |
| What it is    | `core.uninstall-everything.what`        | An app you chose to uninstall from its page, with every file of it Fresher found by its id: caches, logs, settings, sign-ins and the files it keeps for you. |
| Made by       | `core.uninstall-everything.made-by`     | You installed the app; the app made the rest while it ran. |
| If removed    | `core.uninstall-everything.if-removed`  | The app and all those files go to the Trash together. If you install it again, it starts fresh: its settings, sign-ins and the files it kept for you are gone. |
| Comes back    | `core.uninstall-everything.comes-back`  | No. To undo it, put it back from the Trash or from Cleanups. |

The finding is titled by the app's name and gives the reason "{app} and every file of it Fresher found: {size}."
(`core.uninstall.reason.everything`).

## Where it lives

The paths of one installed app on its page (the Apps screen), never on Scan: the app's bundle (`<name>.app` directly in
`/Applications` or `~/Applications`, or one folder below) and the first-level folders and files named exactly by its
bundle id or a helper's in `~/Library`:
the keep-data places plus `~/Library/Preferences/<id>.plist`, `~/Library/HTTPStorages/<id>`, `~/Library/WebKit/<id>`,
`~/Library/Cookies/<id>.binarycookies`, `~/Library/Application Support/<id>`, `~/Library/Containers/<id>` and
`~/Library/Group Containers/<team id>.<anything>`. A container or group container goes only when the access to other
apps' data is granted, and a group container only when it carries the app's own team id and no other installed app
shares it.

## What happens if you remove it

Everything goes to the Trash as one item and comes back as one item. If the app is installed again it starts fresh:
its settings, sign-ins and the files it kept for you are gone once the Trash is emptied. High: the support folder and
the containers may hold what the user made and cannot make again.

## How the app decides

As for keep-data: the user chooses, the engine builds the finding at that moment, the same refusals and kept parts, the
same two judgements at prepare and at confirm, always the Trash (no permanent delete is offered or accepted). Files an
app keeps under its name rather than its id are not found (the page says so); a name table is deferred
([`../deferred/uninstall-system-files.md`](../deferred/uninstall-system-files.md)).
