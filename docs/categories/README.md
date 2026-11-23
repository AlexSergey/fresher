# Categories

The catalogue of everything the app can name on a disk. Every category the app shows has a file here, and every file
here matches a category of a module. A category that is not documented here is not shipped (rule 3 of
[`../rules.md`](../rules.md)): each module's contract test (`assert_module_contract` of `fresher-testing` with
`Catalogue::Required`) fails for a category without its file or with another danger level in it. The dev-only `demo`
and `trial` modules have no files here. One category is the core's, not a module's: `core.own-file`, the items the
user picks by hand on the Largest screen ([`core-own-file.md`](./core-own-file.md), plan
[035](../plans/035-large-files/readme.md)); two more are the core's, `core.uninstall-keep-data` and
`core.uninstall-everything`, the app an uninstall takes with or without its data, offered only from an app's page,
never on Scan ([`core-uninstall-keep-data.md`](./core-uninstall-keep-data.md),
[`core-uninstall-everything.md`](./core-uninstall-everything.md), plan [036](../plans/036-uninstall/readme.md)). How a module is written:
[`../architecture/modules.md`](../architecture/modules.md).

There is no "Other", "Miscellaneous" or "System data" category. Space no rule recognizes is shown as its real folders.

## Danger levels

The danger level says what the user risks by removing a category. It is shown next to the category, on the
confirmation screen, and the confirmation names the highest level among the selected items.

| Level      | Meaning                                                                                          | Examples                                                                   |
| ---------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| **Safe**   | Rebuilt automatically when needed; removing it costs nothing but a little time.                  | logs, Xcode derived data, a removed app's caches, an IDE's caches, Docker build cache          |
| **Low**    | Rebuilt on demand, but it costs time or bandwidth (a download, a rebuild).                       | an installed app's cache, a browser's cache, package manager caches, a downloaded AI model, `node_modules`, an old project's `node_modules`, unused Docker images, an unfinished download, an unused Rust toolchain, old device support files |
| **Medium** | Can be restored, but not automatically; you may lose settings or need to set something up again. | settings of an older IDE version, an old project's Python environment, old iOS simulators, stopped Docker containers, an unused Node version |
| **High**   | May hold the only copy of your data; removing it can lose work.                                  | Docker volumes, iOS device backups, a removed app's support folder, a big download you have not opened, Xcode archives |

The levels are final for a category in its default form; a rule may raise (never lower) a level for a specific finding,
for example a `node_modules` of a project touched yesterday.

## Format

One file per category: category `<module>.<name>` is documented in `<module>-<name>.md` (`docker.images` in
`docker-images.md`). The contract test reads the danger level from the `**Danger level:**` line.

```markdown
# <Category name, as shown in the app>

**Danger level:** <Safe | Low | Medium | High> · **Rule:** `<rule id>` · **Since:** <version>

**Example:** <one line, at most 80 characters, no `|`: what a user recognizes, shown in the README's table>

## What it is

<One or two plain-English sentences: what these files are and which app makes them.>

## Where it lives

<The paths or patterns, with `~` for the home folder.>

## What happens if you remove it

<The consequence, and whether and how it comes back.>

## How the app decides

<What the rule checks so that it never claims something else; the cases it refuses.>
```

