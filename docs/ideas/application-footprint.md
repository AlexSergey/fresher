# Application Footprint

**Since:** 2026-11-15

## Problem

Finder shows an app as 500 MB while its data elsewhere takes 20 GB. The real cost of an app is invisible.

## Direction

- For every installed app: the bundle size plus everything it owns outside `/Applications` (support files, caches,
  containers, group containers, logs), as one number with a breakdown.
- Sorted by total footprint, with the last time the app was used — "you have not opened this in 8 months and it takes
  14 GB" is the most useful sentence the app can say.
- Feeds watch mode: an app whose footprint grows between scans is named.
- Actions link to [`smart-uninstaller.md`](./smart-uninstaller.md) and to the cleanable categories of that app.

Open questions:

- Last-used date: Spotlight metadata vs LaunchServices — which is reliable without extra permissions?

**Partly taken by [plan 012](../plans/012-storage-map/readme.md):** the by-meaning view groups each installed app's bundle, data folders and findings into one row with one size. **Still an idea:** the last-used date and the growth between scans.

**Partly taken by [plan 016](../plans/016-jetbrains/readme.md):** a JetBrains IDE's older versions' data is counted in its line of the storage map. **Still an idea:** the last-used date and the growth between scans (see plan 012's note above).
