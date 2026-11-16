# Snapshots and watch mode

**Since:** 2026-11-15

**Taken by plan [004](../../plans/004-fast-scan/readme.md)** (2026-11-16): a scan is stored as a compact compressed
file of our own format in fixed slots (two scopes, two generations), not as an SQLite `items` table; rescans read only
the folders the change journal names. **The FSEvents question is answered**: the journal records a file growing in
place as an event for its folder, but a position older than its history gives no flag, and a long replay is slow, so
every rescan also reads a sample of the unchanged folders, bounds the replay by age and time, and falls back to a full
scan on any doubt. Folder modification times are never trusted. Diffing two scans stays for plan 010.

## Storing a scan

- A proven SQLite shape for a scanned tree:
  `items(id, parent_id, name, is_dir, size, alloc, mtime, item_count, …)` with indexes on `parent_id` and
  `(parent_id, name)`.
- Bulk inserts in one transaction; fast write settings while a snapshot is written, then back to safe ones.
- Other tools store scans as a JSON export or a compressed binary stream (blocks of CBOR under zstd with an index
  block); SQLite fits us better because the store also holds settings, history and the hash cache. *Decided later:
  plain files, not SQLite — [plan 002, decision 1](../../plans/002-base-architecture/00-design.md); the scan cache is
  plan 004's compact binary file.*
- Store per-category totals with each snapshot, so the history screen never needs to load whole trees.

## Diffing two scans (watch mode)

- Walk two snapshots keyed by path (name plus position among siblings), report **added**, **removed** and **changed**
  with the size change, and roll the change up to every folder above.
- Attribute the growth to an app and a category through the rules, so the answer is "this app grew 6 GB since Monday",
  not "this folder grew".
- Keep a "what came back after you cleaned" view: a category cleaned and grown again is the most useful signal of an
  app that keeps accumulating junk.

## Fast rescans

- A saved snapshot opens instantly; the next scan can skip unchanged folders when their modification time and child
  count match (folder mtime changes when an entry is added or removed). To verify: how reliable this is on APFS.
- File-system event history (`FSEvents`) can tell which folders changed since the last scan **(unverified, to check in
  the plan)**.

## Retention

- Keep every snapshot's totals; keep full trees for the last few scans only (size budget to decide).
