# Opportunities

**Since:** 2026-11-15 · The user's product-strategy assessment: where Fresher can win.

| Direction              | Market                       | Fresher's opportunity | Idea                                                           |
| ---------------------- | ---------------------------- | ------------------------- | -------------------------------------------------------------- |
| Cache cleaning         | well covered                 | go deeper                 | [`system-junk-cleaner.md`](./system-junk-cleaner.md)           |
| Large files            | well covered                 | faster                    | [plan 035](../plans/035-large-files/readme.md)             |
| Duplicates             | well covered                 | more accurate             | [`duplicate-finder.md`](./duplicate-finder.md)                 |
| Uninstaller            | well covered                 | better linking of files   | [plan 036](../plans/036-uninstall/readme.md)               |
| RAM monitor            | well covered                 | integrate with disk       | [`app-resource-profile.md`](./app-resource-profile.md)         |
| CPU monitor            | well covered                 | integrate with disk       | [`app-resource-profile.md`](./app-resource-profile.md)         |
| Disk analyzer          | well covered                 | universal                 | [`storage-explainer.md`](./storage-explainer.md)               |
| Storage attribution    | limited                      | **big**                   | [`live-storage-attribution.md`](./live-storage-attribution.md) |
| Storage leak detection | no full equivalent found     | **big**                   | [`storage-leak-detector.md`](./storage-leak-detector.md)       |
| App resource profile   | features scattered           | **big** — not now         | [`app-resource-profile.md`](./app-resource-profile.md)         |
| Storage history        | partly covered               | **big**                   | [`storage-history.md`](./storage-history.md)                   |
| Cleanup simulator      | no full implementation found | **big**                   | [`cleanup-simulator.md`](./cleanup-simulator.md)               |
| Process timeline       | not found                    | **big** — later           | [`process-storage-timeline.md`](./process-storage-timeline.md) |
| Cleanup guardian       | partly covered               | **big**                   | [`cleanup-guardian.md`](./cleanup-guardian.md)                 |
| Cleanup journal        | partly covered               | better recovery           | [`cleanup-journal.md`](./cleanup-journal.md)                   |

The common thread of the big opportunities: **time**. Snapshots over time (watch mode) turn a cleaner into a tool that
explains how the disk changes — history, leaks, recurrence — which is the base architecture's job to make cheap
and fast.
