# More places of installed apps' caches

**Agreed:** 2026-11-21, by the agent under the user's choice of 2026-11-21 (plan 030, agent's call 1; to check) ·
**Due at:** after plan 030's run on the developer's Mac, one part per plan

## What is postponed

Plan [030](../plans/030-installed-app-caches/readme.md) offers `~/Library/Caches/<bundle id>` of installed apps. The rest of the idea
[Caches of installed apps](../ideas/installed-app-caches.md) is postponed, each part with what doing it needs:

| Part                                                                                                  | Why not in plan 030                                                                                     | What doing it needs                                                                                                   |
| ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| More rows of the table for web-engine apps (`Cache`, `Code Cache`, `GPUCache`) in `~/Library/Application Support/<name>/`: Visual Studio Code, Slack, Discord, Cursor and others; `Service Worker/CacheStorage` and `CachedData` | plan [034](../plans/034-browser-caches/readme.md) built the table (`apps.toml`) with one row, Redis Insight; every other app waits for its bundle id and folders checked on a real install, and `Service Worker/CacheStorage` and `CachedData` for a name, a path or a meaning not yet checked | for each app: the bundle id from `Info.plist`, the folders with `ls`, a row in `apps.toml` and a case in the case files (listed in [the review list](./user-review-2026-11-22.md)) |
| More browsers' caches (Edge, Brave, Vivaldi, Arc) and the browsers' support-folder graphics caches | plan [034](../plans/034-browser-caches/readme.md) built the table with Chrome, Chromium and Firefox; the others wait for a check on a real install, and the support-folder caches are small and protected without Full Disk Access | a row each, checked as above; for the support-folder caches, Full Disk Access to verify the folders on a real Mac |
| Game launchers' download and web caches (Steam, Epic Games, Battle.net)                               | inside their support folders, under their names                                                         | the same table; a launcher's download cache may hold an update in progress (kept while it runs)                       |
| The Metal shader cache                                                                                 | under the user's cache folder in `/private/var/folders/<xx>/<hash>/C/`, refused by the protection list (only `T` is carved out), and Apple's | a deletion-safety decision for that one folder, written in [`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md); offered without being selected (rebuilding costs time on each game's next start) |
| Sandboxed apps' caches (`~/Library/Containers/<id>/Data/Library/Caches`)                               | containers are not read                                                                                 | [Containers of an app's extensions](./app-leftovers-containers.md) first                                              |
| Apple's own apps' caches (`Caches/com.apple.*`, Safari's)                                              | many are written by the system's services, not an app the user opens; some need Full Disk Access        | a list of Apple apps whose cache folder is the app's alone, each checked by hand                                      |

Plan 030's module (`app-caches`, a `places.toml` of one row and an `exclude.toml`) is where the game launchers and the
other rows land; each needs its own case file cases and category text. A place named after an app's name (not its
bundle id) is a row of plan 034's `apps.toml`, which gives the owner's bundle id, and the same `exclude.toml` and the
checks of `decide.rs` (rows 5 to 10: read in full, not the user's, nothing open) apply to it unchanged.
