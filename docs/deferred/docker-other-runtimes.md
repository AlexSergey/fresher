# Docker's other runtimes

**Agreed:** 2026-11-22, by the user's delegation (plan [032](../plans/032-docker/readme.md), agent's call 2) ·
**Due at:** after plan 032's run on the developer's Mac

## What is postponed

Plan 032 describes only the runtimes that have an app bundle: Docker Desktop and OrbStack. Postponed:

- **Colima, Lima, Podman machines and Rancher Desktop.** Each needs three facts, checked read-only on a Mac that has
  it, before a row of `modules/docker/runtimes.toml` can name it:
  - a sign that the runtime is up that is not an app (a socket under the home, a process), since the module's gate is
    an installed app that is running and a socket the walk saw;
  - the place of its CLI (they have no bundle; the module runs only a CLI inside the runtime's own app bundle today) and
    its context name for `--context`;
  - its disk file's place and how it gives freed space back (the `gives-back` sentence), which differ per runtime.
- **One line per image or volume** (`docker system df -v`): the four parts say how much is unused, not which image or
  volume; a volume's size and name would help the user decide, and each line would need its own wording and caution.
- **"Show in Finder" for a disk file**, so the user can look at what the figures are about.
- **A link from the storage map's line** (the runtime's app) **to the insight**; plan 032 changes nothing in the map
  (agent's call 5).

## Why

Plan 032's design reads only what could be verified read-only on the developer's Mac, and the answer's format is taken
from documentation until a runtime is seen running; adding runtimes before that multiplies the unverified rows.
