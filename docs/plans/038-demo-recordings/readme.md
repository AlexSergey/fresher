# 038 — Demo recordings

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** sandbox, app, app/tauri, app/ui, scripts, fixtures, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Think how to make a script that records a short video for the README, which can be
turned into a GIF for GitHub. It must be part of the application and be extended with scenarios."

A README without a moving picture of the app loses most visitors, and a recording made by hand is stale after the next
change and may show private paths. The recording must be a command: same result every time, on made-up data.

## Goals

1. One command produces the README's animation and a video from the real app, with no hand on the mouse.
2. A scenario is a small file; adding a module's story is adding a file.
3. Nothing of the developer's Mac is in the picture, and nothing of this ships in a release build.
4. A scenario that no longer plays fails loudly, naming the step.

## Non-goals

- Recording in CI: capturing the screen needs a permission only a person's Mac has.
- Narration, zooms, titles, editing. A scenario is clicks and pauses.
- A mocked interface: the picture shows the real core on a real (generated) disk.

## Design

Three parts, each reusing what exists.

**1. A demo world** — `fixtures/demos/<world>.toml`, in the case-file format of the sandboxes, written by the same
generator into a folder with a neutral path. Unlike a test's case file it is a believable Mac: several modules at
once, sparse files of realistic sizes (tens of gigabytes that cost no disk), realistic names. The world's clock is its
own, so dates in the picture never change between recordings.

**2. A scenario** — `fixtures/demos/scenarios/<name>.toml`: the world, the window size, and steps from a small closed
list:

```toml
world = "developer-mac"
window = { width = 1100, height = 720 }

[[step]]
click = { role = "button", name = "Scan" }
[[step]]
wait = "scan-finished"
[[step]]
pause = 1200
[[step]]
hover = { role = "button", name = "Details" }
```

Targets are addressed by accessible role and name, as the e2e tests do, so a renamed button breaks the scenario, not
silently the picture. Steps: `click`, `hover`, `scroll`, `wait` (a named state of the app), `pause`.

**3. The player and the recorder.**

- **The player** lives in the app, debug builds only, beside sandbox mode: started with `FRESHER_DEMO=<scenario>` the
  shell sets the window's size and place, hides the build banner, and hands the steps to the UI through one typed
  command. The UI part is a lazy chunk that is never in a release bundle: it plays the steps on the real interface
  with a drawn pointer that glides to each target, and reports "finished" or "failed at step N" to the shell, which
  exits with that status.
- **The recorder** is `scripts/demo-record.ts` (`npm run demo:record -- <scenario|all>`): generates the world, starts
  the build in sandbox mode with the scenario, captures the window's rectangle with the system's screen recording
  until the player finishes, then encodes with `ffmpeg` (a development tool on the developer's Mac, listed in
  [`dependencies.md`](../../dependencies.md), never a dependency of the app): an `.mp4` at full resolution and a
  `.gif` scaled to the README's width through a two-pass palette. It refuses a result over the budget.
- `npm run demo:check -- <scenario|all>` plays without capturing: no permission needed, run before a release and
  whenever a scenario or a screen it touches changes.
- **Output**: `assets/demo/<scenario>.gif` committed and referenced by the README; the `.mp4` stays out of git (for
  the release notes and the site). Budget: 800 px wide, 12 frames a second, 20 seconds, 5 MB.

The first scenarios: `overview` (scan, the ring, the danger zones — the README's hero), `cleanup` (select, review,
confirm, the result, put back), and one per headline module as they are wanted.

**Testing.** The step list's parser and the player's step logic are unit tested, negatives first (an unknown step, a
target that matches nothing or two things, a wait that times out). A script test covers the recorder's command plan
with a fake spawner, as `sandbox-dev.test.ts` does. The demo world is a sandbox like any other: the generator's tests
cover its format, and `demo:check` is its run. Sandbox mode already guarantees the cleaner reaches nothing outside the
world.

## Plans (after approval)

| #   | Plan      | What                                                                    |
| --- | --------- | ----------------------------------------------------------------------- |
| 01  | World     | the generator takes a world file by path; the first demo world          |
| 02  | Player    | the scenario format, the shell's demo mode, the UI's player and pointer |
| 03  | Recorder  | `demo:record`, `demo:check`, capture, encoding, the budget              |
| 04  | Scenarios | `overview` and `cleanup`, the README's picture, docs                    |

Depends on plans 007 and 027. The demo world is also what plan [014](../014-onboarding/readme.md)'s "sample data"
needs; whichever comes first builds it.

## Ideas this touches

"Demo mode and automatic screenshots" of [best practices](../../ideas/best-practices.md), 27.

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **The real app in sandbox mode is recorded**, not the mocked interface in a test browser: the picture is the
   product, with the native window. The price: a person's Mac and the screen recording permission.
2. **The player is inside the app, debug builds only**; no outside tool clicks the window.
3. **Scenarios are data** (a closed list of steps), not code.
4. **The GIF is committed, the video is not.**
5. **The build banner is hidden while recording**; the README says the picture shows generated sample data.
6. **`ffmpeg` only**; no second encoder until the picture's quality asks for one.

## Conclusions

Not written yet.
