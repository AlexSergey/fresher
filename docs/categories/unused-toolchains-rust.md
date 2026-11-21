# Unused Rust toolchains

**Danger level:** Low · **Rule:** none (the module's code, `modules/unused-toolchains`) · **Since:** 0.1.0

## What it is

Versions of the Rust compiler that rustup installed and that nothing on this Mac points at: not rustup's default, not an override, not a project's toolchain file. Made by rustup, the installer of Rust. One finding per toolchain, titled "Rust" and the folder's name (for example `1.75.0-aarch64-apple-darwin`).

## Where it lives

`~/.rustup/toolchains/<name>`. Only the default place is looked at: the app reads no environment variable, so a `RUSTUP_HOME` elsewhere is not found (its space stays in the storage map).

## What happens if you remove it

The toolchain's folder goes to the Trash. A project, a script or an editor that asks for this exact version gets an error until it is installed again. It comes back with one command, `rustup toolchain install <name>` (the finding names it), and a download of a few hundred megabytes. Components and targets you added to it must be added again. Nothing of yours lives in a toolchain, which is why the level is Low.

## How the app decides

The module `unused-toolchains` (code with a data table, `modules/unused-toolchains/toolchains.toml`). What points at a
version: rustup's default and its directory overrides (both in `~/.rustup/settings.toml`), and every `rust-toolchain.toml`,
`rust-toolchain` and `.tool-versions` (its `rust` line) in the scanned folders, except one inside a project's
artifact folder (such as `node_modules` or `target`). For each toolchain folder, the first answer that applies:

| #   | Situation                                                                                     | Result                                      |
| --- | --------------------------------------------------------------------------------------------- | ------------------------------------------- |
| 1   | the found item is not a folder (a linked toolchain, a file)                                   | not claimed                                 |
| 2   | its name is not `<channel>[-<date>]-<triple>` (a custom name)                                 | not claimed                                 |
| 3   | a moving channel (`stable`, `beta` or `nightly` without a date)                               | not claimed                                 |
| 4   | the folder could not be read in full, or no time of it is known                               | not claimed                                 |
| 5   | a known reference keeps it: the default, an override or a pin                                 | not claimed (it is in use)                  |
| 6   | a socket, a pipe or a device inside                                                           | left out: a running program may be using it |
| 7   | not yours, locked by the system, restricted or locked by you, or something inside is          | left out: removing it needs an administrator |
| 8   | the walk did not read every folder                                                            | left out: a project there might use it      |
| 9   | the default could not be read                                                                 | left out: every version is kept             |
| 10  | another pointer or a pin could not be read                                                    | left out: Fresher cannot tell what is used  |
| 11  | otherwise                                                                                     | **offered**                                 |

Age is not judged: "not used", not "old"; the reason gives the date the folder was last opened or changed. The
module needs access to Desktop, Documents and Downloads, where projects live, so that no pin file is missed for want of
access. A scan root narrower than the home folder is not detected: a pin outside the roots is not seen. A pointer the
app cannot see (an editor's setting, `cargo +1.75.0` in a script, `RUSTUP_TOOLCHAIN`) does not keep a toolchain; the
Trash and the install command bring it back. The toolchain's folder goes to the Trash; the app never runs
`rustup`. A toolchain a program holds open when you confirm is skipped.
