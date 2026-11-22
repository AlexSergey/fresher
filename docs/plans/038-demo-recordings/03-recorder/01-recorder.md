# 038/03/01 — The recorder script

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** scripts, repo, docs

`scripts/demo-record.ts`, run as `npm run demo:record -- <scenario|all>` and `npm run demo:check -- <scenario|all>`.
Pure functions build the command plan and read the files; a thin `main` spawns them. Tested with a fake spawner and a
fake file system, as `scripts/sandbox-dev.test.ts` does. **Never run `demo:record` or `demo:check` in this step**:
they start the app's window and capture the screen, which is the user's walk (04/02). The design:
[`../00-design.md`](../00-design.md), section 3.

Read first: `scripts/sandbox-dev.ts` and its test (the `Command`, `Spawner`, `UsageError` shapes to reuse — import
them, do not copy), `scripts/dev.ts`, `scripts/check-staged.ts` and its test, `package.json`'s scripts, `knip.json`,
`.gitattributes`, `.gitignore`, `docs/hacks/README.md` (the `icon.icns` entry's format), 02/01 and 02/02 (as built: the
`demo` command, the two files and their JSON).

## Files

- `scripts/demo-record.ts` (new) — a head comment: what it does, that capture needs the terminal's screen recording
  permission and `ffmpeg`, that check mode needs neither. Exports, each with explicit types:
  - `type Mode = 'check' | 'record'`; `type Parsed = { readonly mode: Mode; readonly target: string }`;
    `parseArgs(argv)`: `[--check] <scenario|all>`, exactly one target, a plain name (`/^[\w-]+$/`), an unknown flag
    refused (`UsageError`, usage `usage: npm run demo:record -- <scenario|all>` / `demo:check`).
  - `scenarioNames(target, list: () => readonly string[])`: `all` → every `fixtures/demos/scenarios/*.toml` stem,
    sorted; a name not in the list refused naming it.
  - Constants: `DEMO_FOLDER = '/private/tmp/fresher-demo'`, `BUDGET = { width: 800, fps: 12, seconds: 20, bytes: 5 *
    1024 * 1024 }`, `CAPTURE_FPS = 30`, `WINDOW_WAIT_MS = 600_000` (the first build may compile), `RUN_TIMEOUT_MS =
    120_000` (from the window's file), `TRIM_LEAD_MS = 500` (the lead-in kept).
  - `generateCommand(name): Command` — `cargo run -q -p fresher-sandbox --locked -- demo <name>`.
  - `appCommand(root, name): Command` — `npm run dev` with `FRESHER_SANDBOX=<the folder generateCommand printed>` and
    `FRESHER_DEMO=<root>/fixtures/demos/scenarios/<name>.toml`.
  - `type WindowRect = { x; y; width; height; scale }` and `readWindow(text): WindowRect` — the JSON of
    `demo-window.json`, every field a finite number, sizes above 0, else an error naming the file.
  - `type Outcome` mirroring `DemoOutcome` (import the generated type from `app/ui/src/ipc/bindings.generated.ts`
    if the root `tsconfig` reaches it; otherwise a local type with the same shape and a comment naming the source)
    and `readResult(text | null): Outcome | { kind: 'missing' }`.
  - `judge(outcome, budget): { ok: true } | { ok: false; message: string }` — `missing` → "the app ended without a
    result"; `failed` → "step N: <reason>"; `finished` over `BUDGET.seconds` → "the scenario took N s, over the 20 s
    budget".
  - `captureCommand(out): Command` — `ffmpeg -hide_banner -loglevel error -y -f avfoundation -capture_cursor 0
    -framerate 30 -i "Capture screen 0:none" -c:v libx264 -preset ultrafast -qp 0 -pix_fmt yuv444p <out>`, stopped by
    writing `q` to its input. One function, so switching to `screencapture -v` later is local (agent's call 7).
  - `encodeCommands(raw, rect, trimSeconds, mp4, palette, gif): readonly Command[]` — in order:
    1. `ffmpeg -y -ss <trim> -i <raw> -vf crop=<w>:<h>:<x>:<y> -c:v libx264 -pix_fmt yuv420p -crf 18 -movflags
       +faststart <mp4>`;
    2. `ffmpeg -y -i <mp4> -vf fps=12,scale=800:-1:flags=lanczos,palettegen=stats_mode=diff <palette>`;
    3. `ffmpeg -y -i <mp4> -i <palette> -lavfi fps=12,scale=800:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle <gif>`.
    The crop uses the rectangle as physical pixels (the capture is at the screen's physical size).
  - `trimSeconds(captureStartedAt, windowSeenAt)`: `max(0, windowSeenAt - captureStartedAt + LEAD_IN - TRIM_LEAD_MS)`
    in seconds, with `LEAD_IN = 1500` (the player's lead-in, named as a constant with a comment pointing to
    `app/ui/src/demo/player.ts`).
  - `checkGif(bytes): { ok: true } | { ok: false; message }` — over `BUDGET.bytes` refused ("the GIF is N MiB, over
    the 5 MiB budget; shorten the scenario").
  - Output paths: `NO_COMMIT/demo/tmp/<name>.mkv`, `NO_COMMIT/demo/tmp/<name>-palette.png`,
    `NO_COMMIT/demo/<name>.mp4`, the GIF written first to `NO_COMMIT/demo/tmp/<name>.gif` and moved to
    `assets/demo/<name>.gif` only when `checkGif` passes.
  - `main` (under `import.meta.main`, the only untested glue, kept short): for each name, stop at the first failure:
    run `generateCommand`; spawn `appCommand` (async, inheriting stdio); poll for `<that folder>/demo-window.json`
    every 100 ms up to `WINDOW_WAIT_MS` (record mode: start the capture as soon as it appears, noting both moments with
    `Date.now()`); wait for the app's process to end, killing it after `RUN_TIMEOUT_MS`; stop the capture; read
    `demo-result.json`; `judge`; in record mode run `encodeCommands`, `checkGif`, move the GIF; print one line per
    scenario: `<name>: played in N s` or `<name>: <message>`. Exit code 0 when every scenario passed, 1 otherwise, 2 on
    a usage error.
- `scripts/demo-record.test.ts` (new) — `node:test`, `describe('demo-record')` with `negative cases` first.
- `package.json` — `"demo:check": "node scripts/demo-record.ts --check"`, `"demo:record": "node
  scripts/demo-record.ts"` (sorted with the others).
- `scripts/check-staged.ts` — `GIF_FOLDER = 'assets/demo/'`, `MAX_GIF_SIZE = 5 * 1024 * 1024`: a `.gif` directly in
  that folder is allowed up to 5 MiB with its own message ("a demo GIF is over 5 MiB"); every other file keeps the
  1 MiB limit. `scripts/check-staged.test.ts`: negatives first (a 6 MiB GIF there refused; a 2 MiB GIF elsewhere
  refused; a 2 MiB PNG in `assets/demo/` refused), then a 4 MiB GIF there allowed.
- `.gitattributes` — `*.gif binary`, `*.mp4 binary`.
- `.gitignore` — nothing new (`NO_COMMIT/` is ignored); check that `assets/demo/` is not ignored.
- `docs/hacks/README.md` — an entry "Demo GIFs up to 5 MiB": what (`assets/demo/*.gif` allowed up to 5 MiB by the
  staged-files guard), why (an animated picture of 20 s at 800 px does not fit 1 MiB), the cost (history grows on each
  re-recording), how to undo (host the GIF as a release asset, plan 040).
- `docs/dependencies.md` — `ffmpeg` in the development tools' table: version 7.1.1 (Homebrew, the one checked on
  2026-11-22), licence GPL-2.0-or-later as built by Homebrew (LGPL-2.1-or-later without `--enable-gpl`), used by
  `npm run demo:record` only, never shipped; installed by the developer (`brew install ffmpeg`), not by the repository.

## Produces

`npm run demo:check` and `npm run demo:record`, for 04/01 and the user's walk.

## Checks

- `scripts/demo-record.test.ts`, negatives first: `parseArgs` refuses no target, two targets, an unknown flag, a name
  with `/`; `scenarioNames` refuses an unknown name; `readWindow` refuses a missing field, a zero size, text that is not
  JSON; `readResult(null)` is `missing`; `judge` fails `missing`, `failed` (the message names the step), and a
  `finished` of 20 001 ms; `checkGif` refuses 5 MiB + 1. Then: `all` lists the stems sorted; `check` mode's plan is the
  generator then the app, with the exact environment; `encodeCommands` for a rect `{x: 160, y: 160, width: 2200,
  height: 1440, scale: 2}` gives the exact three argument lists; `trimSeconds` for known moments; `judge` passes a
  `finished` of 12 000 ms; `checkGif` passes 5 MiB exactly.
- `npm run test:scripts`; `npm run lint:ts`; `npm run lint:code`; `npm run knip` (the new script is an entry by
  `scripts/*.ts`).
- `node scripts/demo-record.ts --check` with no argument prints the usage and exits 2 (safe: it starts nothing).

## Conclusions

Built as briefed, with these differences:

- `appCommand` takes the world folder as a third argument; `parseGenerated` reads it from the generator's output line.
- `Outcome` is a local type (the root tsconfig does not reach `app/ui`).
- The app is started in its own process group and the whole group is signalled on every exit path (`killGroup`), since
  the app is the end of a chain of processes.
- `pollWindow` re-reads a half-written `demo-window.json` before reporting an error.
- `judge` returns the played seconds on success; a capture tool that cannot start stops the app and is reported.
- `main`/`play` is untested glue and was never run; the user's walk (04/02) is its first run. Tests: 40 in
  `scripts/demo-record.test.ts` and `scripts/check-staged.test.ts` cover the pure parts.
