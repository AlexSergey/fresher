# Old installers

**Danger level:** Low · **Rule:** none (the module's code, `modules/downloads`) · **Since:** 0.1.0

**Example:** a `.dmg` or `.pkg` in `~/Downloads` of an app already installed

## What it is

Disk images (`.dmg`) and packages (`.pkg`) you used to install apps that are on this Mac now, at this version or
newer. Made by you, when you downloaded the app.

## Where it lives

`~/Downloads`, first level only, one finding per file ending `.dmg` or `.pkg`, case ignored.

## What happens if you remove it

The app stays installed; to install it again, download it again. It comes back only if you download it again.

## How the app decides

The module `downloads` (code with data tables, `modules/downloads/installers.toml`). A file is claimed only with
proof, and nothing is mounted, expanded or run. A `.pkg` is proven by a receipt of the same name installed at or after
the file's newest change time. A `.dmg` or `.pkg` is proven by its name: the name without its extension names exactly
one installed app that is not Apple's (by its name or the last part of its id), followed only by versions, plain numbers
and the words of the table (`installer`, `arm64`, ...); `Firefox 133.0.dmg` is named, `notes-backup.dmg` and
`Firefox (1).dmg` are not. Then, in order: no receipt and the apps cannot be listed, left out; the name names no app or
two, not claimed; the list of apps is partial, left out; a restored scan, left out; the app changed before the
installer, or is older than the version in the name, or cannot be compared, left out; otherwise **offered**, as soon as
it is proven. A file that is not offered is never offered as a big download. Nothing in Downloads is read unless you
turn on its switch in Settings or grant Full Disk Access. The files go to the Trash.
