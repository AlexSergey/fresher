# A shared day setting in the core

**Agreed:** 2026-11-18 (plan 019 pre-flight) · **Due at:** the third module with an "unchanged for N days" setting

## What is postponed

`jetbrains` and `app-leftovers` each carry the same setting: a number of days from 7 to 3650, 30 by default, with its
descriptor function and its reader (about 30 lines, copied). A module may not depend on another module, so the copy is
accepted for two modules.

## What doing it needs

One function pair in `fresher-core` (the descriptor of the setting named `<module>.<name>` and the reader that clamps
and defaults), used by every module that has the setting; the modules' copies and their tests are removed in the same
change.
