import { type ChildProcess, spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, renameSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';

import { type Command, UsageError } from './sandbox-dev.ts';

// `npm run demo:record -- <scenario|all>` and `npm run demo:check -- <scenario|all>` (plan 038/03/01): plays a demo
// scenario in the development build and records the app's window into a GIF. `check` plays it without recording.
// Per scenario: `fresher-sandbox demo <name>` claims a new world folder and prints it; the app starts in development
// mode (the player exists only there) with that folder and the scenario; the shell writes `demo-window.json` when the
// window is placed and `demo-result.json` when the scenario ends; ffmpeg crops the capture to the window.
// Record mode needs `ffmpeg` and the terminal's screen recording permission; check mode needs neither.
// Nothing is ever deleted here. The one deliberate overwrite of a committed file is `assets/demo/<name>.gif`, replaced
// by a new recording that passed the budget; the world folders under /private/tmp stay, and the script's own outputs
// under NO_COMMIT/demo and tmp/ (`ffmpeg -y`) are rewritten on every run.

export type Mode = 'check' | 'record';
// Mirrors `DemoOutcome` in app/ui/src/ipc/bindings.generated.ts (generated from the shell's demo.rs); the root
// tsconfig does not reach app/ui, so the shape is repeated here.
export type Outcome =
  | { readonly elapsedMs: number; readonly kind: 'finished' }
  | { readonly kind: 'failed'; readonly reason: string; readonly step: number };

export type Parsed = { readonly mode: Mode; readonly target: string };

export type Verdict = { readonly message: string; readonly ok: false } | { readonly ok: true };

export type WindowRect = {
  readonly height: number;
  readonly scale: number;
  readonly width: number;
  readonly x: number;
  readonly y: number;
};

export const DEMO_FOLDER = '/private/tmp/fresher-demo';
export const BUDGET = { bytes: 5 * 1024 * 1024, fps: 12, seconds: 20, width: 800 } as const;
export const CAPTURE_FPS = 30;
export const WINDOW_WAIT_MS = 600_000;
export const RUN_TIMEOUT_MS = 120_000;
export const TRIM_LEAD_MS = 500;
// The capture keeps this long after the result is seen: the app is stopped right after it, and what the screen shows
// then is not the app.
export const TAIL_MARGIN_MS = 200;
// The player's lead-in before the first step (LEAD_IN_MS in app/ui/src/demo/player.ts).
const LEAD_IN = 1500;
const MIB = 1024 * 1024;

const USAGE_RECORD = 'usage: npm run demo:record -- <scenario|all>';
const USAGE_CHECK = 'usage: npm run demo:check -- <scenario|all>';
const SCENARIO_NAME = /^[\w-]+$/;

export const parseArgs = (argv: readonly string[]): Parsed => {
  let mode: Mode = 'record';
  const targets: string[] = [];

  for (const arg of argv) {
    if (arg === '--check') {
      mode = 'check';
    } else if (arg.startsWith('-')) {
      throw new UsageError(`unknown flag ${arg}\n${mode === 'check' ? USAGE_CHECK : USAGE_RECORD}`);
    } else {
      targets.push(arg);
    }
  }

  const usage = mode === 'check' ? USAGE_CHECK : USAGE_RECORD;
  const [target, ...extra] = targets;
  if (target === undefined || extra.length > 0) {
    throw new UsageError(`expected exactly one scenario or all\n${usage}`);
  }
  if (!SCENARIO_NAME.test(target)) {
    throw new UsageError(`${target} is not a scenario name (letters, digits, _ and -)\n${usage}`);
  }

  return { mode, target };
};

export const scenarioNames = (target: string, list: () => readonly string[]): readonly string[] => {
  const known = list();
  if (target === 'all') {
    return [...known].sort();
  }
  if (!known.includes(target)) {
    throw new UsageError(`unknown scenario ${target} (known: ${[...known].sort().join(', ')})`);
  }

  return [target];
};

export const generateCommand = (name: string): Command => ({
  args: ['run', '-q', '-p', 'fresher-sandbox', '--locked', '--', 'demo', name],
  env: {},
  program: 'cargo',
});

// The folder `fresher-sandbox demo` printed: "wrote the demo world <world> into <folder>".
export const parseGenerated = (stdout: string): string => {
  const match = /^wrote the demo world \S+ into (.+)$/m.exec(stdout);
  const folder = match?.[1]?.trim();
  if (folder === undefined || folder === '') {
    throw new Error('the generator did not print the demo folder');
  }

  return folder;
};

export const appCommand = (root: string, name: string, folder: string): Command => ({
  args: ['run', 'dev'],
  env: {
    FRESHER_DEMO: join(root, 'fixtures', 'demos', 'scenarios', `${name}.toml`),
    FRESHER_SANDBOX: folder,
    // A demo plays on the sandbox's own Trash, whatever the developer's shell exports.
    FRESHER_SANDBOX_TRASH: undefined,
  },
  program: 'npm',
});

const isPositive = (value: unknown): boolean => typeof value === 'number' && Number.isFinite(value) && value > 0;
const isFiniteNumber = (value: unknown): boolean => typeof value === 'number' && Number.isFinite(value);

export const readWindow = (text: string): WindowRect => {
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    throw new Error('demo-window.json is not JSON');
  }
  const record = typeof data === 'object' && data !== null ? (data as Record<string, unknown>) : {};
  const { height, scale, width, x, y } = record;
  if (!isFiniteNumber(x) || !isFiniteNumber(y) || !isPositive(width) || !isPositive(height) || !isPositive(scale)) {
    throw new Error('demo-window.json needs finite x and y and positive width, height and scale');
  }

  return { height, scale, width, x, y } as WindowRect;
};

