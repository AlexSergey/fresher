# External Drive Analyzer

**Since:** 2026-11-15

## Problem

External SSDs, HDDs, USB sticks and network shares fill up too, often with hidden macOS files (`.Spotlight-V100`,
`.fseventsd`, `.Trashes`, `._*` AppleDouble files) and old Time Machine backups.

## Direction

- Scan any mounted volume the user picks; the same categories and danger levels apply.
- Drive-specific findings: each volume's own Trash (`.Trashes`), AppleDouble files on non-APFS volumes, Spotlight
  index, Time Machine backups (managed only through `tmutil`, danger level **High**).
- Slow media: the scan adapts its parallelism (an HDD or a network share is slower with many threads), shows progress,
  and can be cancelled at any time.
- Watch mode per volume, keyed by the volume's UUID so a renamed drive keeps its history.

Open questions:

- Network shares (SMB/NFS): include, or leave out at first because of speed and permissions?
- Never scan a volume automatically on mount — confirm it is a user action only.
