# 039 — The agreement, the danger zone and the wording

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, engine, store, testing, sandbox, e2e, app, app/ui, scripts, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Think whether we should add some agreements, for example to protect ourselves if the
user deletes something by accident or because of a bug in the app. Perhaps mark in the review area that it is a danger
zone. And improve the hint texts across the whole app. Make this a plan too."

The app deletes a real person's files. Before the first release three things must be true: the person has been told
plainly whose decision a removal is and what the app does not promise; the last screen before a removal looks as
serious as it is; and every sentence in the app helps them decide.

## Goals

1. Before the first cleanup the user has read and accepted a short agreement, written in plain English.
2. Review shows risky items apart, as a danger zone, and a High item cannot be removed in passing.
3. Every choice, level, reason and result reads in one voice, and a missing or weak description fails a check.
4. When something does go wrong, the app says how to get it back and how to report it.

## Non-goals

- Fear: no alarm colours on safe things, no "your Mac is at risk", no countdown ([best practices](../../ideas/best-practices.md), 24).
- An agreement that replaces safety: the confirmation, the Trash and the journal stay exactly as they are.
- Accounts, a licence server, sending the acceptance anywhere: it is a local setting.
- Translations: this plan makes the English final; other languages are [an idea](../../ideas/ui-translations.md).

## Design

### 1. The agreement

- **What it says**, in five short lines, no legal wall: the app is free and open source and comes with no warranty;
  it removes only what you confirm, so each removal is your decision; a danger level is the app's best judgement, not
  a guarantee; keep a backup of anything you cannot lose; items go to the Trash and can be put back until you empty
  it. A link opens the full licence text.
- **When**: once, before the first real cleanup — not before the first scan, which changes nothing. With plan
  [014](../014-onboarding/readme.md) it becomes the last line of the first-run screens; without it, a sheet before
  the first confirmation. "I understand" and "Not now" (the cleanup does not start; scanning still works).
- **State**: the accepted text's version and date, in the settings file (plan [005](../005-settings/readme.md)). A
  changed text asks again and says what changed. The text is always readable under Settings → About.
- **The same words outside the app**: a "No warranty" section in the README (plan [040](../040-readme/readme.md))
  and on the release page, so nobody meets them for the first time after installing.
- **What it is worth**: the two licences already disclaim warranty and liability; the agreement makes that visible
  and recorded. It lowers the risk, it does not remove it, and it is not a lawyer's work (open question 1).

### 2. If something went wrong

- A help entry on the result screen and on Cleanups: "Removed something you need?" — put back from the journal, find
  it in the Trash, what a permanent delete means, how to report a wrong finding.
- A wrong finding is the most valuable report the project can get: an issue template "the app offered something it
  should not have", and the bug-report file once it exists ([deferred](../../deferred/bug-report-file.md)). Nothing
  is sent by the app.

### 3. Review as the danger zone

- Review splits in two: **Safe to remove** (Safe and Low) and a **Danger zone** (Medium and High), the second set
  apart by a border and the level's colour, with the level's meaning from the core and, per category, the one
  sentence of what is lost.
- The screen's header names the highest level selected before the user reaches the button.
- **High needs its own tick**: "I understand these may be the only copy" — per confirmation, never remembered. The
  button stays inactive without it. A permanent delete keeps its own tick; the two are never merged into one.
- Nothing in the danger zone is ever pre-selected; "Select all" there asks first.
- The split and the wording come from the core (levels and meanings); the UI only draws them.

### 4. The wording

- **A writing guide**, `docs/writing.md`: one voice (plain, calm, second person, no jargon, no exclamation marks),
  the shape of each kind of text — a category (what it is, who made it, what happens if removed, whether it comes
  back), a choice (what it does, what you keep or lose), a left-out reason (what stopped the app, what the user can
  do), a result (what happened, what did not, what next) — a glossary of the words used for one thing only ("remove",
  "move to the Trash", "delete permanently", "left out"), and length limits.
- **A pass over both dictionaries** (`crates/core/locales/en.json`, `app/ui/src/locales/en.json`, about 500 texts)
  against the guide, screen by screen, each change looked at in the gallery and the app.
- **Hints where there are none**: every control, number and icon whose meaning is not on screen gets a description
  (rule 7) — a tooltip or a line beneath it, reachable by keyboard and by VoiceOver.
- **A check that keeps it so**: a script over the dictionaries and the module texts — every choice and setting has a
  description, every category has all its parts, no text over its limit, no banned word, glossary words used
  consistently. It runs in pre-push.

**Sandbox and tests.** The agreement's state lives in the data folder, so a sandbox starts unaccepted:
`fixtures/sandboxes/cleanup-guards.toml` gains the negative "no cleanup starts before the agreement is accepted", and
the autotest accepts it explicitly, as a user would. The engine refuses a cleanup without the acceptance — the UI's
sheet is not the guard. UI e2e, negatives first: declined, a High selection without its tick, a changed text asks
again; then the accepted flow and the two zones. New screenshot baselines for every screen whose text changed. The
finish walks a sandbox with one item of each level by hand.

## Plans (after approval)

| #   | Plan        | What                                                                     |
| --- | ----------- | ------------------------------------------------------------------------ |
| 01  | Agreement   | the text, its version, the engine's refusal, the sheet, About, the README |
| 02  | Danger zone | Review's two zones, the header, the High tick                            |
| 03  | Help        | "Removed something you need?", the issue template                        |
| 04  | Guide       | `docs/writing.md`, the glossary, the check script                        |
| 05  | The pass    | both dictionaries and the modules' texts, hints, baselines               |
| 06  | Finish      | docs, the walk                                                           |

Depends on plans 005, 008 and 027; 01 fits into plan 014 if that is built first. The pass (05) is best done last
before the release, after the modules of plans 031–037 have their texts.

## Ideas this touches

The "trust contract" of [best practices](../../ideas/best-practices.md);
[Cleanup Preview & Safe Removal](../../ideas/cleanup-preview-and-safe-removal.md) (nothing pre-selected above Safe).

## Open questions

1. **A lawyer's look at the agreement and the licences** before the first public release: which country's law the
   author is under, and whether a consumer there can waive anything at all. It costs money and names a legal identity
   (as plan [013](../013-release/readme.md), question 2), so it is the user's to decide. The plan can be built
   without it; the text is then the agent's best effort, not legal advice.

## Decisions

None yet.

## Agent's calls (to check)

1. **The agreement comes before the first cleanup, not the first launch**, and is accepted once per version of its
   text.
2. **Five plain lines**, with the licence one click away; no scroll-to-accept.
3. **The engine enforces it**, not only the interface.
4. **Medium and High form the danger zone; only High needs the extra tick**, every time.
5. **A writing guide and an automatic check**, so the texts stay good after this pass.
6. **Calm wording stays**: the zone is marked by structure and one colour, not by alarm.

## Conclusions

Not written yet.
