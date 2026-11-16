# 006/01 — Probe the system

**Status:** ✅ approved · **Version:** 0.4.0 · **Date:** 2026-11-15 · **Modules:** NO_COMMIT, docs

Throwaway probes that settle, on a real Mac, what the design can only assume: which places make macOS prompt and which
only refuse, which file tells Full Disk Access without a prompt, whether a grant applies without a relaunch, the System
Settings address of each pane and the `Info.plist` keys. No product code; the probes stay in `NO_COMMIT/006-probes/`
and are never committed. The answers go into the "Probe results" section of [`../00-design.md`](../00-design.md) and
change the design where they disagree with it. The design: [`../00-design.md`](../00-design.md), sections 1, 4 and 7.

Depends on nothing; runs in parallel with plan 02.

## Steps

| #   | Step                                                   | Status   | Depends on | Shared files                                  |
| --- | ------------------------------------------------------ | -------- | ---------- | --------------------------------------------- |
| 01  | [Prompts, refusals and the Full Disk Access check](./01-prompts-and-checks.md) | ✅ approved | — | `../00-design.md` ("Probe results") |
| 02  | [Panes, reason strings and development builds](./02-panes-and-reasons.md) | ✅ approved | — | `../00-design.md` ("Probe results") |

The two steps are independent and may run in either order; each appends its own part of "Probe results".

## Conclusions

Not written yet.
