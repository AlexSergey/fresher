# 012/06/02 — The run by hand and the conclusions

**Status:** ✅ approved (by delegation, 2026-11-17) · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** docs

## Run by hand

Waits for the user. Tick each box when it holds; write what was seen under Conclusions.

- [ ] **The developer's Mac**, `npm run dev`, the default root `~`, after a full scan:
  - [ ] the bar's parts add up to the capacity of `df -k` for the home's volume;
  - [ ] "macOS" equals the used space of `/`, Preboot and Update; "Swap and sleep image" the used space of
        `/System/Volumes/VM`; Free the available space;
  - [ ] "Outside the scanned folders" lists its reasons, with the unreadable folders' count and the permission;
  - [ ] the purgeable note is there, at least zero;
  - [ ] by meaning: the apps with their data folders, the Trash, the categories with their selection and the line
        "more under apps" where part of a category sits under apps, the system lines;
  - [ ] by place: opening the home descends where one folder holds 90 % of its parent; each row says what is explained;
  - [ ] the `Shares::of` time and heap on the developer's home, restored from the app's cache (the generated LARGE
        tree gave 3.9 ms and 625 226 bytes; the budget is 50 ms and 5 MB).
- [ ] **During a scan**: "Explained" grows as findings arrive; "Explained" plus "Not scanned yet" stays the used space;
      at the end the full map replaces it in one move. Quit and reopen: the restored map is there at once.
- [ ] **A root at a volume's mount point** (an external drive, or a disk image mounted by hand): its map says "Not
      measured", with no neighbours; a root of `/` is tried once and what the map says is written down (design, 02/04).
- [ ] **The sandbox**: `npm run dev:sandbox -- storage-map`; every line of `EXPECTED.md` holds, including the map's.
      This needs plan 05, which waits for plan 007 in `main`.
- [ ] **Ignore and restore** a category: its bytes move to "Your folders" and back, the total unchanged, no rescan.

## Files

- `docs/plans/012-storage-map/readme.md`, `00-design.md` and every plan's and step's `## Conclusions` — what was built,
  what changed from the design and why, what was measured (`Shares::of` time and heap; the map's query during a scan),
  the run by hand above, the checks on the last code commit.
- `docs/plans/README.md` — the row of 012: 🔨 while building, ✔️ when the user has looked at the result.

## Checks

- `npm run pre-push` passes on the branch's last commit (lint, crate guard, knip, machete, deny, the scripts' tests,
  Rust tests and coverage ≥ 90 %, UI tests and coverage ≥ 90 %); `npm run test:e2e` passes; the numbers in the
  conclusions.
- The user has looked at the result before the squash-merge ([`../../../workflow.md`](../../../workflow.md)); the
  agent's calls stay open for the user's check.

## Conclusions

Not written yet.
