# Categories

The catalogue of everything the app can name on a disk. Every category the app shows has a file here, and every file
here matches a rule in `crates/rules`. A category that is not documented here is not shipped (rule 3 of
[`../rules.md`](../rules.md)).

There is no "Other", "Miscellaneous" or "System data" category. Space no rule recognizes is shown as its real folders.

## Danger levels

The danger level says what the user risks by removing a category. It is shown next to the category, on the
confirmation screen, and the confirmation names the highest level among the selected items.

| Level      | Meaning                                                                                          | Examples                                                                   |
| ---------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| **Safe**   | Rebuilt automatically when needed; removing it costs nothing but a little time.                  | app caches, logs, Xcode derived data                                       |
| **Low**    | Rebuilt on demand, but it costs time or bandwidth (a download, a rebuild).                       | package manager caches, `node_modules`, Docker build cache                 |
| **Medium** | Can be restored, but not automatically; you may lose settings or need to set something up again. | settings of an older IDE version, old iOS simulators, unused Docker images |
| **High**   | May hold the only copy of your data; removing it can lose work.                                  | Docker volumes, stopped containers, iOS device backups                     |

The levels are final for a category in its default form; a rule may raise (never lower) a level for a specific finding,
for example a `node_modules` of a project touched yesterday.

## Format

One file per category, `<area>-<name>.md` (`dev-`, `docker-`, `jetbrains-`, `system-`, `app-`):

```markdown
# <Category name, as shown in the app>

**Danger level:** <Safe | Low | Medium | High> · **Rule:** `<rule id>` · **Since:** <version>

## What it is

<One or two plain-English sentences: what these files are and which app makes them.>

## Where it lives

<The paths or patterns, with `~` for the home folder.>

## What happens if you remove it

<The consequence, and whether and how it comes back.>

## How the app decides

<What the rule checks so that it never claims something else; the cases it refuses.>
```

## Catalogue

| Category | Danger level | Rule | File |
| -------- | ------------ | ---- | ---- |

No category is documented yet. The first ones (JetBrains stale versions, developer caches, `node_modules`, Docker) come
with the plans that implement their rules.
