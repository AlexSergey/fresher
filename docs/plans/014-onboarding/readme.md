# 014 — Onboarding and the first scan

**Status:** 📝 draft · **Version:** 0.5.0 · **Date:** 2026-11-15 · **Modules:** engine, store, app/ui

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Onboarding too."

The first minutes decide whether a person trusts an app that can see and delete their files. Onboarding says what the
app does and does not do, lets the person choose what it looks at, and gets them to a first useful result — without a
wall of prompts.

## Goals

1. In under a minute the person knows: what the app does, that nothing leaves the Mac, that nothing is deleted without
   their confirmation, that it does nothing in the background.
2. The first scan starts by their click, with a scope they have seen.
3. No permission is demanded to begin.
4. The app can be tried on sample data before it reads anything.
5. Everything can be skipped, and shown again later.

## Non-goals

- Accounts, sign-up, e-mail, licence keys.
- Tours that block the interface; "what's new" pop-ups after an update.
- Any urgency: no alarm counters, no "your Mac is at risk".

## Design

### The first run

Three short screens, each answering one question, each with "Skip":

| Screen            | Says                                                                                               |
| ----------------- | -------------------------------------------------------------------------------------------------- |
| What this is      | what the app finds and how it explains it; the three promises: private, nothing deleted without you, nothing in the background |
| What it looks at  | the folders of the first scan, shown and editable; what is never touched                           |
| What it may see   | what works right now, and what Full Disk Access would add — offered, not required; "Continue without it" has equal weight (plan [006](../006-permissions/readme.md)) |

Then the start state of the overview: one button, "Scan". The app reads nothing from the disk before that click.

### Sample data

"Try it on sample data" opens the app on a made-up Mac: the demo module and a scenario of plan
[007](../007-safe-development/readme.md). A banner says the data is not real; nothing is read and nothing can be
removed. One click returns to the real start state.

### The first scan and the first result

- The scan fills the overview live (plan [012](../012-storage-map/readme.md)); it can be cancelled.
- With the first result, a few hints appear once, next to what they explain, each dismissed with a click: what the
  danger levels mean, that sizes marked `+` are incomplete and why, how reviewing and confirming works.
- Nothing is pre-selected for removal by onboarding; the first cleanup is the person's own decision, through the normal
  confirmation.

### State

- "Introduction seen" and each dismissed hint are settings (plan [005](../005-settings/readme.md)); "Show the
  introduction again" and "Show hints again" live there too.
- All text comes from the dictionary; every screen works by keyboard and with VoiceOver (plan
  [003](../003-base-ui/readme.md)).

## Plans (after approval)

| #   | Plan              | What                                                             |
| --- | ----------------- | ---------------------------------------------------------------- |
| 01  | First run         | the three screens, skip, the stored state                        |
| 02  | Sample data       | the demo scenario in the app, the banner, the way back           |
| 03  | First scan        | the start state, live filling, the first result                  |
| 04  | Hints             | the one-time hints and their reset                               |

Depends on plans 002, 003, 005, 006, 007 and 012.

## Ideas this touches

The "trust contract" and the demo mode of [best practices](../../ideas/best-practices.md);
[user settings](../../ideas/user-settings.md).

## Decisions

None yet.

## Agent's calls (to check)

1. **Three screens, all skippable**, and nothing is read from the disk before the first "Scan".
2. **Full Disk Access is offered, never required**, with "Continue without it" of equal weight.
3. **Sample data mode** on the demo scenario.
4. **Hints appear once, in place**; no blocking tour.
5. **No "what's new" pop-ups.**
6. **Target version 0.5.0.**

## Conclusions

Not written yet.