export const readResult = (text: null | string): Outcome | { readonly kind: 'missing' } => {
  if (text === null) {
    return { kind: 'missing' };
  }
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    return { kind: 'missing' };
  }
  const record = typeof data === 'object' && data !== null ? (data as Record<string, unknown>) : {};
  if (record.kind === 'finished' && isFiniteNumber(record.elapsedMs)) {
    return { elapsedMs: record.elapsedMs as number, kind: 'finished' };
  }
  if (record.kind === 'failed' && typeof record.reason === 'string' && isFiniteNumber(record.step)) {
    return { kind: 'failed', reason: record.reason, step: record.step as number };
  }

  return { kind: 'missing' };
};

export const judge = (
  outcome: Outcome | { readonly kind: 'missing' },
  budget: typeof BUDGET,
): { readonly message: string; readonly ok: false } | { readonly ok: true; readonly seconds: number } => {
  if (outcome.kind === 'missing') {
    return { message: 'the app ended without a result', ok: false };
  }
  if (outcome.kind === 'failed') {
    return { message: `step ${String(outcome.step)}: ${outcome.reason}`, ok: false };
  }
  if (outcome.elapsedMs > budget.seconds * 1000) {
    return {
      message: `the scenario took ${String(Math.round(outcome.elapsedMs / 1000))} s, over the ${String(budget.seconds)} s budget`,
      ok: false,
    };
  }

  return { ok: true, seconds: Math.round(outcome.elapsedMs / 1000) };
};

// One function, so switching to `screencapture -v` later stays local. Stopped by writing `q` to its input.
export const captureCommand = (out: string): Command => ({
  args: [
    '-hide_banner',
    '-loglevel',
    'error',
    '-y',
    '-f',
    'avfoundation',
    '-capture_cursor',
    '0',
    '-framerate',
    String(CAPTURE_FPS),
    '-i',
    'Capture screen 0:none',
    '-c:v',
    'libx264',
    '-preset',
    'ultrafast',
    '-qp',
    '0',
    '-pix_fmt',
    'yuv444p',
    out,
  ],
  env: {},
  program: 'ffmpeg',
});

const ffmpeg = (args: readonly string[]): Command => ({ args: ['-y', ...args], env: {}, program: 'ffmpeg' });

// The rectangle is in physical pixels: the capture is at the screen's physical size.
export const encodeCommands = (
  raw: string,
  rect: WindowRect,
  trim: number,
  keep: number,
  mp4: string,
  palette: string,
  gif: string,
): readonly Command[] => {
  const scaling = `fps=${String(BUDGET.fps)},scale=${String(BUDGET.width)}:-1:flags=lanczos`;

  return [
    ffmpeg([
      '-ss',
      trim.toFixed(3),
      '-i',
      raw,
      '-t',
      keep.toFixed(3),
      '-vf',
      `crop=${String(rect.width)}:${String(rect.height)}:${String(rect.x)}:${String(rect.y)}`,
      '-c:v',
      'libx264',
      '-pix_fmt',
      'yuv420p',
      '-crf',
      '18',
      '-movflags',
      '+faststart',
      mp4,
    ]),
    ffmpeg(['-i', mp4, '-vf', `${scaling},palettegen=stats_mode=diff`, palette]),
    ffmpeg([
      '-i',
      mp4,
      '-i',
      palette,
      '-lavfi',
      `${scaling}[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle`,
      gif,
    ]),
  ];
};

