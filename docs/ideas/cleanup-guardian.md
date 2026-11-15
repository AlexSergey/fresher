# Cleanup Guardian

**Since:** 2026-11-15

## Problem

The user cleaned 5 GB and two weeks later it is all back.

## Direction

- Remember which categories were cleaned, which apps produce junk again, how soon the problem returns, and which
  folders grow systematically.
- Then suggest personal maintenance rules: "Xcode has accumulated 4 GB of DerivedData again. Check whether it can be
  cleaned?"
- Shown **when the user opens the app or scans** — never as a notification from the background
  ([`../goal.md`](../goal.md), section 8); a suggestion, never an automatic cleanup.
- Built on the cleanup journal ([`cleanup-journal.md`](./cleanup-journal.md)) and watch mode
  ([`storage-history.md`](./storage-history.md)); close to [`storage-leak-detector.md`](./storage-leak-detector.md).

Open questions:

- A "maintenance list" the user builds once (categories to review on every visit)?
