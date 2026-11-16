# Ignore on a single item

**Agreed:** 2026-11-16, delegated to the agent by the user (plan 005, agent's call 7) · **Due at:** with the finding
row's actions (a row menu), or when a module needs it

## What is postponed

The **Ignore** button on one finding. The core does all of it already: `ignore` takes a finding (its path goes to
`core.ignore.paths`), the engine leaves it out and the cleaner protects it, and the settings list it with Restore.
Only the button in the row is missing.

## What doing it needs

The finding row is a virtual list item with its own keyboard model (one tab stop, the checkbox is not tabbable). A
second action in the row needs that model to reach it by keyboard, for example a row menu or a key, and the rows'
screenshots and zoom checks updated.
