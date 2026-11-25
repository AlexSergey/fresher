import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// The lock check (docs/restrictions/dependencies.md, "Locked sources"): every package of `package-lock.json` comes
// from the npm registry with an `integrity`, or is one of the root's workspaces; every package of `Cargo.lock` with a
// source comes from crates.io with a checksum (a package without one is a workspace crate). `npm ci` installs from
// the `resolved` it is given and trusts its `integrity`, so an edited lock file could point anywhere.

export const NPM_REGISTRY = 'https://registry.npmjs.org/';
export const CRATES_IO = ['registry+https://github.com/rust-lang/crates.io-index', 'sparse+https://index.crates.io/'];

export type LockProblem = { readonly lock: 'Cargo.lock' | 'package-lock.json'; readonly message: string };

type NpmEntry = {
  readonly integrity?: unknown;
  readonly link?: unknown;
  readonly resolved?: unknown;
};

type NpmLock = {
  readonly lockfileVersion?: unknown;
  readonly packages?: Readonly<Record<string, NpmEntry>>;
};

const npm = (message: string): LockProblem => ({ lock: 'package-lock.json', message });
const cargo = (message: string): LockProblem => ({ lock: 'Cargo.lock', message });

export const npmLockProblems = (text: string, workspaces: readonly string[]): readonly LockProblem[] => {
  const lock = JSON.parse(text) as NpmLock;
  if (lock.lockfileVersion !== 3 || lock.packages === undefined) {
    return [npm('not a lock file of version 3 with `packages`')];
  }
  const problems: LockProblem[] = [];
  for (const [key, entry] of Object.entries(lock.packages)) {
    if (key === '') {
      continue;
    }
    if (!key.startsWith('node_modules/') && !key.includes('/node_modules/')) {
      if (!workspaces.includes(key)) {
        problems.push(npm(`${key}: a local package that is not a workspace of the root`));
      }
      continue;
    }
    if (entry.link === true) {
      if (typeof entry.resolved !== 'string' || !workspaces.includes(entry.resolved)) {
        problems.push(npm(`${key}: a link to ${String(entry.resolved)}, which is not a workspace of the root`));
      }
      continue;
    }
    if (typeof entry.resolved !== 'string' || !entry.resolved.startsWith(NPM_REGISTRY)) {
      problems.push(npm(`${key}: resolved from ${String(entry.resolved)}, not ${NPM_REGISTRY}`));
    }
    if (typeof entry.integrity !== 'string' || !entry.integrity.startsWith('sha512-')) {
      problems.push(npm(`${key}: no sha512 integrity`));
    }
  }

  return problems;
};

// One `[[package]]` table of `Cargo.lock`: its `key = "value"` lines.
const cargoPackages = (text: string): readonly Readonly<Record<string, string>>[] =>
  text
    .split(/^\[\[package\]\]$/mu)
    .slice(1)
    .map((table) =>
      Object.fromEntries(
        [...table.matchAll(/^(\w+) = "([^"]*)"$/gmu)].map((match) => [match[1] ?? '', match[2] ?? '']),
      ),
    );

export const cargoLockProblems = (text: string): readonly LockProblem[] => {
  const problems: LockProblem[] = [];
  for (const table of cargoPackages(text)) {
    const name = `${table.name ?? '?'} ${table.version ?? '?'}`;
    if (table.source === undefined) {
      continue;
    }
    if (!CRATES_IO.includes(table.source)) {
      problems.push(cargo(`${name}: from ${table.source}, not crates.io`));
    }
    if (table.checksum === undefined || !/^[0-9a-f]{64}$/u.test(table.checksum)) {
      problems.push(cargo(`${name}: no checksum`));
    }
  }

  return problems;
};

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const read = (name: string): string => readFileSync(join(root, name), 'utf8');
  const manifest = JSON.parse(read('package.json')) as { readonly workspaces?: readonly string[] };
  const problems = [
    ...npmLockProblems(read('package-lock.json'), manifest.workspaces ?? []),
    ...cargoLockProblems(read('Cargo.lock')),
  ];
  for (const problem of problems) {
    process.stderr.write(`check-locks: ${problem.lock}: ${problem.message}\n`);
  }
  process.exitCode = problems.length > 0 ? 1 : 0;
}
