# Developer Environment Cleaner

**Since:** 2026-11-15

## Problem

Developer tools are the biggest hidden consumers on a developer's Mac: Xcode, simulators, Docker, package managers,
IDEs. Their data is regenerable, but not all of it is equally cheap to regenerate.

## Direction

One section per tool, each category with its danger level:

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
