# Games not played for a long time

**Since:** 2026-11-19 · **Proposed by:** the agent

## Problem

A game takes tens of gigabytes, and a game library collects titles the user finished or forgot months ago. The
launcher shows the size but not when the game was last played next to it, and nothing suggests removing it.

## Direction

- Read the launcher's own library files (Steam's library folders and app manifests, then other launchers) to list the
  installed games with their size and the last time they were played.
- Offer a game unplayed for a number of days (a setting, 60 by default) as "your call", never pre-selected.
- **Remove only through the launcher** (Steam's own uninstall), never by deleting the game's files: the launcher keeps
  its library consistent and cloud saves stay intact ([`best-practices.md`](./best-practices.md)).
- The danger level says what returns and what does not: the game downloads again; local saves that are not in the
  cloud may be lost — name them if the launcher tells.

Open questions: how to start the launcher's uninstall without a network call or a background helper; whether a game
counts as played when only its launcher was opened.
