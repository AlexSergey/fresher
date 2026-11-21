# More places of installed apps' caches

**Agreed:** 2026-11-21, by the agent under the user's choice of 2026-11-21 (plan 030, agent's call 1; to check) ·
**Due at:** after plan 030's run on the developer's Mac, one part per plan

## What is postponed

Plan [030](../plans/030-installed-app-caches/readme.md) offers `~/Library/Caches/<bundle id>` of installed apps. The rest of the idea
[Caches of installed apps](../ideas/installed-app-caches.md) is postponed, each part with what doing it needs:

| Part                                                                                                  | Why not in plan 030                                                                                     | What doing it needs                                                                                                   |
| ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Web-engine caches inside `~/Library/Application Support/<name>/` (`Cache`, `Code Cache`, `GPUCache`, `Service Worker/CacheStorage`) of chat, music and editor apps | the folder is named after the app's name, and an app is identified by its bundle id only, never by a loose name | a table (rules as data) tying each folder name to a bundle id, checked app by app on a real Mac; the cache folders as rows with the layout as their sign; the app's running check by that id |
| Browsers' caches (Chrome, Firefox, Edge, Brave, Vivaldi, Arc)                                          | named after the browser, with profiles inside (`Caches/Google/Chrome/<profile>/Cache`, `Caches/Firefox/Profiles/<profile>/cache2`) | the same table with each browser's profile layout; the cache only, never history, cookies, passwords or profiles    |
| Game launchers' download and web caches (Steam, Epic Games, Battle.net)                               | inside their support folders, under their names                                                         | the same table; a launcher's download cache may hold an update in progress (kept while it runs)                       |
| The Metal shader cache                                                                                 | under the user's cache folder in `/private/var/folders/<xx>/<hash>/C/`, refused by the protection list (only `T` is carved out), and Apple's | a deletion-safety decision for that one folder, written in [`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md); offered without being selected (rebuilding costs time on each game's next start) |
| Sandboxed apps' caches (`~/Library/Containers/<id>/Data/Library/Caches`)                               | containers are not read                                                                                 | [Containers of an app's extensions](./app-leftovers-containers.md) first                                              |
| Apple's own apps' caches (`Caches/com.apple.*`, Safari's)                                              | many are written by the system's services, not an app the user opens; some need Full Disk Access        | a list of Apple apps whose cache folder is the app's alone, each checked by hand                                      |

Plan 030's module (`app-caches`, a `places.toml` of one row and an `exclude.toml`) is where these rows land; each
needs its own case file cases and category text. The module finds the owner of a folder by the folder's name as a
bundle id (`BundleId::names` over the installed apps and their helpers), so a row for a place named after the app's
name needs the table above to give the owner's bundle id, and a decision for it beside the owner rule, not the one-row
table alone. The `exclude.toml` and the checks of `decide.rs` (rows 5 to 10: read in full, not the user's, nothing
open) apply to them unchanged.
