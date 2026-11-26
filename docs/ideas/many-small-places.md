# Many small places that add up

**Since:** 2026-11-26 · From the first tester's feedback on 0.1.0

## Problem

Beyond the big categories, a developer's Mac has many small caches and leftovers, each a few hundred megabytes, that
add up to gigabytes: package managers' and tools' caches Fresher does not know yet. The tester expects them found.

## Direction

- Keep a list of candidate places, each checked on a real install before it becomes a row of a module's table, as
  the installed apps' caches do (plan 034's table): one row = one tool, its place, its kind, its danger level.
- Each row goes through the same bar as any category: a proven owner, regenerable, no "Other".
- Feedback from testers adds rows; a row nobody can check stays here.
