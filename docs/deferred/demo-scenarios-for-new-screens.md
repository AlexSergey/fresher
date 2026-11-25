# Demo scenarios for the newer screens

**Agreed:** 2026-11-22, by the user's delegation (plan [038](../plans/038-demo-recordings/readme.md)) ·
**Due at:** when a screen of plans 031, 032, 035, 036, 037 or later is wanted in the README or the release notes;
the two GIFs again after the user's walk of plan 042

## What is postponed

Plan 038 shipped two scenarios, `overview` and `cleanup`, on the screens of `main` at its start (on 2026-11-25 the
user kept `cleanup` only: "the first is enough"). The screens of the later
plans (Xcode, Docker's "Described, not cleaned" section and its insight screen, the Trash line, and whatever plans 035
and 036 add) have none.

When one is wanted:

- one scenario file per story, `fixtures/demos/scenarios/<name>.toml`, steps from the closed list;
- the world extended with the module's scenery and its category id in `shows`, so the scan test keeps it honest (the
  first world leaves xcode, docker, trash, jetbrains and Ollama out, each for a reason in its head comment);
- the card's "Details" buttons share one accessible name today, so a scenario cannot open one category; name them
  "Details of {name}" first (see the user's review of 2026-11-22).

Nothing else is to be built: the player, the recorder and the checks exist.

**The GIF after plan 042** was recorded by the user on 2026-11-25 (`assets/demo/cleanup.gif`, the README's hero).

## Why

A scenario with no reader is a scenario that breaks silently when a text changes; `demo:check` runs by hand. Each
story is written when its picture is wanted.
