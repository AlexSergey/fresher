# The core's place queries without a UI

**Agreed:** 2026-11-20, by the user's approval of plan [027](../plans/027-scan-screen/readme.md)'s draft (decision 6) ·
**Due at:** the next plan that shows the disk by folder or by app

## What is postponed

`get_storage_place` and `get_storage_roots` (the view by place) have no UI since plan 027. `get_storage_meaning` is
used only for the ring's split by level; its apps, places and the rest are unused. The engine and the core keep all of
them, tested.

## What doing it needs

Decide whether a folder view returns (in a category's details, or as its own screen) or the queries are removed with
their tests, bindings and the e2e mock's answers. If it returns, two former deferred items are folded here:

- a way up from a folder opened from the by-meaning view: the way from the volume's root kept in the navigation, or
  the folder's ancestors asked of the core;
- a root's explained size in the roots list (a core change: the roots would carry each root's `explained` sizes).
