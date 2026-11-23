# More kinds of project artifacts

**Since:** 2026-11-23

## Problem

The module `stale-projects` (plan [021](../plans/021-stale-projects/readme.md)) knows ten artifact folders today
(`modules/stale-projects/artifacts.toml`): `node_modules`, `target` (Rust, Maven), `build` and `.gradle` (Gradle),
`.build` (Swift packages), `.dart_tool` and `build` (Dart), `.venv` and `venv` (Python). A developer's disk also fills
with tool caches and the build output of ecosystems the table does not know, so a stale project of those kinds is
offered nothing.

The module still recognises a folder only by its name and the project's marker file beside it.

## Direction

1. **A marker inside the folder, not only beside it.** A table row may also require a file inside the artifact folder
   before it is claimed:
   - `CACHEDIR.TAG` (the cache directory tagging convention: a tool declares the folder a cache that can be deleted);
   - `pyvenv.cfg` for a Python environment, which is surer than the names `.venv` and `venv`;
   - `project.assets.json` for a .NET `obj`.

   This keeps the rule "a module that cannot tell what a path is does not claim it".
2. **Python caches** (Safe or Low): `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox`, `.nox`.
3. **Caches of Node tools**: `.turbo`, `.parcel-cache`, `.angular/cache`, `.nx/cache`. These are caches, not
   framework output. The user's decision of 2026-11-19 stands: `dist`, `.next` and other framework output are never
   offered.
4. **Other ecosystems**, each only beside its project's marker file:
   - .NET: `obj`;
   - PHP: `vendor` (beside `composer.json`);
   - Elixir: `_build`, `deps` (beside `mix.exs`);
   - Haskell: `dist-newstyle` (beside a `*.cabal` or `cabal.project`), `.stack-work`;
   - CMake: `cmake-build-*` (containing `CMakeCache.txt`);
   - Zig: `.zig-cache`, `zig-out` (beside `build.zig`);
   - Android: `.cxx` (beside `build.gradle` or `build.gradle.kts`);
   - Terraform: `.terraform` (beside `.terraform.lock.hcl`). Medium, because the providers are downloaded again.
5. **Game engines**: Unity `Library` (beside `Assets` and `ProjectSettings`), Unreal `Intermediate` and
   `DerivedDataCache` (beside `Config` and `Content`). These are often the biggest folders of all. They are Medium,
   because importing the assets again takes hours. Godot's `.godot` is left out: it also holds the editor's state and
   export credentials.

Not taken, and why:

- **The user's own rules** ("delete folders named X"). This conflicts with "a module that cannot tell what a path is
  does not claim it". If it is ever wanted, it is a list of the user's folders to keep, not a list of folders to
  delete.
- **Machine-readable output, a programmatic API, search by pattern.** These fit a command-line tool, not this app.

Open questions:

- Does each new kind get its own category, or join `build-output` / `dependencies` with the kind named in the finding?
- Do the Python caches go by the project's staleness, or by their own age, like the Rust build cache does?
- Which of the markers and folder layouts can be checked on this Mac, and which stay facts for the user to check?
