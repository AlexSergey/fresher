# Data of JetBrains IDEs that are no longer installed

**Danger level:** Medium · **Rule:** none (the module's code, `modules/jetbrains`) · **Since:** 0.1.0

## What it is

The settings with their plugins, the caches and the logs of a JetBrains IDE (IntelliJ IDEA, WebStorm, DataGrip and
the others) that is no longer installed on this Mac. Removing an IDE leaves every version's folders behind; each
finding names the IDE and the version.

## Where it lives

The version folders of the IDE in the three places of the other JetBrains categories:
`~/Library/Application Support/JetBrains/<Product><Year>.<N>` (settings and plugins),
`~/Library/Caches/JetBrains/<Product><Year>.<N>` (caches) and `~/Library/Logs/JetBrains/<Product><Year>.<N>` (logs),
for example `DataGrip2024.3`. One finding per folder.

## What happens if you remove it

The folders go to the Trash. Nothing installed uses them. If you install that IDE again, it starts from scratch:
default settings and keymaps, no plugins, and its caches are built again. It does not come back unless you install the
IDE again and use it. Medium, not Safe: the settings and plugins are among them, and they cannot be made again.

## How the app decides

The module `jetbrains` recognizes the folders exactly as for the
[older versions' settings](./jetbrains-settings.md): a folder directly inside `JetBrains/`, named exactly a known
product's folder name and `<year>.<minor>`; a file, a link or any other name is never claimed. The rows of that file
apply, in their order. When the first three keep nothing and no installed app has one of the product's bundle ids
(row 4), every folder of the product, in the three places, is offered here only when all of these hold:

- the installed apps were listed completely: every apps folder (`/Applications`, `~/Applications`,
  `/System/Applications` and one folder below each, `~/Applications/JetBrains Toolbox` among them) and the older
  Toolbox app's own folder (`~/Library/Application Support/JetBrains/Toolbox/apps`, up to four folders below) were
  read, and every app bundle in them (an app that is a link is read where it points; a folder link, another volume's
  folder or a link to no bundle makes the list incomplete); otherwise row 1 keeps every folder;
- no app of the vendor that the products table does not know is installed (it may use the data);
- no app of the product (its release or its early access build) is running, and no app of the vendor the products
  table does not know is running (rows 2 and 3; the Toolbox app does not count);
- none of the product's folders, in the three places, changed in the last 30 days: the newest modification time of
  the folders is at least that old. The days are the setting "Offer the data of a removed IDE after"
  (`jetbrains.uninstalled-after-days`, 7 to 3650, 30 by default);
- the scan walked the disk and read every folder: a scan restored from the last one's cache never offers, its times
  are old, and neither does a walk that could not read a folder, which may be one of the product's that changed;
- Spotlight knows no app with one of the product's bundle ids anywhere on the Mac (an IDE on another drive or in a
  download folder); before it is asked, once per scan, Spotlight must know the Finder (its index is on) and every
  mounted volume the system shows, and the one that holds the home, must be local and indexed (`mdutil -s`, 10 s
  each). If Spotlight cannot be asked, its index is off, a volume is not indexed or a network volume is mounted,
  everything is kept: "Fresher could not ask Spotlight about every drive, so this IDE may be installed elsewhere. It
  keeps its data. If Spotlight is off, or a drive is not indexed, turn it on and scan again."

Otherwise every folder of the product is kept and reported as left out here, with the reason: its data changed
recently (or the scan is restored, or could not read every folder), an unknown JetBrains app is installed, the IDE is
installed elsewhere, or the app could not ask Spotlight about every drive. A folder whose time cannot be read, or is
in the future, counts as changed recently. The time is the folder's own: an IDE changes it when it starts or quits (its lock files
and its rotated logs), not when it only writes inside the folder's subfolders.
