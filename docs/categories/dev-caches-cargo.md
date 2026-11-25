# Cargo's download cache

**Danger level:** Low · **Rule:** none (the module's code, `modules/dev-caches`) · **Since:** 0.1.0

**Example:** `~/.cargo/registry/cache`

## What it is

Rust libraries Cargo downloaded, so the next build needs no download. Made by Cargo, Rust's build tool.

## Where it lives

`~/.cargo/registry/cache` (the packed crates) and `~/.cargo/registry/src` (the unpacked ones): one finding each. Only
the tool's default place is looked at: the app reads no environment variable and no config file. A Cargo home that
`CARGO_HOME` moves elsewhere is not found.

The tool's sign: the folder above them, `~/.cargo/registry`, holds a valid `CACHEDIR.TAG` (its content is checked, not
only its name). Without it the folder is not claimed.

## What happens if you remove it

Nothing is lost. Your projects and the programs you installed with Cargo stay. Yes, Cargo downloads each one again when
a project builds with it.

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
its age. `~/.cargo/registry/index`, `~/.cargo/git`, `~/.cargo/bin` and Cargo's settings are never offered. The folder
goes to the Trash; the app never runs the tool's own command, and the tool makes its cache again when it needs it. A
cache the tool holds open when you confirm is skipped.
