# Xcode build data

**Danger level:** Safe · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

## What it is

What Xcode makes while it builds and indexes a project: compiled files, the code index and build logs, one folder per
project. Made by Xcode, Apple's app for building apps. Xcode's other data (device support, previews, the simulators'
caches and archives) is in the four categories of the `xcode` module
([`xcode-device-support.md`](./xcode-device-support.md) and its three siblings).

## Where it lives

`~/Library/Developer/Xcode/DerivedData/<project>-<28 letters>`: one finding per project folder. Only the tool's default
place is looked at: the app reads no environment variable and no config file. Derived data that Xcode's settings put
elsewhere is not found.

The tool's sign: the folder's name is a project's name, a dash and 28 lowercase letters, and the folder holds
`info.plist`. Without it the folder is not claimed.

## What happens if you remove it

It goes to the Trash. Your projects and their code stay; Xcode builds and indexes them again when you open them. Yes.
Xcode makes it again on the next build of the project, with no download; that build is slower.

## How the app decides

The module `dev-caches` (code with a data table, `modules/dev-caches/tools.toml`). For the folder, the first answer that
applies:

| # | Situation                                                                            | Result                                       |
| - | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| 1 | the found item is not a folder (a file, a link, a socket, a pipe)                    | not claimed                                  |
| 2 | the folder's name does not have the project's shape                                  | not claimed                                  |
| 3 | the tool's sign is missing (above)                                                   | not claimed                                  |
| 4 | the folder could not be read in full                                                 | not claimed                                  |
| 5 | a socket, a pipe or a device inside                                                  | left out: a running program may be using it  |
| 6 | not yours, locked by the system, restricted or locked by you, or something inside is | left out: removing it needs an administrator |
| 7 | the open apps cannot be listed                                                       | left out: Fresher cannot list open apps      |
| 8 | Xcode is open                                                                        | left out: its app is open                    |
| 9 | otherwise                                                                            | **offered**                                  |

A folder that is not claimed is not shown and nothing is said about it. Times are not read: a cache is offered whatever
its age. `ModuleCache.noindex`, Archives, simulators and device support are never offered. The folder goes to the Trash;
the app never runs the tool's own command, and the tool makes its cache again when it needs it. A cache the tool holds
open when you confirm is skipped.
