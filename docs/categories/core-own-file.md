# Your own files

**Danger level:** High · **Rule:** none (the core's code, `crates/engine/src/own.rs`) · **Since:** 0.1.0

**Example:** a large file or folder in your home that you pick on the Largest screen

## What it is

Files and folders you picked yourself on the Largest screen. Made by you, or the apps you use; made by no module: the
user picks each item on the Largest screen.

The category's five texts (`crates/core/locales/en.json`):

| Text          | Key                         | Wording                                                                                        |
| ------------- | --------------------------- | ---------------------------------------------------------------------------------------------- |
| Name          | `core.own-file.name`        | Your own files                                                                                 |
| What it is    | `core.own-file.what`        | Files and folders you picked yourself on the Largest screen.                                   |
| Made by       | `core.own-file.made-by`     | You, or the apps you use.                                                                      |
| If removed    | `core.own-file.if-removed`  | They go to the Trash. Fresher cannot tell what they hold or whether you still need them.       |
| Comes back    | `core.own-file.comes-back`  | No. Put them back from the Trash or from Cleanups if you need them.                            |

Each item is titled by its name (`core.own-file.title`) and gives one of two reasons: "This is your own file; Fresher
cannot tell whether you still need it." (`core.own-file.reason.file`) or "This is your own folder; Fresher cannot tell
whether you still need what is in it." (`core.own-file.reason.folder`).

## Where it lives

Anywhere below the home folder that the Largest screen offers: never the home itself, `~/Library`, the Trash, a hidden
name or anything inside one, an app, a package's inside, a link, a place not scanned, a standard folder, a protected
place, a folder holding what a module claims, a place read in part, or a path of the ignore list.

## What happens if you remove it

It goes to the Trash. Anything you have only here is lost once you empty the Trash, so look at each one first. It does
not come back by itself; put it back from the Trash or from Cleanups.

## How the app decides

It never decides: no scan finds these items, nothing is pre-selected, and there is no switch or card for the category.
The Largest screen lists the home's large items and judges every row by one function; a row becomes an item only when
you tick it and the engine judges it offered at that moment (`Engine::pick_own_item`; any other row is refused
`not-own-item`). The item is then held apart from the scan's findings — in no total, card, findings page or the storage
map's explained part — and goes through the normal Review and confirmation at the High level, to the Trash by default.
It is judged again when the plan is prepared and when it is confirmed, and the cleaner's rehearsal reads the disk and
refuses what it must. A new scan forgets every pick.
