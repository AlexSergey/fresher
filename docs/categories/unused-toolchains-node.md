# Unused Node versions

**Danger level:** Medium · **Rule:** none (the module's code, `modules/unused-toolchains`) · **Since:** 0.1.0

**Example:** a Node version in `~/.nvm/versions/node` no project uses

## What it is

Versions of Node.js that nvm installed and that nothing on this Mac points at: not nvm's default or another alias, not a project's `.nvmrc`, `.node-version` or `.tool-versions`. Made by nvm, the Node version manager. One finding per version, titled "Node" and the folder's name (for example `v18.17.0`). **Off by default:** the setting "Node versions (nvm)" turns it on.

## Where it lives

`~/.nvm/versions/node/v<x.y.z>`. Only the default place is looked at: the app reads no environment variable, so an `NVM_DIR` elsewhere is not found (its space stays in the storage map).

## What happens if you remove it

The global packages installed with it go too, and an editor set to this exact version stops finding it. It comes back
with one command, `nvm install <x.y.z>` (the finding names it), and a download; global packages you installed with it
must be installed again. The app cannot see whether a version holds global packages, which is why the level is Medium.

## How the app decides

The module `unused-toolchains` (code with a data table, `modules/unused-toolchains/toolchains.toml`). What points at a
version: nvm's `default` alias and every other alias file in `~/.nvm/alias` (including `lts/*`), and every `.nvmrc`,
`.node-version` and `.tool-versions` (its `nodejs` or `node` line) in the scanned folders, except one inside a project's
artifact folder (such as `node_modules` or `target`). A pin of `18` or `18.17` keeps every installed version with
that prefix. For each version folder, the first answer that applies:

| #   | Situation                                                                                     | Result                                      |
| --- | --------------------------------------------------------------------------------------------- | ------------------------------------------- |
| 1   | the found item is not a folder (a link, a file)                                               | not claimed                                 |
| 2   | its name is not `v<major>.<minor>.<patch>`                                                    | not claimed                                 |
| 3   | the setting "Node versions (nvm)" is off (the default)                                        | not claimed                                 |
| 4   | the newest installed version (`nvm use node` means it; a linked folder counts as installed) | not claimed                                 |
| 5   | the folder could not be read in full, or no time of it is known                               | not claimed                                 |
| 6   | a known reference keeps it: the default, an alias or a pin                                    | not claimed (it is in use)                  |
| 7   | a socket, a pipe or a device inside                                                           | left out: a running program may be using it |
| 8   | not yours, locked by the system, restricted or locked by you, or something inside is          | left out: removing it needs an administrator |
| 9   | the walk did not read every folder                                                            | left out: a project there might use it      |
| 10  | the default could not be read                                                                 | left out: every version is kept             |
| 11  | another alias or a pin could not be read                                                      | left out: Fresher cannot tell what is used  |
| 12  | otherwise                                                                                     | **offered**                                 |

Age is not judged: "not used", not "old"; the reason gives the date the folder was last opened or changed. With no
`default` alias nothing is kept by it, and the newest version is kept anyway. The module needs access to Desktop,
Documents and Downloads, where projects live. A scan root narrower than the home folder is not detected: a pin outside
the roots is not seen. A pointer the app cannot see (an editor's Node path, a shell profile's `nvm use 16`) does not keep
a version. The version's folder goes to the Trash; the app never runs `nvm`, which is a shell function. A version a
program holds open when you confirm is skipped.
