# Ideas

The user's ideas that are not planned yet: no design, no date, often only a sentence or two. Unlike
[`../deferred/`](../deferred/README.md), which holds agreed work postponed to a known stage, an idea may still be
dropped or reshaped. When one is taken up it gets a design and a plan in [`../plans/`](../plans/README.md), and its file
here links there.

## Format

One file per idea, `<topic>.md`:

```markdown
# <Idea>

**Since:** YYYY-MM-DD

## Problem

<What is missing or painful.>

## Direction

<A rough approach, open questions.>
```

| Idea                                                        | Since      | What                                                                                                                                  |
| ----------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| [User settings](./user-settings.md)                         | 2026-11-15 | a settings screen stored in the local SQLite database; never turns off confirmation                                                   |
| [Stale project cleanup](./stale-project-cleanup.md)         | 2026-11-15 | find projects untouched for a customizable period (default 1 year); offer their `node_modules`, `target/` and other regenerable parts |
| [Ignore list](./ignore-list.md)                             | 2026-11-15 | an Ignore button on a section or item (e.g. Docker images), remembered in settings, restorable from there; ignored means not watched  |
| [Uninstalled app leftovers](./uninstalled-app-leftovers.md) | 2026-11-15 | data left in `~/Library` by apps the user removed, matched by bundle identifier, offered per app                                      |
