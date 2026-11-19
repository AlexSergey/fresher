import { existsSync, lstatSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';

import { UsageError } from './sandbox-dev.ts';

// `npm run dev-runs:check -- <record.json>` (plan 026/03/01): reads a development run record and checks every finding
// and every cleanup item that has a path against the disk, without changing it. The report goes beside the record, never
// over an older one. The scan's time, never the wall clock, is "now", so the answer is the same on the same disk.

export type Check =
  | 'changed-after-scan'
  | 'downloads'
  | 'gone'
  | 'kind'
  | 'link'
  | 'outside'
  | 'protected'
  | 'size'
  | 'stale-projects'
  | 'temp';

export type Completeness = 'complete' | 'partial' | 'unknown';

export type Difference = {
  readonly category: string;
  readonly check: Check;
  readonly expected: string;
  readonly found: string;
  readonly module: string;
  readonly path: string;
};

export type Disk = {
  readonly list: (folder: string) => readonly string[];
  readonly lstat: (path: string) => Stat | undefined;
};

export type Item = {
  readonly allocated: number;
  readonly category: string;
  readonly completeness: Completeness;
  readonly module: string;
  readonly outcome: null | string;
  readonly path: string;
  // The item's scan's `finished`, seconds since the epoch.
  readonly scanAt: number;
  readonly source: 'clear' | 'finding';
};

export type RunRecord = {
  readonly build: string;
  readonly file: string;
  readonly home: string;
  readonly items: readonly Item[];
  readonly mode: string;
  readonly roots: readonly string[];
  readonly settings: Readonly<Record<string, unknown>>;
  // Cleanup items with a path whose scan never finished: not checked, but counted in the report.
  readonly unchecked: number;
};

export type Stat = {
  readonly blocks: number;
  readonly ctimeMs: number;
  readonly dev: number;
  readonly ino: number;
  readonly kind: 'file' | 'folder' | 'link' | 'other';
  readonly mtimeMs: number;
};

const USAGE = 'usage: npm run dev-runs:check -- <record.json>';

const DAY = 86_400;
const MIB = 1_048_576;
const SIZE_TOLERANCE = 0.05;
const STALE_PROJECTS_DAYS = 365;
const TEMP_DAYS = 7;
const CACHE_DAYS = 7;

type Json = Readonly<Record<string, unknown>>;

const isObject = (value: unknown): value is Json =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const objectOf = (value: unknown): Json => (isObject(value) ? value : {});

const arrayOf = (value: unknown): readonly unknown[] => (Array.isArray(value) ? value : []);

const textOf = (value: unknown, fallback = ''): string => (typeof value === 'string' ? value : fallback);

const numberOf = (value: unknown): number | undefined =>
  typeof value === 'number' && Number.isFinite(value) ? value : undefined;

const completenessOf = (sizes: Json): Completeness =>
  sizes.completeness === 'complete' || sizes.completeness === 'partial' ? sizes.completeness : 'unknown';

const seconds = (value: unknown): number | undefined => {
  if (typeof value !== 'string') {
    return undefined;
  }
  const ms = Date.parse(value);

  return Number.isNaN(ms) ? undefined : ms / 1000;
};

type Scans = {
  readonly finishedOf: ReadonlyMap<number, number>;
  readonly items: readonly Item[];
  readonly last: number | undefined;
};

const findingItems = (scan: Json, scanAt: number): readonly Item[] => {
  const removed = new Set(arrayOf(scan.removed));
  const items: Item[] = [];
  for (const moduleValue of Object.values(objectOf(scan.modules))) {
    for (const findingValue of arrayOf(objectOf(moduleValue).findings)) {
      const finding = objectOf(findingValue);
      const subject = objectOf(finding.subject);
      if (subject.kind !== 'path' || typeof subject.value !== 'string' || removed.has(finding.id)) {
        continue;
      }
      const sizes = objectOf(finding.sizes);
      items.push({
        allocated: numberOf(sizes.allocated) ?? 0,
        category: textOf(finding.category),
        completeness: completenessOf(sizes),
        module: textOf(finding.module),
        outcome: null,
        path: subject.value,
        scanAt,
        source: 'finding',
      });
    }
  }

  return items;
};

const scanItems = (scans: readonly unknown[]): Scans => {
  const items: Item[] = [];
  const finishedOf = new Map<number, number>();
  let last: number | undefined;
  for (const scanValue of scans) {
    const scan = objectOf(scanValue);
    const finished = seconds(scan.finished);
    if (finished === undefined) {
      continue;
    }
    last = finished;
    const id = numberOf(scan.id);
    if (id !== undefined) {
      finishedOf.set(id, finished);
    }
    items.push(...findingItems(scan, finished));
  }

  return { finishedOf, items, last };
};

type Clears = { readonly items: readonly Item[]; readonly unchecked: number };

const clearItems = (clears: readonly unknown[], scans: Scans): Clears => {
  const items: Item[] = [];
  let unchecked = 0;
  for (const clearValue of clears) {
    const clear = objectOf(clearValue);
    const scanId = numberOf(clear.scan);
    const scanAt = scanId === undefined ? scans.last : scans.finishedOf.get(scanId);
    for (const group of Object.values(objectOf(clear.groups))) {
      for (const itemValue of arrayOf(group)) {
        const item = objectOf(itemValue);
        if (typeof item.path !== 'string') {
          continue;
        }
        if (scanAt === undefined) {
          unchecked += 1;
          continue;
        }
        const sizes = objectOf(item.sizes);
        items.push({
          allocated: numberOf(sizes.allocated) ?? 0,
          category: textOf(item.category),
          completeness: completenessOf(sizes),
          module: textOf(item.module),
          outcome: textOf(item.outcome, 'unknown'),
          path: item.path,
          scanAt,
          source: 'clear',
        });
      }
    }
  }

  return { items, unchecked };
};

export const parseRecord = (file: string, text: string): RunRecord => {
  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new UsageError(`${file} is not JSON\n${USAGE}`);
  }
  if (!isObject(json) || json.format !== 1) {
    throw new UsageError(`${file} is not a run record of format 1\n${USAGE}`);
  }
  if (!isObject(json.session) || !Array.isArray(json.scans) || !Array.isArray(json.clear)) {
    throw new UsageError(`${file} lacks its session, scans or clear\n${USAGE}`);
  }

  const session = json.session;
  const scans = scanItems(json.scans);
  const clears = clearItems(json.clear, scans);

  return {
    build: textOf(objectOf(session.build).name, 'unknown'),
    file,
    home: textOf(session.home),
    items: [...scans.items, ...clears.items],
    mode: textOf(session.mode, 'unknown'),
    roots: arrayOf(session.roots).filter((root): root is string => typeof root === 'string'),
    settings: objectOf(json.settings),
    unchecked: clears.unchecked,
  };
};

