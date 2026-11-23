# Xcode device support files

**Danger level:** Low · **Rule:** none (the module's code, `modules/xcode`) · **Since:** 0.1.0

**Example:** `~/Library/Developer/Xcode/iOS DeviceSupport/17.2 (21C62)`

## What it is

Debug information Xcode copies from an iPhone, iPad, Apple Watch, Apple TV or Vision Pro the first time you connect it, one folder per system version. Made by Xcode.

## Where it lives

`~/Library/Developer/Xcode/iOS DeviceSupport/<name>`, and the same under `watchOS DeviceSupport`, `tvOS DeviceSupport` and `visionOS DeviceSupport`. One finding per folder. The name is `[<model> ]<version> (<build>)`, for example `17.2 (21C62)`, optionally with the device model first. The folder holds `Symbols`; without it the folder is not claimed. The paths and signs are to verify on a Mac that has such folders (plan 031, 03/02).

## What happens if you remove it

The folder goes to the Trash. You can still build and run apps; debugging on a device of that version waits until Xcode copies the files again. Yes: Xcode copies them again when you connect a device on that version, which takes a few minutes. That is why the level is Low, not Safe.

## How the app decides

The module `xcode` (code with a data table, `modules/xcode/places.toml`). For the folder, the first answer that applies:

| #  | Situation                                                                            | Result                                       |
| -- | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| 1  | not a folder (a file, a link, a socket)                                              | not claimed                                  |
| 2  | the name does not read as a device support name                                      | not claimed                                  |
| 3  | the sign (`Symbols`) is not inside                                                   | not claimed                                  |
| 4  | the folder could not be read in full                                                 | not claimed                                  |
| 5  | nothing in it                                                                        | not claimed                                  |
| 6  | a socket, a pipe or a device inside                                                  | left out: a running program may be using it  |
| 7  | not yours, locked by the system, restricted or locked by you, or something inside is | left out: removing it needs an administrator |
| 8  | the open apps cannot be listed                                                       | left out: Fresher cannot tell                |
| 9  | Xcode or the Simulator is open                                                       | left out: the app is open                    |
| 10 | otherwise                                                                            | **offered**                                  |

The two newest versions of each platform are still offered, at the same level, with a reason that says a device you use may need them. Never offered: a name of another shape (`Notes`), a folder without `Symbols`, a link, and Xcode's settings. Times are not read. The folder goes to the Trash; the app runs no command. An item a process holds open when you confirm is skipped.
