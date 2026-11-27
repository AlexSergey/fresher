# A first-run onboarding

**Since:** 2026-11-27 · From the second tester's feedback on 0.1.0

## Problem

A tester opened Fresher, scanned, and only then saw that many folders were not scanned: Full Disk Access was off. To
scan everything they had to turn it on, reopen the app and scan again. They want the permission asked at once, with a
way to skip: "a very minimal onboarding".

## Direction

- One screen on the first run, before the first scan: "Fresher reads your disk only when you press Scan. To see
  every folder, give it Full Disk Access." with [Open System Settings] and [Skip for now]; then the Scan screen.
- The app does nothing without the user: the screen scans nothing and starts nothing in the background. Access is
  probed only on a press (Open System Settings, or a check after the user comes back), never on its own at start.
- Skipping is a full answer: the scan works as today, and the existing access notice ("Some folders were not
  scanned", with Open System Settings and Reopen Fresher) stays for those who skip.
- Shown once; open question: whether it can be reached again later (from the access notice is enough today).
- A candidate for 0.2.0.
