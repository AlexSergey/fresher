# Xcode archives

**Danger level:** High · **Rule:** none (the module's code, `modules/xcode`) · **Since:** 0.1.0

**Example:** `~/Library/Developer/Xcode/Archives/<date>/<name>.xcarchive`

## What it is

The builds you archived in Xcode to publish an app, with what you need to read its crash reports. Made by Xcode, when
you choose Product, then Archive.

## Where it lives

`~/Library/Developer/Xcode/Archives/<YYYY-MM-DD>/<name>.xcarchive`: one finding per archive. The parent folder's name is a date, the archive's name ends with `.xcarchive`, and the archive holds `Info.plist`; without any of these it is not claimed. The path and sign are to verify on a Mac that has archives (plan 031, 03/02).

## What happens if you remove it

You can no longer read that version's crash reports or publish that build again. No: nothing makes an archive again.
That is why the level is High: an archive may hold the only copy of the build and the debug symbols of a version people
still use. Keep the archives of every version still in use.

## How the app decides

The module `xcode` (code with a data table, `modules/xcode/places.toml`). For the folder, the first answer that applies:

| #  | Situation                                                                              | Result                                       |
| -- | -------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1  | not a folder (a file, a link, a socket)                                                | not claimed                                  |
| 2  | the name does not read as an archive in a dated folder                                  | not claimed                                  |
| 3  | the sign (`Info.plist`) is not inside                                                  | not claimed                                  |
| 4  | the folder could not be read in full                                                   | not claimed                                  |
| 5  | nothing in it                                                                          | not claimed                                  |
| 6  | a socket, a pipe or a device inside                                                    | left out: a running program may be using it  |
| 7  | not yours, locked by the system, restricted or locked by you, or something inside is   | left out: removing it needs an administrator |
| 8  | the open apps cannot be listed                                                         | left out: Fresher cannot tell                |
| 9  | Xcode or the Simulator is open                                                         | left out: the app is open                    |
| 10 | otherwise                                                                              | **offered**                                  |

Never offered: a folder that is not an `.xcarchive`, one outside a dated parent, one without `Info.plist`, a link, and Xcode's settings. Nothing is chosen for you. Times are not read: an archive is offered whatever its age. The archive goes to the Trash; the app runs no command. An item a process holds open when you confirm is skipped.
