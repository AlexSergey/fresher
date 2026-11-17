# A removed app's name in the storage map

**Agreed:** 2026-11-17, by the controller of plan 016 (review of step 02/04) · **Due at:** the uninstalled app
leftovers module, or the next change to the storage map's app lines

## What is postponed

A finding of the `jetbrains` module's category `jetbrains.uninstalled` belongs to an app that is no longer installed:
`NewFinding::for_app` gives it the product's release bundle id, and the storage map, which knows only the installed
apps' names, shows its line under the id itself (`com.jetbrains.datagrip`) instead of the product's display name
(DataGrip).

## What doing it needs

- A finding's app with an optional display name (`for_app_named(id, name)` or a name beside the id), carried to the
  map's `AppRef` when no installed app has the id.
- The `jetbrains` module passing the product's name from `products.toml`; the map's tests for a named removed app.
