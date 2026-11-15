# Snapshot summaries instead of full trees

**Since:** 2026-11-15 · **Proposed by:** the agent

A counterproposal to decision 9 of the [architecture ideas](./README.md) ("snapshots in SQLite as an `items` table")
and to "Retention" in [snapshots and watch mode](./snapshots-and-watch-mode.md).

## Problem

A full `items` table per snapshot keeps every path on the Mac, scan after scan:

- **Privacy.** "Paths alone already reveal a lot about a person"
  ([`../../restrictions/privacy.md`](../../restrictions/privacy.md)). A history of every file name — documents,
  projects, clients — is the most sensitive file the app could create, and any program running as the user can read
  it.
- **Size.** Millions of rows per scan; even with "full trees for the last few scans only" the store grows by gigabytes
  for a feature whose question is about apps and categories.
- **Need.** Watch mode asks "which apps and categories grew, by how much, and since when"
  ([`../../goal.md`](../../goal.md), section 5). That does not need every file of every past scan.

## Direction

Store what each question needs, and no more:

| What is stored                                                 | Kept                               | Answers                                             |
| -------------------------------------------------------------- | ---------------------------------- | --------------------------------------------------- |
| per finding and per category totals                            | every scan                         | "this app grew 6 GB since Monday", "what came back" |
| a **summary tree**: folders above a size threshold, to depth N | every scan                         | "which folder grew", for big folders                |
| the **full tree** of the latest scan                           | latest only, replaced on each scan | instant open, fast rescans, drill-down now          |

- Diffs between old scans work on totals and the summary tree; file-level detail exists only between the latest scan
  and the current one.
- The full tree can be turned off in the settings ([`../user-settings.md`](../user-settings.md)): the app then rescans
  in full every time.
- **Forget history** in the settings deletes all snapshots. The hash cache of [duplicates](./duplicates.md) is keyed
  by device and inode, so it keeps no paths.

Trade-offs:

- Lost: a file-level diff between two old scans ("which file appeared in March"). Watch mode's goal does not need it.
- The summary tree's threshold hides small folders that grow slowly; their growth still shows in their category's
  total.

Open questions:

- Threshold and depth: folders of 100 MB and more, or a fixed depth of 4 under each root?
- Should the app store the full tree at all by default, or only once fast rescans are measured to need it (rule 4 of
  [`../../rules.md`](../../rules.md))?
