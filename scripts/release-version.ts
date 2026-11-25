import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// The release version (plan 013/03/02): one version in every place that holds it — the workspace's `Cargo.toml`, both
// `package.json`, `app/tauri/tauri.conf.json`, and the workspace's own entries of `package-lock.json` and
// `Cargo.lock`. `npm run check:version` fails when they differ; `npm run check:version -- 0.2.0` also wants that
// version, its changelog section and its release notes (the release workflow runs it with the tag's version);
// `npm run release:version -- 0.2.0` writes a new version everywhere and refreshes the lock files;
// `node scripts/release-version.ts notes 0.2.0` prints the release's body: its notes, then its changelog section.

export type Place = { readonly name: string; readonly version: string | undefined };

// Runs a program in the repository's root; injected by the tests.
export type Run = (program: string, args: readonly string[]) => void;

export const CARGO_TOML = 'Cargo.toml';
export const PACKAGE_JSON = 'package.json';
export const TAURI_CONF = 'app/tauri/tauri.conf.json';
export const NPM_LOCK = 'package-lock.json';
export const CARGO_LOCK = 'Cargo.lock';
export const CHANGELOG = 'docs/changelog.md';

export const releaseNotesOf = (version: string): string => `docs/releases/${version}.md`;

const USAGE =
  'usage: npm run check:version [-- <version>] | npm run release:version -- <version> | release-version.ts notes <version>';

// Plain `MAJOR.MINOR.PATCH`: the tags are `v*.*.*`.
const SEMVER = /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;

export const isVersion = (text: string): boolean => SEMVER.test(text);

// `version = "…"` of `[workspace.package]`: the table runs to the next header.
const WORKSPACE_VERSION = /^(\[workspace\.package\]\n(?:(?!\[)[^\n]*\n)*?version = )"([^"]*)"/mu;

// The top-level `"version"` of a JSON file written by Prettier: two spaces deep.
const JSON_VERSION = /^( {2}"version": )"([^"]*)"/mu;

export const cargoWorkspaceVersion = (text: string): string | undefined => WORKSPACE_VERSION.exec(text)?.[2];

export const jsonVersion = (text: string): string | undefined => JSON_VERSION.exec(text)?.[2];

export const setCargoWorkspaceVersion = (text: string, version: string): string =>
  text.replace(WORKSPACE_VERSION, `$1"${version}"`);

export const setJsonVersion = (text: string, version: string): string => text.replace(JSON_VERSION, `$1"${version}"`);

type NpmLock = {
  readonly packages?: Readonly<Record<string, { readonly version?: string }>>;
  readonly version?: string;
};

// The root and each workspace in `package-lock.json`.
export const npmLockPlaces = (text: string, workspaces: readonly string[]): readonly Place[] => {
  const lock = JSON.parse(text) as NpmLock;

  return [
    { name: `${NPM_LOCK} (root)`, version: lock.version },
    ...['', ...workspaces].map((key) => ({
      name: `${NPM_LOCK} (packages["${key}"])`,
      version: lock.packages?.[key]?.version,
    })),
  ];
};

// The workspace's crates in `Cargo.lock`: the packages without a `source`.
export const cargoLockPlaces = (text: string): readonly Place[] =>
  text
    .split(/^\[\[package\]\]$/mu)
    .slice(1)
    .filter((table) => !/^source = /mu.test(table))
    .map((table) => ({
      name: `${CARGO_LOCK} (${/^name = "([^"]*)"$/mu.exec(table)?.[1] ?? '?'})`,
      version: /^version = "([^"]*)"$/mu.exec(table)?.[1],
    }));

export const versionPlaces = (read: (file: string) => string): readonly Place[] => {
  const manifest = read(PACKAGE_JSON);
  const workspaces = (JSON.parse(manifest) as { readonly workspaces?: readonly string[] }).workspaces ?? [];

  return [
    { name: CARGO_TOML, version: cargoWorkspaceVersion(read(CARGO_TOML)) },
    { name: PACKAGE_JSON, version: jsonVersion(manifest) },
    ...workspaces.map((workspace) => {
      const file = `${workspace}/${PACKAGE_JSON}`;

      return { name: file, version: jsonVersion(read(file)) };
    }),
    { name: TAURI_CONF, version: jsonVersion(read(TAURI_CONF)) },
    ...npmLockPlaces(read(NPM_LOCK), workspaces),
    ...cargoLockPlaces(read(CARGO_LOCK)),
  ];
};

