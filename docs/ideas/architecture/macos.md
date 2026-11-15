# macOS specifics

**Since:** 2026-11-15

## Dataless files (iCloud and other cloud folders) — critical

- Walking iCloud Drive or another file-provider folder can **download** files whose data is not local.
- `setiopolicy_np(IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES, IOPOL_SCOPE_PROCESS, IOPOL_MATERIALIZE_DATALESS_FILES_OFF)`
  at startup stops that for the whole process; the system's own `du` does the same.
- Without it the scan breaks our no-network rule ([`../../restrictions/privacy.md`](../../restrictions/privacy.md)).

## Purgeable space

- One approach: Finder's free space (through Apple Events) minus `statfs` free space, falling back to
  `diskutil apfs` output. It needs the Automation permission.
- Preferred: in-process, from the URL resource keys `volumeAvailableCapacityForImportantUsage` /
  `…ForOpportunisticUsage` minus `volumeAvailableCapacity` — no permission prompt **(unverified: from Apple's
  documentation; check in the plan)**.
- Purgeable is information, never cleanable.

## Local snapshots

- Listed with `tmutil listlocalsnapshots /`; report-only, thinned only through `tmutil`, never by deleting files.
- `tmutil` touches every mounted volume: skip it while a network volume is mounted, or the user gets a "network volume"
  privacy prompt.

## APFS clones

- `ATTR_CMNEXT_CLONEID` and `ATTR_CMNEXT_PRIVATESIZE` through `getattrlist(bulk)` with `FSOPT_ATTR_CMN_EXTENDED`
  ([scanning](./scanning.md)).

## Full Disk Access

- Probe: is `/Library/Application Support/com.apple.TCC/TCC.db` readable? Only with Full Disk Access.
- Re-check when the window gains focus: the user can grant it while the app runs.

## Moving to the Trash

- Through Finder (Apple Events): "Put Back" works, but it needs the Automation permission and plays a sound.
- Through `NSFileManager trashItemAtURL:resultingItemURL:error:`: no extra permission; the resulting URL tells where
  the item went. The `trash` crate passes no result URL and cannot list or restore on macOS.
- Preferred: call `trashItemAtURL` directly through `objc2-foundation`, store the original and resulting paths in our
  own undo log, and offer "put back" from it.

## Finding what belongs to an app

- Read each app's `Info.plist` directly (bundle id, team id, helpers); Spotlight (`mdfind`, `NSMetadataQuery`) is
  unreliable and is only a supplement.
- Containers: read `.com.apple.containermanagerd.metadata.plist` → `MCMMetadataIdentifier` in each container; group
  containers through the application group identifier.
- Match by bundle id and team id across a fixed list of `~/Library` locations; a normalised app name only with an
  extra check, never alone (loose name matching is the top cause of wrong deletions in the field).
- Privileged helpers (`SMJobBless`) and launch agents are matched by the program they start.
- A skip list of locations never searched deeply (the Trash, iCloud's `Mobile Documents`, password-manager helpers).
- Whether `lsregister` (Launch Services) helps find apps outside `/Applications` — **unverified**.
