# Left-out items folded at the bottom of a category

**Agreed:** 2026-11-26, by the user, on the 0.1.0 release build · **Due at:** 0.2.0

## What is postponed

On a category's details, the items left out for now are listed with their reason next to what is offered, for example
"389 items left out (25.7 GB): It or its project changed within the days set in Settings, so it is kept. A scan after
those days offers it.", followed by the items. The user read the list as what would be removed (the `node_modules` of
a project changed four days earlier, rightly kept by `stale-projects`). The user's proposal: the left-out lines go to
the bottom of the screen, below everything offered, folded under an accordion that shows only the line with the count
and the size, and opens to the items.

## Why

0.1.0 was being tagged; the module decides right, the screen only reads ambiguously, and the details screen is held by
e2e and accessibility tests that a late change would have to pass again.

## What doing it needs

1. The details screen (`app/ui/src/screens/category/`): the left-out groups after the offered items, folded by
   default, the summary line their toggle; the same for the "kept for good" line.
2. The Scan screen's left-out lines read the same way.
3. The unit, e2e (keyboard: the toggle is reachable and named) and accessibility tests; the gallery story of a folded
   and an opened group.
4. [`../test-cases/`](../test-cases/README.md): a case that a category with many left-out items shows them folded.