export const nodeDisk: Disk = {
  list: (folder) => {
    try {
      return readdirSync(folder);
    } catch {
      return [];
    }
  },
  lstat: (path) => {
    const stat = lstatSync(path, { bigint: false, throwIfNoEntry: false });
    if (stat === undefined) {
      return;
    }
    const kind = stat.isSymbolicLink() ? 'link' : stat.isDirectory() ? 'folder' : stat.isFile() ? 'file' : 'other';

    return { blocks: stat.blocks, ctimeMs: stat.ctimeMs, dev: stat.dev, ino: stat.ino, kind, mtimeMs: stat.mtimeMs };
  },
};

const ARTIFACTS = new Set(['.build', '.dart_tool', '.gradle', '.venv', 'build', 'node_modules', 'target', 'venv']);
const FORBIDDEN_COMPONENTS = new Set(['.git', 'dist', 'src']);
const STANDARD_FOLDERS = ['Desktop', 'Documents', 'Downloads', 'Movies', 'Music', 'Pictures', 'Public', 'Library'];
const TEMP_PLACES = ['/private/var/folders', '/private/tmp'];
const PROTECTED_PLACES = ['/System', '/usr', '/bin', '/sbin', '/private/var/db'];
const TEMP_FOLDER = /^\/private\/var\/folders\/[^/]+\/[^/]+\/T$/;

