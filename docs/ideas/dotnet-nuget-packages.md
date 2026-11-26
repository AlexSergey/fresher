# .NET and NuGet packages not used for a long time

**Since:** 2026-11-26 · From the first tester's feedback on 0.1.0

## Problem

.NET development keeps every restored package version in NuGet's global packages folder (`~/.nuget/packages`) and its
HTTP and plugin caches, plus SDKs, workloads and global tools under `~/.dotnet`. They grow with every project and
version and are never trimmed.

## Direction

- A category for NuGet's caches (HTTP cache, plugins cache, temp): regenerable, Low; prefer the tool's own command
  (`dotnet nuget locals <kind> --clear`) when the CLI is installed.
- Package versions in the global packages folder that no project on the Mac restores any more, older than the stale
  projects' days: Low, offered per package version, never the folder whole; a project's `obj/project.assets.json`
  tells which versions it uses.
- .NET SDKs and runtimes nothing points at (`global.json`, the newest per major): with the other toolchains
  ([`unused-toolchain-versions.md`](./unused-toolchain-versions.md)).
- The places must be checked on a real install and pinned in fixtures.
