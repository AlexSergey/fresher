# System Junk Cleaner

**Since:** 2026-11-15

## Problem

Caches, temporary files, logs and crash reports pile up in `~/Library` and `/private/var`. Most of it is safe to remove,
some of it is not, and generic cleaners rarely explain which is which.

## Direction

- Rules for the classic junk, each a named category with its danger level and plain-English description:
  - user caches (`~/Library/Caches/<id>`) — **Safe**, rebuilt by the app;
  - logs (`~/Library/Logs`, app logs) — **Safe**;
  - crash and diagnostic reports (`~/Library/Logs/DiagnosticReports`) — **Safe**;
  - temporary files of the user (`$TMPDIR` entries older than a threshold, never ones in use) — **Safe**; taken up
    by plan [017](../plans/017-temp-files/readme.md), its own module;
  - old iOS / iPadOS software updates (`.ipsw`) — **Low** (downloaded again if needed);
  - Mail downloads and attachments cache — **Low**;
  - language files and universal binaries' unused slices are **out of scope** (they modify app bundles and break code
    signatures).
- A file currently open by a running process is skipped and reported, not removed.
- System-wide locations that need administrator rights are shown, cleaned only with an explicit, separate permission
  step.

Open questions:

- Should an app be asked to quit before its cache is removed (some apps misbehave when the cache vanishes while
  running)?

**Built in plan [033](../plans/033-logs/readme.md)** (2026-11-22, awaiting the user's walk): the user's logs and crash reports, one finding per file. User caches are plans [030](../plans/030-installed-app-caches/readme.md) and [034](../plans/034-browser-caches/readme.md). **Still an idea:** old device software updates, Mail's downloads, and the system-wide places that need administrator rights.
