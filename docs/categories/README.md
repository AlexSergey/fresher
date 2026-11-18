# Categories

The catalogue of everything the app can name on a disk. Every category the app shows has a file here, and every file
here matches a category of a module. A category that is not documented here is not shipped (rule 3 of
[`../rules.md`](../rules.md)): each module's contract test (`assert_module_contract` of `fresher-testing` with
`Catalogue::Required`) fails for a category without its file or with another danger level in it. The dev-only `demo`
and `trial` modules have no files here. How a module is written:
[`../architecture/modules.md`](../architecture/modules.md).

There is no "Other", "Miscellaneous" or "System data" category. Space no rule recognizes is shown as its real folders.

## Danger levels

The danger level says what the user risks by removing a category. It is shown next to the category, on the
confirmation screen, and the confirmation names the highest level among the selected items.

| Level      | Meaning                                                                                          | Examples                                                                   |
| ---------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| **Safe**   | Rebuilt automatically when needed; removing it costs nothing but a little time.                  | app caches, logs, Xcode derived data                                       |
| **Low**    | Rebuilt on demand, but it costs time or bandwidth (a download, a rebuild).                       | package manager caches, `node_modules`, Docker build cache                 |
| **Medium** | Can be restored, but not automatically; you may lose settings or need to set something up again. | settings of an older IDE version, old iOS simulators, unused Docker images |
| **High**   | May hold the only copy of your data; removing it can lose work.                                  | Docker volumes, stopped containers, iOS device backups                     |

The levels are final for a category in its default form; a rule may raise (never lower) a level for a specific finding,
for example a `node_modules` of a project touched yesterday.

## Format

One file per category: category `<module>.<name>` is documented in `<module>-<name>.md` (`docker.images` in
`docker-images.md`). The contract test reads the danger level from the `**Danger level:**` line.

```markdown
# <Category name, as shown in the app>

**Danger level:** <Safe | Low | Medium | High> · **Rule:** `<rule id>` · **Since:** <version>

## What it is

<One or two plain-English sentences: what these files are and which app makes them.>

## Where it lives

<The paths or patterns, with `~` for the home folder.>

## What happens if you remove it

<The consequence, and whether and how it comes back.>

## How the app decides

<What the rule checks so that it never claims something else; the cases it refuses.>
```

## Catalogue

| Category                        | Danger level | Rule                                           | File                                                                     |
| ------------------------------- | ------------ | ---------------------------------------------- | ------------------------------------------------------------------------ |
| `dev-caches.npm`                | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-npm.md`](./dev-caches-npm.md)                               |
| `dev-caches.yarn`               | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-yarn.md`](./dev-caches-yarn.md)                             |
| `dev-caches.homebrew`           | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-homebrew.md`](./dev-caches-homebrew.md)                     |
| `dev-caches.cargo`              | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-cargo.md`](./dev-caches-cargo.md)                           |
| `dev-caches.cocoapods`          | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-cocoapods.md`](./dev-caches-cocoapods.md)                   |
| `dev-caches.gradle`             | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-gradle.md`](./dev-caches-gradle.md)                         |
| `dev-caches.pip`                | Low          | none (the module's code, `modules/dev-caches`) | [`dev-caches-pip.md`](./dev-caches-pip.md)                               |
| `dev-caches.go-build`           | Safe         | none (the module's code, `modules/dev-caches`) | [`dev-caches-go-build.md`](./dev-caches-go-build.md)                     |
| `dev-caches.xcode-derived-data` | Safe         | none (the module's code, `modules/dev-caches`) | [`dev-caches-xcode-derived-data.md`](./dev-caches-xcode-derived-data.md) |
| `jetbrains.caches`              | Safe         | none (the module's code, `modules/jetbrains`)  | [`jetbrains-caches.md`](./jetbrains-caches.md)                           |
| `jetbrains.logs`                | Safe         | none (the module's code, `modules/jetbrains`)  | [`jetbrains-logs.md`](./jetbrains-logs.md)                               |
| `jetbrains.settings`            | Medium       | none (the module's code, `modules/jetbrains`)  | [`jetbrains-settings.md`](./jetbrains-settings.md)                       |
| `jetbrains.uninstalled`         | Medium       | none (the module's code, `modules/jetbrains`)  | [`jetbrains-uninstalled.md`](./jetbrains-uninstalled.md)                 |
| `temp.old-files`                | Safe         | none (the module's code, `modules/temp`)       | [`temp-old-files.md`](./temp-old-files.md)                               |

The next ones (`node_modules`, Docker) come with the plans that implement their rules.
