# 038 — Demo recordings

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** sandbox, testing, app, app/tauri, app/ui, scripts, fixtures, docs

Written on 2026-11-22 from the one-file draft and a fresh read of the code on `main` 33d51a6c. Approved by the user's
delegation of 2026-11-22. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Think how to make a script that records a short video for the README, which can be
turned into a GIF for GitHub. It must be part of the application and be extended with scenarios."

A README without a moving picture of the app loses most visitors, and a recording made by hand is stale after the next
change and may show private paths. The recording must be a command: same result every time, on made-up data.

## Goal

`npm run demo:record -- overview` generates a believable made-up Mac (a **demo world**), starts the development build
in sandbox mode on it, lets the app play a **scenario** (clicks, hovers, scrolls, waits and pauses on the real
interface, with a drawn pointer), captures the window with `ffmpeg` and writes `assets/demo/overview.gif` (800 px, 12
frames a second, at most 20 s and 5 MiB) and a full-resolution `.mp4` kept out of git. `npm run demo:check` plays the
same without capturing and fails naming the step that no longer plays. Nothing of the developer's Mac is in the
picture (its own clock, sizes and disk; a neutral path; a check of private texts), and nothing of it is in a release
build.

**No core change**: `crates/core`, `crates/scanner`, `crates/cleaner`, `crates/engine` and `crates/store` are not
touched. The changes are in development code: `crates/sandbox` (the world's header, the scenery kind, the scenario,
the `demo` command), `crates/testing` (one stated fact), the shell's debug-only demo mode, the UI's debug-only player,
a script, fixtures and docs. No new dependency: `ts-rs` (a workspace dependency) is added to `fresher-sandbox`;
`ffmpeg` is a development tool on the developer's Mac, listed in `dependencies.md`.

Branch: `feat/038-demo-recordings`, from `main` 33d51a6c, rebased onto `main` once plans 031, 032, 035, 036 and 037 are
merged, squash-merged once when done ([`../../workflow.md`](../../workflow.md)). Depends on plans
[007](../007-safe-development/readme.md) (sandbox mode, the generator) and [027](../027-scan-screen/readme.md) (the scan
screen). The rebase is mechanical: the only shared lists are the shell's command lists (`lib.rs`,
`dev_runs/commands.rs`), `ipc/index.ts`, `bindings.generated.ts` (regenerated) and the docs.

## Plans

| #   | Plan                                          | Status      | Depends on | What                                                                                    |
| --- | --------------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------------- |
| 01  | [The demo world](./01-world/readme.md)        | 🔨 in progress | —          | the demo header and scenery kind, stated sizes and volume, the first world and its test |
| 02  | [The player](./02-player/readme.md)           | 🔨 in progress | 01         | the scenario format and `demo` command, the shell's demo mode, the UI's player          |
| 03  | [The recorder](./03-recorder/readme.md)       | 🔨 in progress | 02         | `demo:check`, `demo:record`, capture, encoding, the budget, the GIF's size rule         |
| 04  | [Scenarios and finish](./04-finish/readme.md) | 🔨 in progress | all        | `overview` and `cleanup`, the docs, the conclusions                                     |

### Order

```
01/01 ── 01/02 ── 01/03 ── 02/01 ── 02/02 ── 02/03 ── 03/01 ── 04/01 ── 04/02
```

One step at a time: each builds on the types of the one before, and the workspace stays green after each (`cargo
nextest`, `vitest`, `node --test scripts/*.test.ts`).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI.
- `git diff 33d51a6c -- crates/core crates/scanner crates/cleaner crates/engine crates/store` is empty (after the
  rebase: against the rebased base).
- Every demo world generates and, scanned in sandbox mode with every module, offers each category of its `shows`.
- Every shipped scenario parses and names an existing world; each refusal of the format has a test.
- `npm run build --workspace app/ui` leaves no player code in `app/ui/dist` (no `demo-player` marker); the shell's
  demo module is `#[cfg(debug_assertions)]`.
- By hand (the user): `npm run demo:check -- all` passes; `npm run demo:record -- overview` writes a GIF within budget
  that shows no private path.

