import assert from 'node:assert/strict';
import { linkSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, utimesSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, describe, it } from 'node:test';

import {
  type Check,
  checkRecord,
  type Difference,
  type Disk,
  nodeDisk,
  parseRecord,
  renderReport,
  reportPath,
  type RunRecord,
  type Stat,
} from './dev-runs-check.ts';
import { UsageError } from './sandbox-dev.ts';

const SCAN = Date.parse('2026-11-20T10:00:00Z') / 1000;
const SCAN_ISO = '2026-11-20T10:00:00Z';
const DAY = 86_400;
const OLD = SCAN - 400 * DAY;
const MIB = 1_048_576;

const made: string[] = [];
const temp = (): string => {
  const folder = mkdtempSync(join(tmpdir(), 'dev-runs-check-'));
  made.push(folder);

  return folder;
};
after(() => {
  for (const folder of made) {
    rmSync(folder, { force: true, recursive: true });
  }
});

const make = (path: string, content = ''): string => {
  mkdirSync(join(path, '..'), { recursive: true });
  writeFileSync(path, content);

  return path;
};

// Sets the time of a folder or file and of everything below it (links are not followed).
const age = (path: string, seconds: number): void => {
  const entries = nodeDisk.list(path);
  for (const name of entries) {
    age(join(path, name), seconds);
  }
  if (nodeDisk.lstat(path)?.kind !== 'link') {
    utimesSync(path, seconds, seconds);
  }
};

type Entry = {
  readonly allocated?: number;
  readonly category?: string;
  readonly completeness?: string;
  readonly kind?: string;
  readonly module?: string;
  readonly outcome?: string;
  readonly path: string;
};

type Shape = {
  readonly clear?: readonly Entry[];
  readonly findings?: readonly Entry[];
  readonly home: string;
  readonly removed?: readonly number[];
  readonly roots?: readonly string[];
  readonly settings?: Readonly<Record<string, unknown>>;
};

const text = (shape: Shape): string =>
  JSON.stringify({
    clear: [
      {
        groups: {
          g: (shape.clear ?? []).map((entry) => ({
            category: entry.category ?? 'c',
            module: entry.module ?? 'stale-projects',
            outcome: entry.outcome ?? 'would-move',
            path: entry.path,
            sizes: { allocated: entry.allocated ?? 0, completeness: entry.completeness ?? 'unknown' },
          })),
        },
        id: 1,
        scan: 1,
      },
    ],
    format: 1,
    scans: [
      {
        finished: SCAN_ISO,
        id: 1,
        modules: {
          m: {
            findings: (shape.findings ?? []).map((entry, index) => ({
              category: entry.category ?? 'c',
              id: index + 1,
              module: entry.module ?? 'stale-projects',
              sizes: { allocated: entry.allocated ?? 0, completeness: entry.completeness ?? 'unknown' },
              subject: { kind: 'path', value: entry.path },
            })),
          },
        },
        removed: shape.removed ?? [],
      },
    ],
    session: { build: { name: '0.1.0+abc1234' }, home: shape.home, mode: 'mac', roots: shape.roots ?? ['~'] },
    settings: shape.settings ?? {},
  });

const record = (shape: Shape): RunRecord => parseRecord('run.json', text(shape));

const checks = (differences: readonly Difference[]): readonly Check[] => differences.map((d) => d.check);

const stat = (kind: Stat['kind'], ino: number, mtime = OLD): Stat => ({
  blocks: 0,
  ctimeMs: mtime * 1000,
  dev: 1,
  ino,
  kind,
  mtimeMs: mtime * 1000,
});

// A disk in memory: paths with their stats; a folder lists the entries whose parent it is.
const fakeDisk = (entries: Readonly<Record<string, Stat>>): Disk => ({
  list: (folder) =>
    Object.keys(entries)
      .filter((path) => path.startsWith(`${folder}/`) && !path.slice(folder.length + 1).includes('/'))
      .map((path) => path.slice(folder.length + 1)),
  lstat: (path) => entries[path],
});

// A project `<home>/proj` with an old `src/` folder and a `node_modules` below it, all older than the days.
const project = (home: string): string => {
  make(join(home, 'proj', 'src', 'main.ts'));
  make(join(home, 'proj', 'node_modules', 'pkg', 'index.js'));
  mkdirSync(join(home, 'proj', '.git', 'refs', 'heads'), { recursive: true });
  age(join(home, 'proj'), OLD);

  return join(home, 'proj', 'node_modules');
};

