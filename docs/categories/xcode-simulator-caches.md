# Simulator caches

**Danger level:** Safe · **Rule:** none (the module's code, `modules/xcode`) · **Since:** 0.1.0

**Example:** `~/Library/Developer/CoreSimulator/Caches`

## What it is

Files the iOS simulators keep to start faster, one set for each version of macOS you ran them on. Made by the Simulator, part of Xcode.

## Where it lives

`~/Library/Developer/CoreSimulator/Caches`: one finding, the folder itself. Verified on the developer's Mac: it holds one folder per macOS build under `dyld`, and an old macOS build's folder stays after an update. The folder holds `dyld`; without it the folder is not claimed.

## What happens if you remove it

What is inside the folder goes to the Trash and the folder stays. Your simulators and their apps stay: they live in `CoreSimulator/Devices`, which is not touched. Yes: the next time a simulator starts it makes its shared cache again; that start takes longer.

## How the app decides

The module `xcode` (code with a data table, `modules/xcode/places.toml`). For the folder, the first answer that applies:

| #  | Situation                                                                              | Result                                       |
| -- | -------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1  | not a folder (a file, a link, a socket)                                                | not claimed                                  |
| 2  | the name does not read as the folder's name (it is not judged)                                  | not claimed                                  |
| 3  | the sign (`dyld`) is not inside                                                  | not claimed                                  |
| 4  | the folder could not be read in full                                                   | not claimed                                  |
| 5  | nothing in it                                                                          | not claimed                                  |
| 6  | a socket, a pipe or a device inside                                                    | left out: a running program may be using it  |
| 7  | not yours, locked by the system, restricted or locked by you, or something inside is   | left out: removing it needs an administrator |
| 8  | the open apps cannot be listed                                                         | left out: Fresher cannot tell                |
| 9  | Xcode or the Simulator is open                                                         | left out: the app is open                    |
| 10 | otherwise                                                                              | **offered**                                  |

Never offered: a simulator device or runtime (deferred), DerivedData (that is [`dev-caches`](./dev-caches-xcode-derived-data.md)' own), a link. Times are not read. The folder is emptied into the Trash, never removed; the app runs no command. An item a process holds open when you confirm is skipped.
