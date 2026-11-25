# npm's download cache

**Danger level:** Low · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

**Example:** `~/.npm/_cacache`

## What it is

Copies of the packages npm downloaded, so the next install needs no download. Made by npm, which comes with Node.js.

## Where it lives

`~/.npm/_cacache`, one folder. Only the tool's default place is looked at: the app reads no environment variable and no
config file. A cache that `npm_config_cache` or npm's config moves elsewhere is not found.

The tool's sign: the folder holds both `index-v5` and `content-v2`, the two parts npm writes. Without it the folder is
not claimed.

## What happens if you remove it

Nothing is lost. Your projects and the packages in them keep working. Yes, npm downloads each package again when a
project needs it.

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
its age. The rest of `~/.npm` (`_npx`, `_logs`, its settings) is never offered. The folder goes to the Trash; the app
never runs the tool's own command, and the tool makes its cache again when it needs it. A cache the tool holds open when
you confirm is skipped.