// Every place must hold `expected`, or, without it, the version of the workspace's `Cargo.toml`.
export const versionProblems = (places: readonly Place[], expected?: string): readonly string[] => {
  const wanted = expected ?? places[0]?.version;
  if (wanted === undefined) {
    return [`${CARGO_TOML}: no version in [workspace.package]`];
  }

  return places
    .filter((place) => place.version !== wanted)
    .map((place) => `${place.name}: ${place.version ?? 'no version'}, not ${wanted}`);
};

// A release also needs its changelog section, `## [x.y.z] - YYYY-MM-DD`, and its release notes.
export const releaseProblems = (version: string, changelog: string, hasNotes: boolean): readonly string[] => {
  const problems: string[] = [];
  const heading = new RegExp(`^## \\[${version.replaceAll('.', '\\.')}\\] - \\d{4}-\\d{2}-\\d{2}$`, 'mu');
  if (!heading.test(changelog)) {
    problems.push(`${CHANGELOG}: no section "## [${version}] - YYYY-MM-DD"`);
  }
  if (!hasNotes) {
    problems.push(`${releaseNotesOf(version)}: the release notes are missing`);
  }

  return problems;
};

// The version's changelog section without its heading: from the line after `## [x.y.z] - …` to the next `## `
// heading or the end; `undefined` when the changelog has no such section.
export const changelogSection = (changelog: string, version: string): string | undefined => {
  const lines = changelog.split('\n');
  const start = lines.findIndex((line) => line.startsWith(`## [${version}] - `));
  if (start === -1) {
    return undefined;
  }
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith('## '));

  return (end === -1 ? rest : rest.slice(0, end)).join('\n').trim();
};

// The GitHub release's body: the release notes, then "What's new" with the version's changelog section.
export const releaseBody = (notes: string, section: string, version: string): string =>
  `${notes.trimEnd()}\n\n## What's new in ${version}\n\n${section}\n`;

export const checkVersion = (root: string, expected?: string): readonly string[] => {
  if (expected !== undefined && !isVersion(expected)) {
    return [`${expected} is not a version of the form MAJOR.MINOR.PATCH`];
  }
  const read = (file: string): string => readFileSync(join(root, file), 'utf8');
  const problems = [...versionProblems(versionPlaces(read), expected)];
  if (expected !== undefined) {
    problems.push(...releaseProblems(expected, read(CHANGELOG), existsSync(join(root, releaseNotesOf(expected)))));
  }

  return problems;
};

// Writes the version into the manifests, then lets npm and Cargo write it into their lock files, offline: only the
// workspace's own entries change.
export const setVersion = (root: string, version: string, run: Run): readonly string[] => {
  if (!isVersion(version)) {
    return [`${version} is not a version of the form MAJOR.MINOR.PATCH`];
  }
  const read = (file: string): string => readFileSync(join(root, file), 'utf8');
  const write = (file: string, text: string): void => {
    writeFileSync(join(root, file), text);
  };
  const workspaces = (JSON.parse(read(PACKAGE_JSON)) as { readonly workspaces?: readonly string[] }).workspaces ?? [];
  write(CARGO_TOML, setCargoWorkspaceVersion(read(CARGO_TOML), version));
  for (const file of [PACKAGE_JSON, ...workspaces.map((workspace) => `${workspace}/${PACKAGE_JSON}`), TAURI_CONF]) {
    write(file, setJsonVersion(read(file), version));
  }
  run('npm', ['install', '--package-lock-only', '--ignore-scripts', '--no-audit', '--no-fund', '--offline']);
  run('cargo', ['update', '--workspace', '--offline']);

  return versionProblems(versionPlaces(read), version);
};

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const [mode, version, ...rest] = process.argv.slice(2);
  let problems: readonly string[] | undefined;
  if (mode === 'check' && rest.length === 0) {
    problems = checkVersion(root, version);
  } else if (mode === 'notes' && version !== undefined && rest.length === 0) {
    const read = (file: string): string => readFileSync(join(root, file), 'utf8');
    problems = checkVersion(root, version);
    const section = changelogSection(read(CHANGELOG), version);
    if (problems.length === 0 && section !== undefined) {
      process.stdout.write(releaseBody(read(releaseNotesOf(version)), section, version));
    }
  } else if (mode === 'set' && version !== undefined && rest.length === 0) {
    problems = setVersion(root, version, (program, args) => {
      execFileSync(program, [...args], { cwd: root, stdio: 'inherit' });
    });
  }
  if (problems === undefined) {
    process.stderr.write(`release-version: ${USAGE}\n`);
    process.exitCode = 2;
  } else {
    for (const problem of problems) {
      process.stderr.write(`release-version: ${problem}\n`);
    }
    process.exitCode = problems.length > 0 ? 1 : 0;
  }
}
