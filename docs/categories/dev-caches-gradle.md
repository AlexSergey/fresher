# Gradle's caches

**Danger level:** Low · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

**Example:** `~/.gradle/caches`

## What it is

Libraries Gradle downloaded and files it prepared for builds of Java, Kotlin and Android projects. Made by Gradle, the
build tool of Java, Kotlin and Android projects, and the IDEs that run it.

## Where it lives

`~/.gradle/caches`, one folder. Only the tool's default place is looked at: the app reads no environment variable and no
config file. A Gradle home that `GRADLE_USER_HOME` moves elsewhere is not found.

The tool's sign: the folder holds a valid `CACHEDIR.TAG` (its content is checked, not only its name). Without it the
folder is not claimed.

## What happens if you remove it

They go to the Trash. Your projects and Gradle's settings stay. Yes. The next build downloads and prepares them again,
so it takes longer and needs the network.

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
its age. The rest of `~/.gradle` (the wrapper's downloads, the daemon's files, the settings) is never offered. The
folder goes to the Trash; the app never runs the tool's own command, and the tool makes its cache again when it needs
it. A cache the tool holds open when you confirm is skipped.
