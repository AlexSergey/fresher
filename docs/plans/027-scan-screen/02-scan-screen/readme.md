# 027/02 — The Scan screen

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-20 · **Modules:** app/ui, e2e

The overview becomes one screen: the hero's ring of the home volume with its legend, the category cards in four
danger zones, the selection bar. The map block of the home volume, the view switch, the view by place and the rows
under the cards leave. The screen keeps its name, Overview, until 03/02. The design:
[`../00-design.md`](../00-design.md), sections 2, 3, 4 and 6 (agent's calls 3, 4, 7, 9, 13 and 16).

## Steps

| #   | Step                                                   | Status      | Depends on   | Shared files                                                                                   |
| --- | ------------------------------------------------------ | ----------- | ------------ | ---------------------------------------------------------------------------------------------- |
| 01  | [The hero's ring and legend](./01-hero-legend.md)      | 🔨 in progress | 01/01, 01/03 | `app/ui/src/screens/overview/overview-screen.tsx`, `app/ui/src/locales/en.json`, `app/ui/src/product/sizes.ts` |
| 02  | [Danger zones and cards](./02-zones-and-cards.md)      | 🔨 in progress | 01/01, 01/02 | `app/ui/src/locales/en.json`                                                                    |
| 03  | [The overview in zones](./03-scan-content.md)          | 🔨 in progress | 02/01, 02/02 | `app/ui/src/screens/overview/`, `app/ui/src/state/{navigation,storage}.ts`, `app/ui/src/locales/en.json` |

01 and 02 touch different components and run in parallel (both need 01/01 for the zone names in the core's
dictionary; 01 needs 01/03's ring, 02 needs 01/02's menu); 03 wires both into the screen.

## Conclusions

Built in three steps, awaiting the user's walk.

- **02/01** the hero's ring and legend; the caption names the disk in a sentence; the legend's "i" opens the core's
  description of the part. Stat tiles and the usage bar beside the ring went. The ring's centre no longer swallows
  the pointer; a stale highlight no longer dims the whole ring (review fix).
- **02/02** danger zones and cards: cards do not move, each has Details and a settings menu (ignore the category, or
  its module). A card shows the access text of a limited category on a line.
- **02/03** the overview in zones: the view switch, the view by place, the by-meaning rows and the home map block
  are deleted from the UI. Perf specs pass only with `--workers 1` (timing based). The "Found, but left out" section
  was added in 03/01's review for categories with only left-out lines.
