# 004/01/03 — Volumes, links, clones and prompts

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos (probe)

## Files

- `NO_COMMIT/probes/004/volumes/` — a Cargo project outside the workspace:
  - `volume` — for a path: `statfs` (device, file system type, total and used blocks), the volume's UUID through
    `getattrlist` (`ATTR_VOL_UUID` or the nearest attribute that gives it), case sensitivity
    (`VOL_CAP_FMT_CASE_SENSITIVE`), firmlinks (`/` against `/System/Volumes/Data` for the home folder);
  - `census` — a bulk walk of the developer's home folder that prints counts only: entries, folders, files by size
    band, hard-linked files and their groups, cloned files and clone groups, cloud files, links, unreadable folders by
    `errno`, mount points crossed;
  - `prompts` — opens, one by one, each folder of a fixed list (Desktop, Documents, Downloads, Pictures, Movies, Music,
    `Library/Containers`, `Library/Group Containers`, `Library/Mail`, `Library/Safari`, `Library/Messages`, iCloud
    Drive, a removable volume, a network volume if mounted) from a fresh binary with no grant, and records whether a
    system prompt appeared and which `errno` came back while it was not answered.

## Questions to answer

1. A stable identity for a volume across reboots and remounts (the UUID), and how it differs from the device number.
2. Case sensitivity of the home volume; how the walk should treat names (the design keeps exact comparison, like the
   reference walk; this checks that nothing in the matcher depends on case).
3. How common hard links and clones are in a real home (counts and the share of bytes), so the tree's link set and
   clone accounting are sized on facts.
4. The share of files and bytes under the folding threshold candidates (256 KiB, 1 MiB, 4 MiB) and the resulting
   node count — the memory and cache estimates of the design depend on it.
5. Which folders raise a system prompt when opened by a non-sandboxed app without a grant, on the developer's macOS
   version; `EPERM` against `EACCES` for each — this sets the stand-in avoid list of plan 03/03 and the mapping of
   design call 16.
6. The peak resident memory of a process reading `getrusage(RUSAGE_SELF).ru_maxrss` (bytes on macOS, not KiB).

## Produces

Answers in the probe's `RESULTS.md`: the volume identity to store; the census (counts only); the stand-in avoid list;
the unit of `ru_maxrss`.

## Checks

- The census prints no name or path; its totals agree with `du -sk` of the same folder within 1 % (allocated bytes).
- Every folder of the prompt list has a recorded outcome (prompt or not, `errno`).
- The probe is run once per supported macOS version available to the developer; the versions are written down.