const within = (path: string, root: string): boolean => path.startsWith(`${root}/`);

const expandRoot = (root: string, home: string): string =>
  root === '~' ? home : root.startsWith('~/') ? join(home, root.slice(2)) : root;

const isProtected = (path: string, home: string): boolean => {
  if (within(path, '/usr/local') || path === '/usr/local') {
    return false;
  }
  if (PROTECTED_PLACES.some((place) => path === place || within(path, place))) {
    return true;
  }

  return path === home || STANDARD_FOLDERS.some((name) => path === join(home, name));
};

const sizeOf = (disk: Disk, path: string, seen: Set<string>): number => {
  const stat = disk.lstat(path);
  if (stat === undefined) {
    return 0;
  }
  const key = `${stat.dev}:${stat.ino}`;
  let total = 0;
  if (!seen.has(key)) {
    seen.add(key);
    total += stat.blocks * 512;
  }
  if (stat.kind === 'folder') {
    for (const name of disk.list(path)) {
      total += sizeOf(disk, join(path, name), seen);
    }
  }

  return total;
};

// A disk time in whole seconds, as the scan's `finished` is: a change within the scan's last second is not "after" it.
const wholeSeconds = (ms: number): number => Math.floor(ms / 1000);

// The newest time (seconds) of an entry and of everything below it, links not followed: `mtime` only for the folders
// of a project or a cache, `mtime` and `ctime` of every entry for a temporary item.
const newest = (disk: Disk, path: string, temporary: boolean): number => {
  const stat = disk.lstat(path);
  if (stat === undefined) {
    return Number.NEGATIVE_INFINITY;
  }
  let latest = Number.NEGATIVE_INFINITY;
  if (temporary || stat.kind === 'folder') {
    latest = wholeSeconds(temporary ? Math.max(stat.mtimeMs, stat.ctimeMs) : stat.mtimeMs);
  }
  if (stat.kind === 'folder') {
    for (const name of disk.list(path)) {
      latest = Math.max(latest, newest(disk, join(path, name), temporary));
    }
  }

  return latest;
};

// The project's own activity: its folders other than the artifacts and `.git`, and the branches of `.git`.
const projectActivity = (disk: Disk, project: string): number => {
  let latest = Number.NEGATIVE_INFINITY;
  const visit = (folder: string): void => {
    for (const name of disk.list(folder)) {
      const path = join(folder, name);
      const stat = disk.lstat(path);
      if (stat?.kind !== 'folder' || ARTIFACTS.has(name) || name === '.git') {
        continue;
      }
      latest = Math.max(latest, wholeSeconds(stat.mtimeMs));
      visit(path);
    }
  };
  visit(project);
  latest = Math.max(latest, newest(disk, join(project, '.git', 'refs', 'heads'), false));

  return latest;
};

const isCache = (path: string): boolean => {
  const parts = path.split('/');

  return (
    (basename(path) === 'incremental' && parts.slice(0, -1).includes('target')) ||
    path.endsWith('/target/llvm-cov-target')
  );
};

const daysSetting = (settings: Json, key: string, fallback: number): number => numberOf(settings[key]) ?? fallback;

const STALE_PROJECTS_EXPECTED = 'a build folder or cache of a project unused for the days, outside src, dist and .git';

