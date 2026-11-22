# Xcode previews

**Danger level:** Safe · **Rule:** none (the module's code, `modules/xcode`) · **Since:** 0.1.0

## What it is

What Xcode builds to show your SwiftUI previews. Made by Xcode.

## Where it lives

`~/Library/Developer/Xcode/UserData/Previews`: one finding, the folder itself. The folder holds `Simulator Devices`; without it the folder is not claimed. The path and sign are to verify on a Mac that has it (plan 031, 03/02).

## What happens if you remove it

What is inside the folder goes to the Trash and the folder stays. Yes: Xcode builds the previews again the next time you open a preview; the first one takes longer.

## How the app decides

The module `xcode` (code with a data table, `modules/xcode/places.toml`). For the folder, the first answer that applies:

| #  | Situation                                                                              | Result                                       |
| -- | -------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1  | not a folder (a file, a link, a socket)                                                | not claimed                                  |
| 2  | the name does not read as the folder's name (it is not judged)                                  | not claimed                                  |
| 3  | the sign (`Simulator Devices`) is not inside                                                  | not claimed                                  |
| 4  | the folder could not be read in full                                                   | not claimed                                  |
| 5  | nothing in it                                                                          | not claimed                                  |
| 6  | a socket, a pipe or a device inside                                                    | left out: a running program may be using it  |
| 7  | not yours, locked by the system, restricted or locked by you, or something inside is   | left out: removing it needs an administrator |
| 8  | the open apps cannot be listed                                                         | left out: Fresher cannot tell                |
| 9  | Xcode or the Simulator is open                                                         | left out: the app is open                    |
| 10 | otherwise                                                                              | **offered**                                  |

Nothing else of `UserData` is offered: your Xcode settings and key bindings stay. A link is never followed. Times are not read. The folder is emptied into the Trash, never removed; the app runs no command. An item a process holds open when you confirm is skipped.
