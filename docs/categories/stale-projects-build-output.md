# Builds of old projects

**Danger level:** Low · **Rule:** none (the module's code, `modules/stale-projects`) · **Since:** 0.1.0

**Example:** `target` or `build` of a project you have not changed in a year

## What it is

What a project's build tool made and can make again, in projects you have not changed for a long time. Made by the project's build tool, when you built it.

## Where it lives

Beside a project's marker file: `target` (Rust, Maven), `build` and `.gradle` (Gradle), `.build` (Swift packages), `.dart_tool` and `build` (Dart). A Go project has none.

## What happens if you remove it

Your code stays; the next build makes them again, which takes a while. They come back with the next build.

## How the app decides

The module `stale-projects` (code with a data table, `modules/stale-projects/artifacts.toml`). It never runs git, cargo, npm or any tool and never follows a link. Projects in `~/Library`, the apps folders, an app bundle, a hidden folder of the home (`~/.cargo`, `~/.config`) and the home itself are not looked at. A project inside another (a monorepo) is decided with its outermost project, as one group named by the outermost: the group's last activity is the newest of its projects' own, so a commit in a nested repository keeps the whole group, and a project of it that could not be dated keeps the group from being offered. The last activity is the newest change time of the project's folders (its artifact folders and `.git` left out) and of its git branches' folder; it does not see a file edited in place, which changes no folder's time, and a project whose artifact was removed by hand looks active, because removing it changed the project's folder. A project that could not be read in full is left out, never offered. A project that changed within the days of the setting `Offer projects unchanged for` (30 to 3650, 365 by default) is left out as recent; a project older than that is stale and its artifacts are **offered**, except one that itself changed within those days (a package installed into it last week), which is left out as recent, and one that is itself a project's folder, which is never claimed. The folders go to the Trash. A Rust `target` that is left out while caches inside it are offered is shown with its size less theirs.