// Why a path is not a stale project's artifact by its place and name alone.
const artifactFault = (path: string, home: string): string | undefined => {
  if (!ARTIFACTS.has(basename(path)) && !isCache(path)) {
    return `the name ${basename(path)} is not an artifact or a cache`;
  }
  const below = within(path, home) ? path.slice(home.length + 1) : path;
  const parts = below.split('/').filter((part) => part !== '');
  const forbidden = parts.find((part) => FORBIDDEN_COMPONENTS.has(part));
  if (forbidden !== undefined) {
    return `a ${forbidden} folder in the path`;
  }
  if (within(path, join(home, 'Library'))) {
    return 'under the home Library';
  }
  if (parts.some((part) => part.endsWith('.app'))) {
    return 'inside an app';
  }
  if (within(path, home) && parts[0]?.startsWith('.') === true) {
    return 'under a hidden folder of the home';
  }

  return undefined;
};

type Context = {
  readonly disk: Disk;
  readonly home: string;
  readonly roots: readonly string[];
  readonly settings: Json;
};

type Report = (check: Check, expected: string, found: string) => void;

const changedAfter = (report: Report, scanAt: number, latest: number): void => {
  report('changed-after-scan', `nothing changed after the scan (${scanAt})`, `changed at ${latest}`);
};

const checkTemp = (item: Item, context: Context, report: Report): void => {
  const parent = dirname(item.path);
  if (parent !== '/private/tmp' && !TEMP_FOLDER.test(parent)) {
    report('temp', 'directly inside /private/tmp or a user temporary folder', `inside ${parent}`);

    return;
  }
  const days = daysSetting(context.settings, 'temp.older-than-days', TEMP_DAYS);
  const latest = newest(context.disk, item.path, true);
  if (latest > item.scanAt) {
    changedAfter(report, item.scanAt, latest);
  } else if (latest > item.scanAt - days * DAY) {
    report('temp', `nothing changed within ${days} days of the scan`, `changed at ${latest}`);
  }
};

const checkStaleProject = (item: Item, context: Context, report: Report): void => {
  const fault = artifactFault(item.path, context.home);
  if (fault !== undefined) {
    report('stale-projects', STALE_PROJECTS_EXPECTED, fault);

    return;
  }
  const cache = isCache(item.path);
  const days = cache
    ? CACHE_DAYS
    : daysSetting(context.settings, 'stale-projects.unused-for-days', STALE_PROJECTS_DAYS);
  const latest = cache ? newest(context.disk, item.path, false) : projectActivity(context.disk, dirname(item.path));
  if (latest > item.scanAt) {
    changedAfter(report, item.scanAt, latest);
  } else if (latest > item.scanAt - days * DAY) {
    report('stale-projects', `no activity within ${days} days of the scan`, `activity at ${latest}`);
  }
};

// Whether an item that ended as `outcome` should still be on the disk.
const stillExpected = (outcome: null | string): boolean =>
  outcome === null ||
  outcome === 'failed' ||
  outcome === 'not-reached' ||
  outcome === 'skipped' ||
  outcome.startsWith('would-');

const checkPlace = (item: Item, context: Context, report: Report): void => {
  const inRoot = context.roots.some((root) => within(item.path, root));
  if (!inRoot && !TEMP_PLACES.some((place) => within(item.path, place))) {
    report('outside', 'strictly inside a scan root or a temporary place', 'outside every root');
  }
  if (isProtected(item.path, context.home)) {
    report('protected', 'not a protected place, not the home or a standard folder of it', 'a protected place');
  }
};

const checkModule = (item: Item, context: Context, report: Report): void => {
  if (item.module === 'downloads') {
    if (dirname(item.path) !== join(context.home, 'Downloads')) {
      report('downloads', 'directly inside the Downloads folder', `inside ${dirname(item.path)}`);
    }
  } else if (item.module === 'temp') {
    checkTemp(item, context, report);
  } else if (item.module === 'stale-projects') {
    checkStaleProject(item, context, report);
  }
};

