# High and Largest, in a sandbox

What only a person on a real window can see of the High level: the "Only if you are sure" group, the High tick in the
confirmation, the Largest screen, and an uninstall of everything. The autotests hold the same screens with a mocked
core; these cases run them on a real scan. Nothing outside the sandbox is touched: the sandbox's own Trash
(`sandbox/<feature>/home/.Trash`) is the one used. How sandbox mode works:
[`../testing.md`](../testing.md#sandbox-mode).

## Setup

From the repository's root, one sandbox per case, the development build in sandbox mode:

```sh
rm -rf sandbox/app-leftovers && npm run dev:sandbox -- app-leftovers    # HL-01, HL-02
rm -rf sandbox/large-files   && npm run dev:sandbox -- large-files      # HL-03
rm -rf sandbox/uninstall     && npm run dev:sandbox -- uninstall        # HL-04
```

Each sandbox has its `EXPECTED.md` with every item it plants. Quit the window before starting the next one. A fresh
sandbox asks to accept the short agreement before its first cleanup: "I understand" goes on.

## Cases

### HL-01 The "Only if you are sure" group

`app-leftovers`. Scan. On Scan the group **Only if you are sure** lists **Data of removed apps**, its tile unticked;
the group has no "Select all" (a group of what needs a look in Review, like this one, never has it). Open the tile's details: they
name the group, "Only if you are sure", and the danger level High, "May hold the only copy of your data", with the
texts "What it is", "If you remove it" and "Comes back?". The item is the support folder of the removed app,
`com.example.Gone4`.

### HL-02 The High tick in the confirmation

`app-leftovers`. Tick **Data of removed apps** on Scan: the selection bar says it is to check in Review. Press Review,
tick **All Data of removed apps** (or each item), then **Move to the Trash**. The confirmation opens with the tick
"I understand these may be the only copy of my data" unchecked, and **Yes, move to the Trash** disabled. "Delete
permanently instead" is unchecked too. Tick the High tick: the button is enabled. Cancel and open it again: the tick is
cleared again. Tick it and confirm: "Moved to the Trash"; `home/Library/Application Support/com.example.Gone4` is in
`home/.Trash`. In Cleanups the cleanup lists it as in the Trash; **Put back** returns the folder to its place.

### HL-03 Largest's ticks

`large-files`. Scan. **Largest** is in the sidebar. On **Files** (the list "Largest files, largest first") the sandbox's
large files are listed largest first, each with its kind and place: `Installer.dmg` (Disk image), `Old.utm` (Virtual
machine), `Photos.zip` (Archive, with a line naming `Photos` beside it); nothing from `Library`, the Trash, a hidden
folder, inside a package, or below 10 MB. Tick `Installer.dmg` ("Choose Installer.dmg"): the selection bar says "1 item
selected". Review shows it under "Your own files" as "Only if you are sure". Press **Move to the Trash**: the
confirmation asks the High tick (as HL-02), disabled until ticked. Confirm: `Installer.dmg` is in `home/.Trash`, and
Largest no longer lists it, with a notice that a cleanup changed the disk. Cleanups: **Put back** returns it. On
**Folders** and **Browse** have no checkbox ([LF-01, LF-04](./largest-folders.md)).

### HL-04 Uninstall everything

`uninstall`. Scan, open **Apps**, open **Purger**. The page offers two choices, "Remove the app, keep its settings and
data" (chosen at first) and "Remove the app and everything it left", and the **Uninstall…** button. Choose the second
and press Uninstall…: the confirmation names High, opens with the High tick unchecked and **Yes, move to the Trash**
disabled, and "and N more files that go with it" opens the list: its caches, logs, saved windows, login item, settings
and `Application Support` folder. Tick and confirm: `Applications/Purger.app` and all its files are in the sandbox's
Trash, and the app leaves the Apps list. Cleanups: **Put back all still in the Trash** returns all of them. With the
first choice, the confirmation shows no High tick and its button is enabled at once.