## Ideas this touches

"Demo mode and automatic screenshots" of [best practices](../../ideas/best-practices.md), 27: the recordings are built;
automatic screenshots stay an idea. The demo world is also what plan [014](../014-onboarding/readme.md)'s "sample data"
needs.

## Risks

- **`avfoundation` screen capture on macOS 27** is unverified (no screen was recorded while planning): the capture is
  one function of the recorder, switchable to `screencapture -v`.
- **`tauri dev` start-up and teardown**: the recorder waits for the window's file and the result's file, never for
  the process's exit code; a 120 s timeout ends a hung run.
- **A GIF over 5 MiB**: the budget refuses it; a shorter scenario or a smaller window is the fix.
- **Wording changes** (plan 039) break targets: by design, `demo:check` names the step.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken.
- 2026-11-22 — settled while planning (agent's calls, to check): the app shows allocated sizes, so a demo world states
  them (`sizes = "as-declared"`, a new stated fact) instead of relying on sparse files; it states its volume too; the
  GIF gets its own 5 MiB limit in the staged-files guard; capture is `ffmpeg`'s `avfoundation`
  ([`00-design.md`](./00-design.md), agent's calls 4, 7, 8, 9).

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. The real app in sandbox mode is recorded, not a mocked interface.
2. The player is inside the app, debug builds only.
3. Scenarios are data, parsed in Rust, typed for the UI.
4. The GIF is committed (up to 5 MiB), the video is not.
5. The build banner is hidden while a demo plays.
6. `ffmpeg` only.
7. Capture with `avfoundation`, the whole main screen, cropped afterwards.
8. A stated `allocated` size for a demo world's sparse files.
9. A stated volume for a demo world.
10. The world lives in a new `/private/tmp/fresher-demo/<scenario>-<n>` on every run; nothing is removed.
11. The page's text is checked for the real home's and the sandbox's paths.
12. Two files at the sandbox's root carry the window's rectangle and the result.
13. The demo commands are in the one IPC module.
14. The window at (80, 80), vibrancy kept.
15. Budget: 800 px, 12 fps, 20 s, 5 MiB; the length checked by `demo:check` too.

## Conclusions

**Status: 🔨 in progress, awaiting the user's walk.** Built on the branch `feat/038-demo-recordings` in four plans and
nine tasks; every automated check passes; nothing was played or captured (it needs a window and the screen).

- **Built**: the demo world (`[demo]` header, `scenery` kind, a stated allocated size, a stated volume, `developer-mac`:
  58 files, about 144 GB declared, a 460 GiB disk), the scenario format and `fresher-sandbox demo`, the shell's demo
  mode (debug builds), the UI's player (a development-only chunk), `npm run demo:check` and `npm run demo:record`, the
  GIF's 5 MiB limit, two scenarios, and the docs.
- **Numbers**: sandbox 242 tests, testing 499, the app's demo, sandbox and bindings tests 122, UI 1276 (98.5 % lines),
  scripts 113; Rust coverage of the touched crates 94 to 99 %. The release bundle has no `data-demo-player`. No core
  change (`git diff` of core, scanner, cleaner, engine and store against the base is empty). One new dependency edge
  (`ts-rs` in `fresher-sandbox`); `ffmpeg` is a development tool.
- **Changed from the design**: the demo folder is new on every run and nothing is removed (only the cleaner deletes:
  the controller's ruling, call 10); the player refuses a permanent delete, `Delete` or `Remove` at run time and a test
  traces every target to a text; the finder prefers the page over the sidebar; the app's manifest and capability list
  the two commands in every build.
- **Calls taken while building**: the scenarios open no category (every "Details" button shares one name: an
  accessibility gap for the user's review); the cleanup scenario ends the put-back with a pause, not a wait.
- **Left to the user**: the walks 1.21 and 2.19 of [`../../manual-testing.md`](../../manual-testing.md), the facts not
  verified and the calls in [`../../deferred/user-review-2026-11-22.md`](../../deferred/user-review-2026-11-22.md).
