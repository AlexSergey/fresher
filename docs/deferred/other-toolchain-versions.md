# Other toolchains' unused versions

**Agreed:** 2026-11-21, by the agent under the user's delegation of 2026-11-20 (plan 024, agent's call 1; to check) ·
**Due at:** after plan 024's run on the developer's Mac, one manager per plan, each when the core has what it needs

## What is postponed

Plan [024](../plans/024-unused-toolchains/readme.md) offers the rustup toolchains and the nvm Node versions nothing on
the Mac points at. The other managers of the idea [Unused toolchain versions](../ideas/unused-toolchain-versions.md)
are not offered: their space is shown in the storage map as their real folders. Each needs something plan 024 did not
build.

| Manager                 | Where versions live                                            | What marks a version as used                         | What is missing                                                                                                                                  |
| ----------------------- | -------------------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| pyenv                   | `~/.pyenv/versions/<version>`                                  | `~/.pyenv/version`, `.python-version`, **every virtualenv built on it** (`home =` in `pyvenv.cfg`) | virtualenvs live outside projects (pipx's, poetry's, editors' own folders); a base removed under a missed one breaks it with a confusing error. Needs every `pyvenv.cfg` the walk can see (a marker of its own) and a rule for the places it cannot |
| SDKMAN                  | `~/.sdkman/candidates/<tool>/<version>`                        | `candidates/<tool>/current` (a link), `.sdkmanrc`    | the walk keeps no link's target: the default cannot be read. Needs the walk (or `small_file`'s sibling) to report a link's target, never following it |
| fnm                     | `~/.local/share/fnm/node-versions/<version>`                   | `aliases/default` (a link), `.nvmrc`, `.node-version` | the same: its aliases are links                                                                                                                  |
| volta                   | `~/.volta/tools/image/node/<version>`                          | `tools/user/platform.json`, `volta` in `package.json` | its pins live in `package.json`, read in every project and never in thousands of dependencies: a marker read limited to projects' roots           |
| JDKs                    | `~/Library/Java/JavaVirtualMachines`, `~/.gradle/jdks`, SDKMAN | `.java-version`, `.sdkmanrc`, `JAVA_HOME`, a Gradle toolchain block, an IDE's project JDK | Gradle's toolchain spec is code in a build file; an IDE's choice lives in its own settings; `/Library/Java/JavaVirtualMachines` needs administrator rights (never, rule of no elevation) |
| Android SDK             | `~/Library/Android/sdk/{system-images,ndk,build-tools}`        | an AVD's `config.ini`, a project's `ndkVersion`, the build tools a build names | an AVD holds the user's emulator data (apps, accounts): a category of its own, High, perhaps with the simulators; `ndkVersion` is in build code |
| asdf, mise (installs)   | `~/.asdf/installs/<tool>/<version>`, `~/.local/share/mise/installs/<tool>/<version>` | `.tool-versions`, `mise.toml`, their global files | many tools under one manager; each tool's version shape; `mise.toml`'s `[tools]` table. `.tool-versions` is already read by plan 024 for rustup and nvm |

## What doing it needs

- Plan 024's `small_file` reader is enough for pyenv's, volta's and asdf's own files; SDKMAN and fnm need a link's
  target reported (`readlink`, never followed) by the walk or by a reader of the same rules.
- pyenv: a marker interest on `pyvenv.cfg`, every one read, and a rule for virtualenvs outside the scanned folders
  (keep every version when one cannot be seen?); Medium, since a broken virtualenv is not obvious to fix.
- Each manager: a row of `modules/unused-toolchains/toolchains.toml` (or its own module), its parsers, its category
  doc, fixtures and a case file, as plan 024 did; the user's rule for Node (off by default) asked again for each.
