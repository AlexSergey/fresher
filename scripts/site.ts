import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { parseCatalogue, parseLevels, readCategories } from './readme-catalogue.ts';
import { parseCounterCode, parseRelease, renderDownload, renderIndex, REPOSITORY } from './site-pages.ts';

// `npm run site:build` writes getfresher.app into dist/site (plan 046): site/ with its slots filled from
// docs/categories, the latest release and site/config.json, the images from assets/, and /download. The only
// file-system and network code of the site.
//
//   node scripts/site.ts build [--release <release.json>] [--out <folder>]
//
// Without --release it asks GitHub for the latest release (with $GITHUB_TOKEN when set). $GOATCOUNTER_CODE, when set,
// stands in for site/config.json's code.

const SCRIPT = 'site';
const LATEST_RELEASE = `https://api.github.com/repos/${REPOSITORY}/releases/latest`;

// Published path → source path, relative to the repository's root.
const COPIED: readonly (readonly [string, string])[] = [
  ['styles.css', 'site/styles.css'],
  ['logo.png', 'assets/logo.png'],
  ['cleanup.gif', 'assets/demo/cleanup.gif'],
  ['cleanup-still.png', 'site/cleanup-still.png'],
  ['social-preview.png', 'assets/social-preview.png'],
];

export type BuildOptions = {
  readonly counterOverride: string | undefined;
  readonly out: string;
  readonly release: unknown;
  readonly root: string;
};

const readCategoryFiles = (folder: string): ReadonlyMap<string, string> =>
  new Map(
    readdirSync(folder)
      .filter((name) => name.endsWith('.md') && name !== 'README.md')
      .toSorted()
      .map((name) => [name, readFileSync(path.join(folder, name), 'utf8')]),
  );

const readCounterCode = (root: string, override: string | undefined): string => {
  if (override !== undefined && override !== '') {
    return override;
  }
  const config: unknown = JSON.parse(readFileSync(path.join(root, 'site', 'config.json'), 'utf8'));
  const code = typeof config === 'object' && config !== null && 'goatcounter' in config ? config.goatcounter : '';

  return typeof code === 'string' ? code : '';
};

// The output folder is emptied first, so it must be a folder of its own: never the root, site/ or a parent of them.
// It must lie strictly inside <root>/dist/, the system's temporary folder or the runner's ($RUNNER_TEMP).
const isInside = (folder: string, base: string): boolean => {
  const relative = path.relative(base, folder);

  return relative !== '' && !relative.startsWith('..') && !path.isAbsolute(relative);
};

const isSafeOut = (root: string, out: string): boolean => {
  const site = path.join(root, 'site');
  const bases = [path.join(root, 'dist'), tmpdir(), process.env.RUNNER_TEMP ?? '']
    .filter((base) => base !== '')
    .map((base) => path.resolve(base));
  const holdsRoot = out === root || isInside(root, out);
  const inSite = out === site || isInside(out, site);

  return bases.some((base) => isInside(out, base)) && !holdsRoot && !inSite;
};

export const build = (options: BuildOptions): readonly string[] => {
  const { out, root } = options;
  if (!isSafeOut(root, path.resolve(out))) {
    return [`the output folder ${out} is not a folder of its own inside dist/ or a temporary folder`];
  }

  const parsedRelease = parseRelease(options.release);
  const counter = parseCounterCode(readCounterCode(root, options.counterOverride));
  const catalogueFolder = path.join(root, 'docs', 'categories');
  const catalogueReadme = readFileSync(path.join(catalogueFolder, 'README.md'), 'utf8');
  const { levels, problems: levelProblems } = parseLevels(catalogueReadme);
  const { categories, problems: categoryProblems } = readCategories(
    parseCatalogue(catalogueReadme),
    readCategoryFiles(catalogueFolder),
  );

  const problems = [
    ...levelProblems.map((problem) => `${problem.where}: ${problem.message}`),
    ...categoryProblems.map((problem) => `${problem.where}: ${problem.message}`),
    ...('problem' in parsedRelease ? [`the latest release: ${parsedRelease.problem.message}`] : []),
    ...('problem' in counter ? [`site/config.json: ${counter.problem.message}`] : []),
  ];
  if (problems.length > 0 || 'problem' in parsedRelease || 'problem' in counter) {
    return problems;
  }

  const { release } = parsedRelease;
  const index = renderIndex(readFileSync(path.join(root, 'site', 'index.html'), 'utf8'), {
    categories,
    counterCode: counter.code,
    levels,
    version: release.version,
  });
  if ('problem' in index) {
    return [index.problem.message];
  }

  rmSync(out, { force: true, recursive: true });
  mkdirSync(path.join(out, 'download'), { recursive: true });
  writeFileSync(path.join(out, 'index.html'), index.page);
  writeFileSync(path.join(out, 'download', 'index.html'), renderDownload(release, counter.code));
  for (const [published, source] of COPIED) {
    copyFileSync(path.join(root, source), path.join(out, published));
  }

  return [];
};

const fetchLatestRelease = async (): Promise<unknown> => {
  const token = process.env.GITHUB_TOKEN ?? '';
  const response = await fetch(LATEST_RELEASE, {
    headers: {
      Accept: 'application/vnd.github+json',
      ...(token === '' ? {} : { Authorization: `Bearer ${token}` }),
      'User-Agent': 'fresher-site-build',
      'X-GitHub-Api-Version': '2022-11-28',
    },
    signal: AbortSignal.timeout(30_000),
  });
  if (!response.ok) {
    const remaining = response.headers.get('x-ratelimit-remaining') ?? 'unknown';
    throw new Error(
      `GitHub answered ${String(response.status)} for the latest release (API requests left: ${remaining})`,
    );
  }

  return response.json();
};

const optionAfter = (args: readonly string[], name: string): string | undefined => {
  const index = args.indexOf(name);

  return index === -1 ? undefined : args[index + 1];
};

if (import.meta.main) {
  const args = process.argv.slice(2);
  const root = path.resolve(import.meta.dirname, '..');

  if (args[0] !== 'build') {
    process.stderr.write(`${SCRIPT}: usage: node scripts/site.ts build [--release <file>] [--out <folder>]\n`);
    process.exitCode = 2;
  } else {
    const releaseFile = optionAfter(args, '--release');
    const out = path.resolve(optionAfter(args, '--out') ?? path.join(root, 'dist', 'site'));
    const release: unknown =
      releaseFile === undefined ? await fetchLatestRelease() : JSON.parse(readFileSync(releaseFile, 'utf8'));
    const problems = build({ counterOverride: process.env.GOATCOUNTER_CODE, out, release, root });

    for (const problem of problems) {
      process.stderr.write(`${SCRIPT}: ${problem}\n`);
    }
    if (problems.length === 0) {
      process.stdout.write(`${SCRIPT}: wrote ${out}\n`);
    }
    process.exitCode = problems.length > 0 ? 1 : 0;
  }
}