The `**Example:**` line is shown in the README's "What it finds" table (`npm run readme:generate`);
`npm run check:readme` refuses a file without it.

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
| `app-leftovers.caches` | Safe | none (the module's code, `modules/app-leftovers`) | [`app-leftovers-caches.md`](./app-leftovers-caches.md) |
| `app-leftovers.settings` | Medium | none (the module's code, `modules/app-leftovers`) | [`app-leftovers-settings.md`](./app-leftovers-settings.md) |
| `app-leftovers.data` | High | none (the module's code, `modules/app-leftovers`) | [`app-leftovers-data.md`](./app-leftovers-data.md) |
| `app-caches.library-caches` | Low | none (the module's code, `modules/app-caches`) | [`app-caches-library-caches.md`](./app-caches-library-caches.md) |
| `app-caches.browsers` | Low | none (the module's code, `modules/app-caches`, table `apps.toml`) | [`app-caches-browsers.md`](./app-caches-browsers.md) |
| `app-caches.web-engine` | Low | none (the module's code, `modules/app-caches`, table `apps.toml`) | [`app-caches-web-engine.md`](./app-caches-web-engine.md) |
| `downloads.unfinished` | Low | none (the module's code, `modules/downloads`) | [`downloads-unfinished.md`](./downloads-unfinished.md) |
| `downloads.safari-resumable` | Medium | none (the module's code, `modules/downloads`) | [`downloads-safari-resumable.md`](./downloads-safari-resumable.md) |
| `downloads.installers` | Low | none (the module's code, `modules/downloads`) | [`downloads-installers.md`](./downloads-installers.md) |
| `downloads.macos-installers` | Low | none (the module's code, `modules/downloads`) | [`downloads-macos-installers.md`](./downloads-macos-installers.md) |
| `downloads.large-untouched` | High | none (the module's code, `modules/downloads`) | [`downloads-large-untouched.md`](./downloads-large-untouched.md) |
| `stale-projects.dependencies` | Low | none (the module's code, `modules/stale-projects`) | [`stale-projects-dependencies.md`](./stale-projects-dependencies.md) |
| `stale-projects.build-output` | Low | none (the module's code, `modules/stale-projects`) | [`stale-projects-build-output.md`](./stale-projects-build-output.md) |
| `stale-projects.python-environments` | Medium | none (the module's code, `modules/stale-projects`) | [`stale-projects-python-environments.md`](./stale-projects-python-environments.md) |
| `stale-projects.rust-build-cache` | Low | none (the module's code, `modules/stale-projects`) | [`stale-projects-rust-build-cache.md`](./stale-projects-rust-build-cache.md) |
| `local-ai-models.hugging-face-models` | Low | none (the module's code, `modules/local-ai-models`) | [`local-ai-models-hugging-face-models.md`](./local-ai-models-hugging-face-models.md) |
| `local-ai-models.hugging-face-datasets` | Low | none (the module's code, `modules/local-ai-models`) | [`local-ai-models-hugging-face-datasets.md`](./local-ai-models-hugging-face-datasets.md) |
| `local-ai-models.lm-studio` | Medium | none (the module's code, `modules/local-ai-models`) | [`local-ai-models-lm-studio.md`](./local-ai-models-lm-studio.md) |
| `local-ai-models.ollama` | Medium | none (the module's code, `modules/local-ai-models`) | [`local-ai-models-ollama.md`](./local-ai-models-ollama.md) |
| `unused-toolchains.rust` | Low | none (the module's code, `modules/unused-toolchains`) | [`unused-toolchains-rust.md`](./unused-toolchains-rust.md) |
| `unused-toolchains.node` | Medium | none (the module's code, `modules/unused-toolchains`) | [`unused-toolchains-node.md`](./unused-toolchains-node.md) |
| `temp.old-files`                | Safe         | none (the module's code, `modules/temp`)       | [`temp-old-files.md`](./temp-old-files.md)                               |
| `logs.app-logs`                 | Safe         | none (the module's code, `modules/logs`)       | [`logs-app-logs.md`](./logs-app-logs.md)                                 |
| `logs.crash-reports`            | Safe         | none (the module's code, `modules/logs`)       | [`logs-crash-reports.md`](./logs-crash-reports.md)                       |
| `xcode.device-support`          | Low          | none (the module's code, `modules/xcode`)      | [`xcode-device-support.md`](./xcode-device-support.md)                   |
| `xcode.previews`                | Safe         | none (the module's code, `modules/xcode`)      | [`xcode-previews.md`](./xcode-previews.md)                               |
| `xcode.simulator-caches`        | Safe         | none (the module's code, `modules/xcode`)      | [`xcode-simulator-caches.md`](./xcode-simulator-caches.md)               |
| `xcode.archives`                | High         | none (the module's code, `modules/xcode`)      | [`xcode-archives.md`](./xcode-archives.md)                               |
| `core.own-file`                 | High         | none (the core's code, `crates/engine/src/own.rs`; made by no module: the user picks each item on the Largest screen) | [`core-own-file.md`](./core-own-file.md) |
| `core.uninstall-keep-data`      | Medium       | none (the core's code, `crates/engine/src/apps/uninstall.rs`; made by no module: the user chooses it on an app's page) | [`core-uninstall-keep-data.md`](./core-uninstall-keep-data.md) |
| `core.uninstall-everything`     | High         | none (the core's code, `crates/engine/src/apps/uninstall.rs`; made by no module: the user chooses it on an app's page) | [`core-uninstall-everything.md`](./core-uninstall-everything.md) |

The next one (`node_modules`) comes with the plan that implements its rules.

## Described, not cleaned

Docker is not a category: nothing of it is offered, selected, counted or removed by Fresher (plan
[032](../plans/032-docker/readme.md); the user's decision of 2026-11-22). The module `docker` only describes what a
running runtime says it takes, and shows the command that frees each part for the user to run. The levels below colour
the wording of those parts, nothing else; there is no category file and no row in the catalogue.

| Part               | Level  | The command shown                          | Why that level                                         |
| ------------------ | ------ | ------------------------------------------ | ------------------------------------------------------ |
| Build cache        | Safe   | `docker --context <ctx> builder prune --all`  | rebuilt by the next build                           |
| Unused images      | Low    | `docker --context <ctx> image prune --all`    | pulled or built again, at the cost of time          |
| Stopped containers | Medium | `docker --context <ctx> container prune`      | their writable layer is lost                        |
| Unused volumes     | High   | `docker --context <ctx> volume prune`         | a volume may hold the only copy of a database       |

The disk file of the runtime is shown beside them (its largest size, what it takes on the disk, what is used inside),
because freed space inside Docker does not shrink what the Mac loses until the runtime gives it back.
