# A module names its app's data folders for the storage map

**Agreed:** 2026-11-17, by the controller of plan 016 (review of step 03/01) · **Due at:** the next change to the
storage map's app lines, or a module whose app keeps its data under folders not named after its bundle id

## What is postponed

The storage map's by-meaning view counts an app's data only in folders named after the app's bundle id. A JetBrains
IDE keeps its data in folders named after the product and version (`WebStorm2025.3`), so the folders of the version in
use show among the user's folders, not in WebStorm's line, though the `jetbrains` module knows whose they are.

## What doing it needs

- A way for a module to name the folders of an installed app's data that it recognizes and does not offer (a
  left-out with its app, or an attribution beside the findings), carried to the map's app line as data.
- The `jetbrains` module naming the version in use's folders for the newest installed app of the product; the map's
  and the core e2e's tests for that line's data.