describe('dev-runs-check', () => {
  describe('negative cases', () => {
    it('a_record_of_another_format_is_refused', () => {
      const other = JSON.stringify({ clear: [], format: 2, scans: [], session: {} });

      assert.throws(() => parseRecord('run.json', other), UsageError);
      assert.throws(() => parseRecord('run.json', JSON.stringify({ clear: [], format: 1, scans: [] })), UsageError);
    });

    it('a_file_that_is_not_json_is_refused', () => {
      assert.throws(() => parseRecord('run.json', 'not json'), UsageError);
    });

    it('a_missing_finding_is_a_difference', () => {
      const home = temp();

      const differences = checkRecord(record({ findings: [{ path: join(home, 'nothing') }], home }), nodeDisk);

      assert.deepEqual(checks(differences), ['gone']);
    });

    it('a_removed_finding_is_not_checked_as_a_finding', () => {
      const home = temp();
      const shape: Shape = {
        clear: [{ outcome: 'moved', path: join(home, 'moved') }],
        findings: [{ path: join(home, 'moved') }],
        home,
        removed: [1],
      };

      assert.deepEqual(checkRecord(record(shape), nodeDisk), []);
      assert.deepEqual(checks(checkRecord(record({ ...shape, removed: [] }), nodeDisk)), ['gone']);
    });

    it('a_link_in_place_of_a_folder_is_a_difference', () => {
      const home = temp();
      mkdirSync(join(home, 'real'));
      symlinkSync(join(home, 'real'), join(home, 'node_modules'));

      const differences = checkRecord(record({ findings: [{ path: join(home, 'node_modules') }], home }), nodeDisk);

      assert.ok(checks(differences).includes('link'));
    });

    it('a_socket_or_pipe_is_a_difference', () => {
      const home = '/home/u';
      const disk = fakeDisk({ [`${home}/proj/node_modules`]: stat('other', 2) });

      const differences = checkRecord(record({ findings: [{ path: `${home}/proj/node_modules` }], home }), disk);

      assert.ok(checks(differences).includes('kind'));
    });

    it('a_size_beyond_the_tolerance_is_a_difference', () => {
      const home = temp();
      const folder = join(home, 'cache');
      make(join(folder, 'big'), 'x'.repeat(4 * MIB));
      const found = ((): number => {
        const file = nodeDisk.lstat(join(folder, 'big'));
        const own = nodeDisk.lstat(folder);

        return ((file?.blocks ?? 0) + (own?.blocks ?? 0)) * 512;
      })();
      const entry = (allocated: number): Shape => ({
        findings: [{ allocated, completeness: 'complete', module: 'other', path: folder }],
        home,
      });

      assert.deepEqual(checks(checkRecord(record(entry(found * 3)), nodeDisk)), ['size']);
      assert.deepEqual(checks(checkRecord(record(entry(found + MIB / 2)), nodeDisk)), []);
    });

    it('a_partial_size_is_not_compared', () => {
      const home = temp();
      make(join(home, 'cache', 'big'), 'x'.repeat(4 * MIB));

      const shape: Shape = {
        findings: [{ allocated: 1, completeness: 'partial', module: 'other', path: join(home, 'cache') }],
        home,
      };

      assert.deepEqual(checkRecord(record(shape), nodeDisk), []);
    });

    it('a_path_outside_the_roots_is_a_difference', () => {
      const home = temp();
      const lookAlike = `${home}2`;
      mkdirSync(lookAlike);
      const elsewhere = temp();
      mkdirSync(join(elsewhere, 'x'));

      const shape: Shape = {
        findings: [
          { module: 'other', path: lookAlike },
          { module: 'other', path: join(elsewhere, 'x') },
        ],
        home,
        roots: [home],
      };

      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['outside', 'outside']);
    });

    it('the_home_or_its_standard_folder_itself_is_a_difference', () => {
      const home = temp();
      mkdirSync(join(home, 'Documents'));
      const disk = fakeDisk({ '/System/Library/x': stat('folder', 9) });

      const shape: Shape = {
        findings: [
          { module: 'other', path: home },
          { module: 'other', path: join(home, 'Documents') },
        ],
        home,
      };
      const system: Shape = { findings: [{ module: 'other', path: '/System/Library/x' }], home, roots: ['/System'] };

      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['outside', 'protected', 'protected']);
      assert.deepEqual(checks(checkRecord(record(system), disk)), ['protected']);
    });

    it('a_stale_project_with_a_recent_folder_or_branch_is_a_difference', () => {
      const home = temp();
      const artifact = project(home);
      const recent = SCAN - 10 * DAY;
      const shape: Shape = { findings: [{ path: artifact }], home };

      utimesSync(join(home, 'proj', 'src'), recent, recent);
      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['stale-projects']);

      utimesSync(join(home, 'proj', 'src'), OLD, OLD);
      assert.deepEqual(checkRecord(record(shape), nodeDisk), []);
      utimesSync(join(home, 'proj', '.git', 'refs', 'heads'), recent, recent);
      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['stale-projects']);
    });

    it('an_artifact_named_src_or_under_library_or_an_app_is_a_difference', () => {
      const home = temp();
      const paths = [
        join(home, 'proj', 'src'),
        join(home, 'Library', 'Caches', 'node_modules'),
        join(home, 'Tool.app', 'Contents', 'node_modules'),
        join(home, 'proj', 'dist', 'node_modules'),
        join(home, '.hidden', 'node_modules'),
      ];
      for (const path of paths) {
        mkdirSync(path, { recursive: true });
        age(path, OLD);
      }

      const differences = checkRecord(record({ findings: paths.map((path) => ({ path })), home }), nodeDisk);

      assert.deepEqual(checks(differences), [
        'stale-projects',
        'stale-projects',
        'stale-projects',
        'stale-projects',
        'stale-projects',
      ]);
    });

    it('a_rust_cache_built_within_seven_days_is_a_difference', () => {
      const home = temp();
      const cache = join(home, 'proj', 'target', 'debug', 'incremental');
      make(join(cache, 'session', 'data'));
      age(cache, OLD);
      const shape: Shape = { findings: [{ path: cache }], home };

      assert.deepEqual(checkRecord(record(shape), nodeDisk), []);
      utimesSync(join(cache, 'session'), SCAN - 2 * DAY, SCAN - 2 * DAY);
      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['stale-projects']);
    });

    it('a_download_below_the_first_level_is_a_difference', () => {
      const home = temp();
      make(join(home, 'Downloads', 'sub', 'file.dmg'));
      make(join(home, 'Downloads', 'top.dmg'));

      const shape: Shape = {
        findings: [
          { module: 'downloads', path: join(home, 'Downloads', 'sub', 'file.dmg') },
          { module: 'downloads', path: join(home, 'Downloads', 'top.dmg') },
        ],
        home,
      };

      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['downloads']);
    });

    it('a_temp_item_changed_within_the_days_is_a_difference', () => {
      const item = '/private/tmp/build-1';
      const disk = fakeDisk({
        '/private/tmp/build-1': stat('folder', 1),
        '/private/tmp/build-1/a': stat('file', 2, SCAN - 3 * DAY),
      });

      const differences = checkRecord(record({ findings: [{ module: 'temp', path: item }], home: '/home/u' }), disk);

      assert.deepEqual(checks(differences), ['temp']);
    });

    it('a_moved_item_still_in_place_is_a_difference', () => {
      const home = temp();
      mkdirSync(join(home, 'still'));

      const differences = checkRecord(
        record({ clear: [{ module: 'other', outcome: 'moved', path: join(home, 'still') }], home }),
        nodeDisk,
      );

      assert.deepEqual(checks(differences), ['gone']);
    });

    it('an_existing_report_is_never_overwritten', () => {
      const only =
        (taken: readonly string[]) =>
        (path: string): boolean =>
          taken.includes(path);

      assert.equal(reportPath('r.json', only([])), 'r.json.check.md');
      assert.equal(reportPath('r.json', only(['r.json.check.md'])), 'r.json.check-2.md');
      assert.equal(reportPath('r.json', only(['r.json.check.md', 'r.json.check-2.md'])), 'r.json.check-3.md');
    });

    it('a_cleanup_item_of_an_unfinished_scan_is_counted_not_dropped_silently', () => {
      const body = JSON.parse(
        text({ clear: [{ path: '/home/u/a' }, { path: '/home/u/b' }], home: '/home/u' }),
      ) as Record<string, unknown>;
      body.scans = [{ id: 1, modules: {} }];

      const parsed = parseRecord('run.json', JSON.stringify(body));

      assert.deepEqual(parsed.items, []);
      assert.equal(parsed.unchecked, 2);
      assert.match(renderReport(parsed, []), /Cleanup items not checked \(their scan never finished\): 2/);
    });
  });

  describe('positive cases', () => {
    it('a_record_that_matches_the_disk_has_no_difference', () => {
      const home = temp();
      const artifact = project(home);
      make(join(home, 'Downloads', 'old.dmg'));
      age(join(home, 'Downloads'), OLD);
      const disk = fakeDisk({
        '/private/tmp/old': stat('folder', 5),
        '/private/tmp/old/x': stat('file', 6),
      });

      const stale = checkRecord(
        record({
          clear: [{ outcome: 'would-move', path: artifact }],
          findings: [{ path: artifact }, { module: 'downloads', path: join(home, 'Downloads', 'old.dmg') }],
          home,
        }),
        nodeDisk,
      );
      const temps = checkRecord(
        record({ findings: [{ module: 'temp', path: '/private/tmp/old' }], home: '/home/u' }),
        disk,
      );

      assert.deepEqual(stale, []);
      assert.deepEqual(temps, []);
    });

    it('hard_links_are_counted_once', () => {
      const home = temp();
      const folder = join(home, 'cache');
      make(join(folder, 'a'), 'x'.repeat(4 * MIB));
      linkSync(join(folder, 'a'), join(folder, 'b'));
      const once = ((nodeDisk.lstat(join(folder, 'a'))?.blocks ?? 0) + (nodeDisk.lstat(folder)?.blocks ?? 0)) * 512;

      const differences = checkRecord(
        record({ findings: [{ allocated: once, completeness: 'complete', module: 'other', path: folder }], home }),
        nodeDisk,
      );

      assert.deepEqual(differences, []);
    });

    it('a_change_after_the_scan_is_reported_as_such', () => {
      const home = temp();
      const artifact = project(home);
      utimesSync(join(home, 'proj', 'src'), SCAN + 3600, SCAN + 3600);

      const differences = checkRecord(record({ findings: [{ path: artifact }], home }), nodeDisk);

      assert.deepEqual(checks(differences), ['changed-after-scan']);
    });

    it('a_change_within_the_scans_last_second_is_not_after_it', () => {
      const home = temp();
      const artifact = project(home);
      utimesSync(join(home, 'proj', 'src'), SCAN + 0.6, SCAN + 0.6);

      const differences = checkRecord(record({ findings: [{ path: artifact }], home }), nodeDisk);

      assert.deepEqual(checks(differences), ['stale-projects']);
    });

    it('an_item_without_a_path_is_not_checked', () => {
      const body = JSON.parse(text({ home: '/home/u' })) as Record<string, unknown>;
      body.clear = [
        { groups: { g: [{ category: 'c', module: 'docker', outcome: 'tool-done', path: null }] }, id: 1, scan: 1 },
      ];
      body.scans = [
        {
          finished: SCAN_ISO,
          id: 1,
          modules: {
            docker: {
              findings: [{ id: 1, module: 'docker', sizes: {}, subject: { kind: 'external', value: { id: 'x' } } }],
            },
          },
        },
      ];

      const parsed = parseRecord('run.json', JSON.stringify(body));

      assert.deepEqual(parsed.items, []);
    });

    it('the_report_lists_every_difference_in_a_table', () => {
      const home = temp();
      const parsed = record({ findings: [{ category: 'cat', path: join(home, 'a|b') }], home });

      const report = renderReport(parsed, checkRecord(parsed, nodeDisk));

      assert.match(report, /^# Check of run\.json/);
      assert.match(report, /Build: 0\.1\.0\+abc1234/);
      assert.match(report, /Mode: mac/);
      assert.match(report, /Findings checked: 1/);
      assert.match(report, /Differences: 1/);
      assert.match(report, /\| Path \| Module \| Category \| Check \| Expected \| Found \|/);
      assert.match(report, /a\\\|b \| stale-projects \| cat \| gone \|/);
    });

    it('the_report_says_when_there_is_no_difference', () => {
      const parsed = record({ home: temp() });

      assert.match(renderReport(parsed, []), /No differences\./);
    });

    it('the_settings_days_are_used_and_the_defaults_without_them', () => {
      const home = temp();
      const artifact = project(home);
      utimesSync(join(home, 'proj', 'src'), SCAN - 100 * DAY, SCAN - 100 * DAY);
      const shape: Shape = { findings: [{ path: artifact }], home };

      const withDays = (days: unknown): readonly Check[] =>
        checks(checkRecord(record({ ...shape, settings: { 'stale-projects.unused-for-days': days } }), nodeDisk));

      assert.deepEqual(checks(checkRecord(record(shape), nodeDisk)), ['stale-projects']);
      assert.deepEqual(withDays('x'), ['stale-projects']);
      assert.deepEqual(withDays(200), ['stale-projects']);
      assert.deepEqual(withDays(30), []);
    });
  });
});