const checkItem = (item: Item, context: Context, report: Report): void => {
  const stat = context.disk.lstat(item.path);
  if (item.outcome === 'moved' || item.outcome === 'deleted') {
    if (stat !== undefined) {
      report('gone', `gone, the item ${item.outcome}`, 'still in place');
    }

    return;
  }
  if (stat === undefined) {
    if (stillExpected(item.outcome)) {
      report('gone', 'in place', 'missing');
    }

    return;
  }

  if (stat.kind === 'link') {
    report('link', 'not a link', 'a link');
  } else if (stat.kind === 'other') {
    report('kind', 'a file or a folder', 'neither (a socket, a pipe or a device)');
  }
  const readable = stat.kind === 'file' || stat.kind === 'folder';
  if (readable && item.completeness === 'complete') {
    const found = sizeOf(context.disk, item.path, new Set());
    if (Math.abs(found - item.allocated) > Math.max(item.allocated * SIZE_TOLERANCE, MIB)) {
      report('size', `${item.allocated} bytes within 5 % or 1 MiB`, `${found} bytes`);
    }
  }
  checkPlace(item, context, report);
  if (readable) {
    checkModule(item, context, report);
  }
};

export const checkRecord = (record: RunRecord, disk: Disk): readonly Difference[] => {
  const differences: Difference[] = [];
  const context: Context = {
    disk,
    home: record.home,
    roots: record.roots.map((root) => expandRoot(root, record.home)),
    settings: record.settings,
  };
  for (const item of record.items) {
    checkItem(item, context, (check, expected, found) => {
      differences.push({ category: item.category, check, expected, found, module: item.module, path: item.path });
    });
  }

  return differences;
};

const cell = (text: string): string => text.replaceAll('|', '\\|').replaceAll('\n', ' ');

export const renderReport = (record: RunRecord, differences: readonly Difference[]): string => {
  const findings = record.items.filter((item) => item.source === 'finding').length;
  const cleared = record.items.length - findings;
  const lines = [
    `# Check of ${record.file}`,
    '',
    `Build: ${record.build}`,
    `Mode: ${record.mode}`,
    '',
    `- Findings checked: ${findings}`,
    `- Cleanup items checked: ${cleared}`,
    `- Cleanup items not checked (their scan never finished): ${record.unchecked}`,
    `- Differences: ${differences.length}`,
    '',
  ];
  if (differences.length === 0) {
    lines.push('No differences.');
  } else {
    lines.push('| Path | Module | Category | Check | Expected | Found |', '| --- | --- | --- | --- | --- | --- |');
    for (const d of differences) {
      lines.push(`| ${[d.path, d.module, d.category, d.check, d.expected, d.found].map(cell).join(' | ')} |`);
    }
  }

  return `${lines.join('\n')}\n`;
};

export const reportPath = (record: string, exists: (path: string) => boolean): string => {
  if (!exists(`${record}.check.md`)) {
    return `${record}.check.md`;
  }
  for (let n = 2; ; n += 1) {
    const candidate = `${record}.check-${n}.md`;
    if (!exists(candidate)) {
      return candidate;
    }
  }
};

if (import.meta.main) {
  try {
    const args = process.argv.slice(2);
    const [file, ...extra] = args;
    if (file === undefined || extra.length > 0) {
      throw new UsageError(`expected exactly one record\n${USAGE}`);
    }
    const record = parseRecord(file, readFileSync(file, 'utf8'));
    const differences = checkRecord(record, nodeDisk);
    const path = reportPath(file, existsSync);
    writeFileSync(path, renderReport(record, differences), { flag: 'wx' });
    process.stdout.write(`${path}\n`);
    process.exitCode = differences.length === 0 ? 0 : 1;
  } catch (error) {
    if (!(error instanceof UsageError)) {
      throw error;
    }
    process.stderr.write(`dev-runs-check: ${error.message}\n`);
    process.exitCode = 2;
  }
}