// Seconds to cut from the start of the capture: everything before the lead-in, keeping TRIM_LEAD_MS of it.
export const trimSeconds = (captureStartedAt: number, windowSeenAt: number): number =>
  Math.max(0, windowSeenAt - captureStartedAt + LEAD_IN - TRIM_LEAD_MS) / 1000;

// Seconds to keep after the head trim: the capture up to the moment the result was seen, plus the margin. The capture
// runs on until the app is stopped, so without this the GIF ends on whatever is on the screen then.
export const keepSeconds = (captureStartedAt: number, resultSeenAt: number, trim: number): number =>
  Math.max(0, (resultSeenAt - captureStartedAt + TAIL_MARGIN_MS) / 1000 - trim);

export const checkGif = (bytes: number): Verdict =>
  bytes > BUDGET.bytes
    ? {
        message: `the GIF is ${(bytes / MIB).toFixed(1)} MiB, over the ${String(BUDGET.bytes / MIB)} MiB budget; shorten the scenario`,
        ok: false,
      }
    : { ok: true };

// The app runs as a chain (npm, node, npm exec, tauri dev, the app), so it is started in its own process group and
// the whole group is signalled. `signal` is `process.kill`; a group that is already gone is not an error.
export type Signal = (pid: number, signal: 'SIGTERM') => void;

export const killGroup = (pid: number | undefined, signal: Signal): void => {
  if (pid === undefined) {
    return;
  }
  try {
    signal(-pid, 'SIGTERM');
  } catch {
    // the group has already ended
  }
};

export const WINDOW_PARSE_TRIES = 20;

// Waits for demo-window.json: `read` gives its text or null while the file is missing; a file that does not parse yet
// (half written) is read again, up to `tries` times. `over` says the app has ended or the wait ran out.
export const pollWindow = async (
  read: () => null | string,
  pause: () => Promise<void>,
  over: () => boolean,
  tries: number = WINDOW_PARSE_TRIES,
): Promise<null | WindowRect> => {
  let failures = 0;
  while (!over()) {
    const text = read();
    if (text !== null) {
      try {
        return readWindow(text);
      } catch (error) {
        failures += 1;
        if (failures >= tries) {
          throw error;
        }
      }
    }
    await pause();
  }

  return null;
};

const readIfExists = (path: string): null | string => (existsSync(path) ? readFileSync(path, 'utf8') : null);

const toSpawnEnv = (command: Command): Record<string, string> =>
  Object.fromEntries(
    Object.entries({ ...process.env, ...command.env }).filter(
      (entry): entry is [string, string] => entry[1] !== undefined,
    ),
  );

const exited = (child: ChildProcess): Promise<number> =>
  new Promise((resolve) => {
    child.once('exit', (code) => {
      resolve(code ?? 1);
    });
    child.once('error', () => {
      resolve(1);
    });
  });

const runSync = (command: Command, cwd: string): number =>
  spawnSync(command.program, [...command.args], { cwd, env: toSpawnEnv(command), stdio: 'inherit' }).status ?? 1;

type PlayResult = { readonly message: string; readonly ok: false } | { readonly ok: true; readonly seconds: number };

type Recording = {
  readonly captureStartedAt: number;
  readonly raw: string;
  readonly rect: WindowRect;
  readonly resultSeenAt: number;
  readonly seconds: number;
  readonly windowSeenAt: number;
};

// Encodes the capture and moves the GIF into assets/demo/ only when it is within the budget.
const finish = (name: string, root: string, recording: Recording): PlayResult => {
  const out = join(root, 'NO_COMMIT', 'demo');
  const tmp = join(out, 'tmp');
  const paths = {
    gif: join(tmp, `${name}.gif`),
    mp4: join(out, `${name}.mp4`),
    palette: join(tmp, `${name}-palette.png`),
  };
  const trim = trimSeconds(recording.captureStartedAt, recording.windowSeenAt);
  const keep = keepSeconds(recording.captureStartedAt, recording.resultSeenAt, trim);
  const commands = encodeCommands(recording.raw, recording.rect, trim, keep, paths.mp4, paths.palette, paths.gif);
  for (const command of commands) {
    if (runSync(command, root) !== 0) {
      return { message: 'ffmpeg failed while encoding', ok: false };
    }
  }
  const size = checkGif(statSync(paths.gif).size);
  if (!size.ok) {
    return size;
  }
  mkdirSync(join(root, 'assets', 'demo'), { recursive: true });
  renameSync(paths.gif, join(root, 'assets', 'demo', `${name}.gif`));

  return { ok: true, seconds: recording.seconds };
};

