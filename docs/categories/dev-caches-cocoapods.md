# CocoaPods' download cache

**Danger level:** Low · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

**Example:** `~/Library/Caches/CocoaPods`

## What it is

Copies of the libraries CocoaPods downloaded for iOS and macOS projects. Made by CocoaPods, a dependency manager for
Swift and Objective-C projects.

## Where it lives

`~/Library/Caches/CocoaPods`, one folder. Only the tool's default place is looked at: the app reads no environment
variable and no config file. A cache that CocoaPods' config moves elsewhere is not found.

The tool's sign: the folder holds `Pods`, where CocoaPods keeps its downloads. Without it the folder is not claimed.

## What happens if you remove it

It goes to the Trash. Your projects keep the libraries already installed in them. Yes. CocoaPods downloads each library
again the next time a project installs it.

## How the app decides

The module `dev-caches` (code with a data table, `modules/dev-caches/tools.toml`). For the folder, the first answer that
applies:

| # | Situation                                                                            | Result                                       |
| - | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| 1 | the found item is not a folder (a file, a link, a socket, a pipe)                    | not claimed                                  |
| 2 | the tool's sign is missing (above)                                                   | not claimed                                  |
| 3 | the folder could not be read in full                                                 | not claimed                                  |
| 4 | a socket, a pipe or a device inside                                                  | left out: a running program may be using it  |
| 5 | not yours, locked by the system, restricted or locked by you, or something inside is | left out: removing it needs an administrator |
| 6 | otherwise                                                                            | **offered**                                  |

A folder that is not claimed is not shown and nothing is said about it. Times are not read: a cache is offered whatever
its age. The `Pods` folders of projects and CocoaPods' spec repositories are never offered. The folder goes to the
Trash; the app never runs the tool's own command, and the tool makes its cache again when it needs it. A cache the tool
holds open when you confirm is skipped.
