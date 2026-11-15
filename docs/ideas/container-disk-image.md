# Why Docker does not give the space back

**Since:** 2026-11-15 · **Proposed by:** the agent

## Problem

On a Mac, Docker runs in a Linux virtual machine whose whole disk is one file (Docker Desktop's `Docker.raw`; other
runtimes such as Colima or Podman keep a similar disk image). Developers hit the same wall again and again:

- Finder shows the file as 64 GB or more — its maximum, not what it takes;
- after removing images and build cache, the Mac gets nothing back, because the space is freed inside the virtual disk
  and returns to macOS only when the runtime shrinks the file, later or after a restart **(unverified: how and when
  each runtime does it; check in the plan)**.

Without an explanation, the Docker section looks broken right after the user cleaned it.

## Direction

- An explanation card in the Docker section, three numbers in plain English:
  - **Maximum size** — the logical size Finder shows ("Docker may grow up to 64 GB");
  - **On your disk** — the allocated size, what the Mac actually loses;
  - **Used inside Docker** — images, containers, volumes and build cache, from the Docker CLI.
- After a Docker cleanup the result says: "Freed 8 GB inside Docker. macOS gets it back when Docker shrinks its disk
  — this can take a while, or a restart of Docker."
- **Never counted twice.** The Docker categories live inside the disk image; the totals count the image file once,
  and the categories as its breakdown, not as extra space.
- **The disk image itself is never offered.** Deleting it wipes every image, container and volume — the only copy of
  a local database included. It is information only, with no cleanup action
  ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md): Docker objects only through the CLI).
- If Docker is not running, the card shows the two file numbers and says "start Docker to see what is inside"; the app
  does not start it ([`../goal.md`](../goal.md), section 8: no background work, only what the user asks).

Open questions:

- Each runtime's way to shrink the disk (a setting, a command, a restart): point the user at it, or offer to run it as
  an explicit action?
- Which runtimes to cover first — Docker Desktop only, or Colima and Podman too?
