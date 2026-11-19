import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { isAbsolute, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// `npm run dev:sandbox -- <feature|path> [--real-trash]` (plan 007/02/04): starts the development build in sandbox mode.
// A bare feature name means `sandbox/<feature>/`, generated first when it is missing; anything with a path separator is
// a folder used as it is. The app itself checks the folder (app/tauri/src/sandbox.rs). The run is recorded (`npm run dev`, plan 026).

export type Command = {
  readonly args: readonly string[];
  // A value of `undefined` removes the variable from the child's environment.
  readonly env: Readonly<Record<string, string | undefined>>;
  readonly program: string;
};

export type Parsed = {
  readonly realTrash: boolean;
  readonly target: string;
};

export type Spawner = (command: Command) => number;

const USAGE = 'usage: npm run dev:sandbox -- <feature|path> [--real-trash]';

export class UsageError extends Error {}

const FEATURE_NAME = /^[\w-]+$/;

const isPath = (target: string): boolean => target.includes('/') || target.includes('\\');

export const parseArgs = (argv: readonly string[]): Parsed => {
  let realTrash = false;
  const targets: string[] = [];

  for (const arg of argv) {
    if (arg === '--real-trash') {
      realTrash = true;
    } else if (arg.startsWith('-')) {
      throw new UsageError(`unknown flag ${arg}\n${USAGE}`);
    } else {
      targets.push(arg);
    }
  }

  const [target, ...extra] = targets;
  if (target === undefined || extra.length > 0) {
    throw new UsageError(`expected exactly one feature or path\n${USAGE}`);
  }
  if (!isPath(target) && !FEATURE_NAME.test(target)) {
    throw new UsageError(`${target} is not a feature name (letters, digits, _ and -) or a path\n${USAGE}`);
  }

  return { realTrash, target };
};

export const sandboxFolder = (target: string, root: string, cwd: string): string =>
  isPath(target) ? (isAbsolute(target) ? target : resolve(cwd, target)) : join(root, 'sandbox', target);

// The commands to run, in order: the generator when a feature's folder is missing, then the development build.
export const plan = (
  parsed: Parsed,
  root: string,
  cwd: string,
  exists: (path: string) => boolean,
): readonly Command[] => {
  const folder = sandboxFolder(parsed.target, root, cwd);
  const commands: Command[] = [];

  if (!isPath(parsed.target) && !exists(folder)) {
    commands.push({ args: ['run', 'sandbox', '--', parsed.target], env: {}, program: 'npm' });
  }
  commands.push({
    args: ['run', 'dev'],
    env: { FRESHER_SANDBOX: folder, FRESHER_SANDBOX_TRASH: parsed.realTrash ? 'real' : undefined },
    program: 'npm',
  });

  return commands;
};

// Runs the commands until one fails; returns the exit code.
export const run = (commands: readonly Command[], spawn: Spawner): number => {
  for (const command of commands) {
    const code = spawn(command);
    if (code !== 0) {
      return code;
    }
  }

  return 0;
};

const withoutUndefined = (env: Record<string, string | undefined>): Record<string, string> =>
  Object.fromEntries(Object.entries(env).filter((entry): entry is [string, string] => entry[1] !== undefined));

export const spawnReal: Spawner = (command) =>
  spawnSync(command.program, [...command.args], {
    env: withoutUndefined({ ...process.env, ...command.env }),
    stdio: 'inherit',
  }).status ?? 1;

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  try {
    const commands = plan(parseArgs(process.argv.slice(2)), root, process.env.INIT_CWD ?? process.cwd(), existsSync);
    process.exitCode = run(commands, spawnReal);
  } catch (error) {
    if (!(error instanceof UsageError)) {
      throw error;
    }
    process.stderr.write(`sandbox-dev: ${error.message}\n`);
    process.exitCode = 2;
  }
}
