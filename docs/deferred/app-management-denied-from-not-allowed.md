# App Management learned only from a refusal that means it

**Agreed:** 2026-11-22, by the user's delegation of plan [036](../plans/036-uninstall/readme.md) (the agent's ruling
while building; to check) · **Due at:** after plan 036's walk on the developer's Mac, or the next change to the cleaner

## What is postponed

The engine learns App Management as `Denied` when the Trash refuses the bundle of an uninstall, whatever the reason:
the cleaner maps every Trash refusal (`NotAllowed`, `Occupied`, `NoParent`, `Refused`) to one failure, `TrashRefused`. An
`Occupied` or `NoParent` refusal then reads as "macOS did not let Fresher move an app". The row-6 checks (the bundle is
the user's, its folder writable) make that rare, not impossible.

## What doing it needs

A core `Failure` variant that tells `NotAllowed` from the others and the cleaner's mapping of it (the cleaner is frozen
in plan 036), then `Management::learn` reading only that variant. The walk first: whether macOS prompts, refuses
silently or shows its own notice is not verified.
