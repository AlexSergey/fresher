# Homebrew's downloads

**Danger level:** Low · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

**Example:** `~/Library/Caches/Homebrew/downloads`

## What it is

Packages and apps Homebrew downloaded to install them; once installed, they are not needed. Made by Homebrew, a tool
that installs apps and command-line tools.

## Where it lives

`~/Library/Caches/Homebrew/downloads`, one folder. Only the tool's default place is looked at: the app reads no
environment variable and no config file. A cache that `HOMEBREW_CACHE` moves elsewhere is not found.

The tool's sign: the folder above it, `~/Library/Caches/Homebrew`, holds `api`, which Homebrew writes there. Without it
the folder is not claimed.

## What happens if you remove it

Nothing is lost. Everything Homebrew installed keeps working. Only when Homebrew installs or upgrades something; it also
clears old downloads on its own.

## How the app decides

The module `dev-caches` (code with a data table, `modules/dev-caches/tools.toml`). For the folder, the first answer that
applies:

| # | Situation                                                                            | Result                                       |
| - | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| 1 | the found item is not a folder (a file, a link, a socket, a pipe)                    | not claimed                                  |
| 2 | the tool's sign is missing (above)                                                   | not claimed                                  |
| 3 | the folder could not be read in full                                                 | not claimed                                  |
| 4 | a socket, a pipe or a device inside                                                  | kept for good: a running program may be using it  |
| 5 | not yours, locked by the system, restricted or locked by you, or something inside is | kept for good: removing it needs an administrator |
| 6 | otherwise                                                                            | **offered**                                  |

A folder that is not claimed is not shown and nothing is said about it. Times are not read: a cache is offered whatever
its age. The rest of `~/Library/Caches/Homebrew`, and everything Homebrew installed, are never offered. The folder goes
to the Trash; the app never runs the tool's own command, and the tool makes its cache again when it needs it. A cache
the tool holds open when you confirm is skipped.
