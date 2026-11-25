# Old temporary files

**Danger level:** Safe · **Rule:** none (the module's code, `modules/temp`) · **Since:** 0.1.0

**Example:** items in your temporary folder untouched for a week

## What it is

Working files that apps and command-line tools put in your Mac's temporary folders and did not clean up. Nothing has
changed them for at least a week (the setting runs from 7 to 30 days and can only be raised).

## Where it lives

- your temporary folder, the one `$TMPDIR` points to: `/private/var/folders/<xx>/<hash>/T`;
- the shared temporary folder, `/private/tmp` (the link `/tmp` points to it): only your own items.

Each item directly inside one of them is one finding, a file, a folder or a link, taken whole. macOS clears these
folders on its own: every day it removes what is older than three days, and a restart empties `/private/tmp`.
Fresher only helps with what is still there after a week.

## What happens if you remove it

The item goes to the Trash; the space comes back when you empty it. Apps keep their real data elsewhere and make new
temporary files as they work. An item named after an app that is open is kept, and an item that is in use when you
confirm is skipped.

## How the app decides

The module `temp` (code, not rules: the answer depends on times, kinds, owners and the running apps). For each item,
the first answer that applies:

| #  | Situation                                                                                                    | Result                                         |
| -- | ------------------------------------------------------------------------------------------------------------ | ---------------------------------------------- |
| 1  | the name is a lock or a system marker (`.X*-lock`, `.X11-unix`, `.ICE-unix`, `.font-unix`, `.XIM-unix`, `quota.user`, `quota.group`, `.vfs_rsrc_streams_*`, `.AddressBookLocks`, `.LINKS`, `*.lock`, `*.pid`, `LCK..*`, `.#*`), or a folder holds an entry named `*.lock`, `*.pid`, `LCK..*` or `.#*` anywhere inside | kept for good: a lock |
| 2  | the item, or something inside it, is a socket, a pipe or a device                                            | kept for good: a program uses it to talk to another |
| 3  | not yours, locked by the system, restricted, holds something inside that is not yours or locked, or named `com.apple.…` | kept for good: it belongs to the system |
| 4  | locked by you (Finder's Get Info)                                                                            | left out: locked                               |
| 5  | the open apps cannot be listed                                                                               | left out: Fresher cannot list open apps        |
| 6  | named after an app that is open: its bundle id as the whole name or followed by a dot, also with one leading dot stripped (`.com.example.App.XXXXXX`) | left out: the app is open |
| 7  | a folder whose inside could not be read in full                                                              | not claimed                                    |
| 8  | the newest change anywhere inside is unknown, in the future, or not more than the setting's days ago, or a file inside was read within those days | left out: changed or read recently |
| 9  | otherwise                                                                                                    | **offered**                                    |

"More than N days" is exact: seven days is kept, seven days and one second is offered. The age is the newest
modification or status change of the item and of everything inside it; an access time only keeps, never offers. A
link is offered as itself; the Trash gets the link, never its target. System and administrator items are shown as left
out with their reason and never offered.

Names are read as bytes, never as text, and case is ignored: a name that is not valid UTF-8 is matched like any other,
and a wider match only keeps more.

Places it never touches: `/private/var/tmp`, the system's other folders in `/private/var/folders` (`C/`, `0/`, `X/`
and other users' folders), `T/TemporaryItems`, `~/Library/Autosave Information` (unsaved documents) and everything
outside the two places above.

The place's contract: a name the module cannot read is no reason to keep an item here. macOS itself removes anything
in these folders after three days, so a top-level item of your own temporary folder, or your own item in
`/private/tmp`, that passes every row above is offered whatever its name. This is the one exception to "Unknown is
not junk" ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md)); nowhere else does a name the
app cannot read count. The walk reads each place as a root of its own, so an item of a place is cleaned only while a
module names that place; once none does, the cleaner refuses it.
