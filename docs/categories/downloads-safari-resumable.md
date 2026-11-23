# Safari downloads you can resume

**Danger level:** Medium · **Rule:** none (the module's code, `modules/downloads`) · **Since:** 0.1.0

**Example:** a paused Safari download, `<name>.download`, in `~/Downloads`

## What it is

Downloads Safari paused or could not finish. Double-clicking one in Finder resumes it. Made by Safari, while it
downloads a file.

## Where it lives

`~/Downloads`, first level only: a folder ending `.download` that holds `Info.plist`, one finding per folder, summed
whole.

## What happens if you remove it

It goes to the Trash, and the download cannot be resumed: you start it again from the beginning. It does not come back.

## How the app decides

The module `downloads`. The same four rows as unfinished downloads, with Safari's ids and the newest change inside the
folder: a restored scan, an unreadable list of running apps, Safari open, or a change within the days, are left out.
Then a folder not read in full, left out (scan again); one holding a socket, pipe or device, left out; one with a file
that is not yours or is locked, left out (needs an administrator); otherwise **offered**. A `.download` folder without
`Info.plist` is not claimed. Nothing in Downloads is read unless you turn on its switch in Settings or grant Full Disk
Access. The folders go to the Trash.
