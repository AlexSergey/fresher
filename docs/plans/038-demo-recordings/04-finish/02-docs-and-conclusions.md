# 038/04/02 — Docs and conclusions

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** docs

The docs the rules ask for, the runs by hand written as the user's checklist, the plan's conclusions, and the plan's
section in the user's review list. **Do not run `demo:check` or `demo:record`**: they open the app's window and
capture the screen; they are the user's walk. The automated checks are run.

Read first: the whole plan (`../readme.md`, `../00-design.md`, every step and its conclusions as built),
`docs/testing.md` ("Sandboxes", "Sandbox mode", "Commands"), `docs/manual-testing.md` (its sections and their format),
`docs/architecture/README.md` ("Crates", "UI"), `docs/ideas/best-practices.md` (item 27), `docs/ideas/README.md`,
`docs/deferred/README.md`, `docs/deferred/user-review-2026-11-22.md` (the format of a plan's section),
`docs/plans/README.md`.

## Files

- `docs/testing.md`:
  - "Sandboxes" / "The case file": the `[demo]` header (`now`, `sizes`, `shows`, `[demo.volume]`), the `scenery` kind
    and its refusals, the stated `allocated` fact and the `[volume]` of `world.toml`, in the format of the section's
    other entries; `fixtures/demos/` in "Where things live".
  - A new subsection "Demo recordings" after "Sandbox mode": the scenario format (steps, targets, waits, the limits),
    demo mode (`FRESHER_DEMO`, the two commands, the two files), the player (lead-in, glide, timeouts, the private-text
    check), `npm run demo:check -- <scenario|all>` (no permission, run before a release and when a scenario or a screen
    it touches changes) and `npm run demo:record -- <scenario|all>` (the screen recording permission for the terminal,
    `ffmpeg`, the outputs and the budget), the demo worlds' scan test (`cargo nextest run -p fresher-app --test demos`),
    and "quit any running development window first" (single-instance hands the launch to it).
  - "Commands": rows for `npm run demo:check` and `npm run demo:record`.
- `docs/manual-testing.md` — a new section "Demo recordings (plan 038)" with the walk below as checkboxes: `### 1.21`
  for the sandbox part (the world by hand, `demo:check`) and `### 2.19` for the recording on the real screen (1.15–1.20
  and 2.13–2.18 are taken by plans 033, 034, 037, 031, 032 and 035).
- `docs/architecture/README.md` — "Crates": `fresher-sandbox` also holds demo worlds and scenarios (dev only, exported
  to the UI's types); "UI": the debug-only player chunk (`app/ui/src/demo/`), never in a release bundle; the shell's
  demo mode beside sandbox mode, debug builds only.
- `docs/ideas/best-practices.md` item 27 and `docs/ideas/README.md` — the recordings are built by plan 038; automatic
  screenshots stay an idea.
- `docs/deferred/` — a new item `demo-scenarios-for-new-screens.md` (with its row in `docs/deferred/README.md`): due when
  a screen of plans 031, 032, 035, 036, 037 or later is wanted in the README or the release notes; what: one scenario
  file per story, the world extended with the module's scenery and its `shows`; nothing else to build.
- `docs/deferred/user-review-2026-11-22.md` — append a section `## 038 — Demo recordings` after the last plan's (the
  file and its row in `docs/deferred/README.md` exist), short bullets:
  - **Walks by hand**:
    - quit any development window; `npm run demo:check -- all`: both scenarios print `played in N s` (N ≤ 20);
    - give the terminal the screen recording permission (System Settings → Privacy & Security → Screen & System Audio
      Recording); set a plain wallpaper (the sidebar's blur shows it); `npm run demo:record -- overview`;
    - look at `assets/demo/overview.gif` and `NO_COMMIT/demo/overview.mp4`: the window only, no cursor of the Mac, the
      drawn pointer, no banner, no real path or name, sizes in gigabytes, the world's disk in the ring, under 5 MiB;
    - commit the GIF if it is good (plan 040 places it in the README);
    - `npm run dev:sandbox -- fixtures/demos/developer-mac.toml`: the world looks believable by hand (the banner shows
      there, as in any sandbox).
  - **Facts not verified**: `avfoundation` screen capture on macOS 27 (if it fails: switch `captureCommand` to
    `screencapture -v`); that `"Capture screen 0"` is the main display's device name; that `tauri dev` ends when the
    app exits; the crop of the window's rectangle (rounded corners, the overlay title bar); the cleanup's result and
    the freed-space line with a stated volume; how long the first build takes before the window appears.
  - **Agent's calls worth a second look**: the GIF committed with a 5 MiB exception (or hosted as a release asset);
    the stated `allocated` fact; the stated volume (not named in the banner); `/private/tmp/fresher-demo` and its
    removal guarded by the marker; the vibrancy kept; the two demo command names in the release bundle's table.
  - **Known gaps**: no scenario for the screens of plans 031–037 yet (deferred); no CI run of `demo:check` (needs a
    window); automatic screenshots not built.
- `docs/plans/038-demo-recordings/readme.md`, `00-design.md` and every plan's readme and step — the **Conclusions**:
  what was built, what changed from the design and why, the numbers (tests added, coverage of Rust and UI, the world's
  declared size and file count, the release bundle check's result), the calls taken while building; status 🔨 in
  progress, "awaiting the user's walk". Each step's own conclusions are written by its implementer; check they exist.
- `docs/plans/README.md` — the row of 038: 🔨 in progress, modules `sandbox, testing, app, app/tauri, app/ui, scripts,
  fixtures, docs`.

## Checks

- `npm run pre-push` on the branch's last commit (lint, crate guard, knip, machete, deny, scripts' tests, coverage
  ≥ 90 % in Rust and the UI).
- `git diff 33d51a6c -- crates/core crates/scanner crates/cleaner crates/engine crates/store` is empty (after the
  rebase: against the rebased base).
- `cargo nextest run -p fresher-app --test demos --test sandboxes --test bindings` passes.
- `npm run build --workspace app/ui && grep -rl "data-demo-player" app/ui/dist` finds nothing.
- Every link in the changed docs resolves; `typos`; Prettier on the changed Markdown.

## Conclusions

Written as briefed: `docs/testing.md` ("The case file", "Demo recordings", "Commands", "Where things live"),
`docs/manual-testing.md` 1.21 and 2.19, `docs/architecture/README.md`, `docs/ideas/best-practices.md` and
`docs/ideas/README.md`, `docs/deferred/demo-scenarios-for-new-screens.md` and its row, the section "038 — Demo
recordings" of `docs/deferred/user-review-2026-11-22.md`, the conclusions of every plan and step, and the row in
`docs/plans/README.md`.

Checks: `git diff 207fe60c -- crates/core crates/scanner crates/cleaner crates/engine crates/store` is empty; the
release bundle of `npm run build --workspace app/ui` has no `data-demo-player`; `cargo nextest run -p fresher-app --test
demos --test sandboxes --test bindings` 122 passed; `npm run pre-push` passed on the last commit. `demo:check` and
`demo:record` were never run: they are the user's walk.
