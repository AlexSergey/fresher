# Developer Environment Cleaner

**Since:** 2026-11-15

## Problem

Developer tools are the biggest hidden consumers on a developer's Mac: Xcode, simulators, Docker, package managers,
IDEs. Their data is regenerable, but not all of it is equally cheap to regenerate.

## Direction

One section per tool, all in the group Development ([plan 015](../plans/015-module-groups/readme.md)), each
category with its danger level:

- **Xcode**: DerivedData (**Safe**), Archives (**High** — they hold shipped builds' symbols), device support files of
  old iOS versions (**Low**), old simulators and their runtimes (**Medium**, through `xcrun simctl`), previews cache.
- **Docker**: build cache (**Low**), dangling images (**Low**), unused images (**Medium**), stopped containers
  (**High**), volumes (**High** — may hold the only copy of a database). Always through the Docker CLI.
- **Package managers**: Homebrew (`brew cleanup`), npm, Yarn, pnpm, Cargo registry and git caches, CocoaPods, Gradle,
  Maven, pip, Go modules — **Low**.
- **IDEs**: stale versions of JetBrains IDE data (the case that started the project), VS Code extension caches.
- Project artifacts (`node_modules`, `target/`) are covered by [`stale-project-cleanup.md`](./stale-project-cleanup.md).
- Where a tool has its own cleanup command, prefer it over deleting files.

Open questions:

- Detect tools that are installed but not used for months and point at their whole footprint?

**Partly taken by [plan 016](../plans/016-jetbrains/readme.md):** the IDEs line for JetBrains: older versions' settings, caches and logs, and the data of an IDE that is no longer installed, in the module `jetbrains`. **Still an idea:** VS Code extension caches and every other tool above.

**Partly taken by [plan 018](../plans/018-developer-caches/readme.md) (built):** the package managers' download caches (npm, Yarn, Homebrew, Cargo's registry, CocoaPods, Gradle, pip), Go's build cache and Xcode's DerivedData, moved to the Trash rather than through the tools' own commands, in the module `dev-caches`. **Still an idea:** Docker, simulators and runtimes, device support, Archives, previews, VS Code, pnpm's store, Maven's repository, Cargo's git caches and index, derived data's module cache.

**Drafted in plans [031](../plans/031-xcode/readme.md) and [032](../plans/032-docker/readme.md)** (2026-11-22): Xcode's archives, device support, previews, simulator caches and unavailable simulator devices; Docker's images, containers, volumes and build cache through its command line. **Still an idea:** simulator runtimes, VS Code, pnpm's store, Maven's repository, Cargo's git caches and index, derived data's module cache.
