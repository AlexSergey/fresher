import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { type Command, run, spawnReal, UsageError } from './sandbox-dev.ts';

// `npm run dev` (plan 026/02/03): starts the development build with the development run record on:
// `FRESHER_DEV_RUNS`, `FRESHER_BUILD_COMMIT`, `FRESHER_BUILD_DIRTY`. The app checks the folder (app/tauri/src/dev_runs/).
// It takes no argument: `npm run dev -- --release` would build an app that really removes and records nothing. The bare
// `npm run tauri` script still forwards anything, but such a build runs with the development identifier and refuses to
// start (app/tauri/src/identity.rs).

export type BuildFacts = { readonly commit: string; readonly dirty: boolean };

export type Git = (args: readonly string[]) => { readonly status: number; readonly stdout: string };

const USAGE = 'usage: npm run dev (no arguments)';

export const buildFacts = (git: Git): BuildFacts => {
  const head = git(['rev-parse', '--short', 'HEAD']);
  const status = git(['status', '--porcelain']);

  return {
    commit: head.status === 0 ? (head.stdout.split('\n')[0]?.trim() ?? 'unknown') || 'unknown' : 'unknown',
    dirty: status.status === 0 && status.stdout.trim() !== '',
  };
};

export const devCommand = (root: string, facts: BuildFacts, argv: readonly string[]): Command => {
  if (argv.length > 0) {
    throw new UsageError(`unexpected argument ${argv.join(' ')}\n${USAGE}`);
  }

  return {
    args: ['exec', '--', 'tauri', 'dev', '--config', 'app/tauri/tauri.dev.conf.json'],
    env: {
      FRESHER_BUILD_COMMIT: facts.commit,
      FRESHER_BUILD_DIRTY: facts.dirty ? '1' : '0',
      FRESHER_DEV_RUNS: join(root, 'NO_COMMIT', 'dev-runs'),
    },
    program: 'npm',
  };
};

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const git: Git = (args) => {
    const result = spawnSync('git', [...args], { cwd: root, encoding: 'utf8' });

    return { status: result.status ?? 1, stdout: result.stdout };
  };
  try {
    process.exitCode = run([devCommand(root, buildFacts(git), process.argv.slice(2))], spawnReal);
  } catch (error) {
    if (!(error instanceof UsageError)) {
      throw error;
    }
    process.stderr.write(`dev: ${error.message}\n`);
    process.exitCode = 2;
  }
}
