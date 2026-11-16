# 006/01/02 — Panes, reason strings and development builds

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** NO_COMMIT, docs

## Files

- `NO_COMMIT/006-probes/panes.sh` — opens each candidate `x-apple.systempreferences:` address with `/usr/bin/open`;
  the result is noted by hand.
- `NO_COMMIT/006-probes/plist/` — a throwaway Tauri build with `app/tauri/Info.plist` copied in, to see the merged
  `Contents/Info.plist` of the bundle.
- `docs/plans/006-permissions/00-design.md` — "Probe results": the answers below.

## What is measured

| Question                                                                                                   | Answer recorded as                    |
| ---------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| The address that opens Privacy & Security → Full Disk Access, and → Files and Folders, on the current macOS and on macOS 13 | the address per version; the fallback (the Privacy & Security pane) |
| Does `/usr/bin/open` with that address work with the network off?                                          | yes / no                              |
| Does Tauri 2's bundler merge `app/tauri/Info.plist` into the bundle, and does the dev run (`npm run tauri dev`) use it? | yes / how instead               |
| Does data from other apps (macOS 14+) take a reason string of its own, and under which key?                | the key, or "none: a fixed text"     |
| Which reason strings macOS shows for Desktop, Documents, Downloads, removable and network volumes           | the five keys of the design confirmed |
| Under `npm run tauri dev`, which app does macOS attribute access to (the terminal app)?                     | the app                               |
| Does an ad-hoc signed rebuild keep the grants of the previous build?                                        | yes / no                              |

## Produces

The pane addresses that 03/02 encodes, confirmation of the `Info.plist` route for 03/04, and the facts about
development builds that 05/03 writes into the developer docs.

## Checks

- "Probe results" has an answer for every row above, each marked measured (with the macOS version) or unverified.
- No probe file is staged.
