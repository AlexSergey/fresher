# Storage Leak Detector

**Since:** 2026-11-15

## Problem

Large files are easy to find; **folders that keep growing for no clear reason** are not: logs growing 500 MB a day, a
browser cache that never stops growing, Docker accumulating unused layers, an app leaving thousands of temporary
files, temporary folders not cleaned after processes exit.

## Direction

- The key difference: detect **abnormal storage behaviour**, not big files.
- Built on watch mode's snapshots ([`storage-history.md`](./storage-history.md)): for each folder and app, the growth
  rate across scans; a folder whose size grows steadily scan after scan, or faster than its usual rate, is flagged as a
  possible leak with its rate ("+500 MB a day for 9 days").
- Signals: steady growth, file count growing (thousands of small temp files), temp folders whose owning process is gone,
  logs never rotated.
- A flag is information with an explanation and, where a rule applies, a cleanup suggestion; never an automatic action.
- Detection happens when the user scans; no background monitoring.

Open questions:

- How many scans are needed before a trend is trusted (two points are not a trend)?
- Thresholds as data (rule 5 of [`../rules.md`](../rules.md)), per category.