const play = async (name: string, mode: Mode, root: string): Promise<PlayResult> => {
  const generated = generateCommand(name);
  const made = spawnSync(generated.program, [...generated.args], {
    cwd: root,
    encoding: 'utf8',
    env: toSpawnEnv(generated),
  });
  if (made.status !== 0) {
    return { message: `the generator failed (${made.stderr.trim().split('\n').pop() ?? 'no output'})`, ok: false };
  }
  const folder = parseGenerated(made.stdout);

  const out = join(root, 'NO_COMMIT', 'demo');
  const tmp = join(out, 'tmp');
  mkdirSync(tmp, { recursive: true });
  const raw = join(tmp, `${name}.mkv`);

  const app = appCommand(root, name, folder);
  const appProcess = spawn(app.program, [...app.args], {
    cwd: root,
    detached: true,
    env: toSpawnEnv(app),
    stdio: 'inherit',
  });
  const stopApp = (): void => {
    killGroup(appProcess.pid, (pid, signal) => process.kill(pid, signal));
  };
  const appEnded = exited(appProcess);
  const state = { ended: false };
  void appEnded.then(() => {
    state.ended = true;
  });

  let capture: ChildProcess | null = null;
  try {
    const windowFile = join(folder, 'demo-window.json');
    const waitUntil = Date.now() + WINDOW_WAIT_MS;
    const rect = await pollWindow(
      () => readIfExists(windowFile),
      () => sleep(100),
      () => state.ended || Date.now() >= waitUntil,
    );
    if (rect === null) {
      return { message: 'the app never placed its window', ok: false };
    }
    const windowSeenAt = Date.now();

    let captureStartedAt = 0;
    const captureError = { message: null as null | string };
    if (mode === 'record') {
      const command = captureCommand(raw);
      captureStartedAt = Date.now();
      capture = spawn(command.program, [...command.args], { stdio: ['pipe', 'inherit', 'inherit'] });
      capture.once('error', () => {
        captureError.message = `could not start ${command.program} (is it installed? brew install ffmpeg)`;
        stopApp();
      });
    }

    const timer = setTimeout(stopApp, RUN_TIMEOUT_MS);
    const resultFile = join(folder, 'demo-result.json');
    let resultSeenAt = Date.now();
    let resultSeen = false;
    const watching = setInterval(() => {
      if (!resultSeen && readResult(readIfExists(resultFile)).kind !== 'missing') {
        resultSeen = true;
        resultSeenAt = Date.now();
      }
    }, 50);
    await appEnded;
    clearInterval(watching);
    clearTimeout(timer);
    if (capture !== null && captureError.message === null) {
      capture.stdin?.write('q');
      capture.stdin?.end();
      await exited(capture);
    }
    if (captureError.message !== null) {
      return { message: captureError.message, ok: false };
    }

    const verdict = judge(readResult(readIfExists(resultFile)), BUDGET);
    if (!verdict.ok || mode === 'check') {
      return verdict;
    }

    return finish(name, root, { captureStartedAt, raw, rect, resultSeenAt, seconds: verdict.seconds, windowSeenAt });
  } catch (error) {
    return { message: error instanceof Error ? error.message : String(error), ok: false };
  } finally {
    stopApp();
    capture?.kill();
  }
};

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  try {
    const parsed = parseArgs(process.argv.slice(2));
    const scenarios = join(root, 'fixtures', 'demos', 'scenarios');
    const names = scenarioNames(parsed.target, () =>
      readdirSync(scenarios)
        .filter((file) => file.endsWith('.toml'))
        .map((file) => file.slice(0, -'.toml'.length)),
    );
    let code = 0;
    for (const name of names) {
      const result = await play(name, parsed.mode, root);
      process.stdout.write(`${name}: ${result.ok ? `played in ${String(result.seconds)} s` : result.message}\n`);
      if (!result.ok) {
        code = 1;
        break;
      }
    }
    process.exitCode = code;
  } catch (error) {
    if (!(error instanceof UsageError)) {
      throw error;
    }
    process.stderr.write(`demo-record: ${error.message}\n`);
    process.exitCode = 2;
  }
}
