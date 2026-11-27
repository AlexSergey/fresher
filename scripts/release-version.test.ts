import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { after, describe, it } from 'node:test';

import { buildSection } from './flags.ts';
import {
  cargoLockPlaces,
  cargoWorkspaceVersion,
  changelogSection,
  checkVersion,
  isVersion,
  jsonVersion,
  npmLockPlaces,
  releaseBody,
  releaseProblems,
  type Run,
  setCargoWorkspaceVersion,
  setJsonVersion,
  setVersion,
  versionProblems,
} from './release-version.ts';

const CARGO_TOML = [
  '[workspace]',
  'members = ["crates/*"]',
  '',
  '[workspace.package]',
  'edition = "2024"',
  'version = "0.1.0"',
  'license = "MIT OR Apache-2.0"',
  '',
  '[workspace.dependencies]',
  'serde = { version = "1.0.0" }',
  '',
].join('\n');

const packageJson = (version: string, extra = ''): string =>
  `{\n  "name": "fresher",\n  "version": "${version}",${extra}\n  "dependencies": {\n    "react": {\n      "version": "19.0.0"\n    }\n  }\n}\n`;

const npmLock = (root: string, ui: string): string =>
  `${JSON.stringify(
    {
      lockfileVersion: 3,
      name: 'fresher',
      packages: {
        '': { name: 'fresher', version: root },
        'app/ui': { name: 'fresher-ui', version: ui },
        'node_modules/react': { version: '19.0.0' },
      },
      version: root,
    },
    undefined,
    2,
  )}\n`;

const cargoLock = (app: string, core: string): string =>
  [
    'version = 4',
    '',
    '[[package]]',
    'name = "fresher-app"',
    `version = "${app}"`,
    '',
    '[[package]]',
    'name = "fresher-core"',
    `version = "${core}"`,
    '',
    '[[package]]',
    'name = "serde"',
    'version = "1.0.0"',
    'source = "registry+https://github.com/rust-lang/crates.io-index"',
    '',
  ].join('\n');

const CHANGELOG = '# Changelog\n\n## [Unreleased]\n\n## [0.1.0] - 2026-11-25\n\n### Added\n\n- Scan.\n';

const folders: string[] = [];

// A repository with every place at `version`; `overrides` replaces a file's text.
const repository = (version: string, overrides: Readonly<Record<string, string>> = {}): string => {
  const root = mkdtempSync(join(tmpdir(), 'release-version-'));
  folders.push(root);
  const files: Record<string, string> = {
    'app/tauri/tauri.conf.json': `{\n  "productName": "Fresher",\n  "version": "${version}",\n  "bundle": {}\n}\n`,
    'app/ui/package.json': packageJson(version),
    'Cargo.lock': cargoLock(version, version),
    'Cargo.toml': setCargoWorkspaceVersion(CARGO_TOML, version),
    'docs/changelog.md': CHANGELOG,
    'package-lock.json': npmLock(version, version),
    'package.json': packageJson(version, '\n  "workspaces": ["app/ui"],'),
    ...overrides,
  };
  for (const [file, text] of Object.entries(files)) {
    mkdirSync(dirname(join(root, file)), { recursive: true });
    writeFileSync(join(root, file), text);
  }

  return root;
};

// Stands in for npm and Cargo: writes the lock files as they would.
const refreshingLocks =
  (root: string, version: string, calls: string[]): Run =>
  (program, args) => {
    calls.push([program, ...args].join(' '));
    if (program === 'npm') {
      writeFileSync(join(root, 'package-lock.json'), npmLock(version, version));
    } else {
      writeFileSync(join(root, 'Cargo.lock'), cargoLock(version, version));
    }
  };

after(() => {
  for (const folder of folders) {
    rmSync(folder, { force: true, recursive: true });
  }
});

describe('release-version', () => {
  describe('negative cases', () => {
    it('a_version_that_is_not_major_minor_patch_is_refused', () => {
      for (const text of ['', 'v0.1.0', '0.1', '0.1.0-rc.1', '01.0.0', '0.1.0 ']) {
        assert.equal(isVersion(text), false, text);
      }
    });

    it('a_place_holding_another_version_is_named', () => {
      const root = repository('0.1.0', { 'Cargo.lock': cargoLock('0.1.0', '0.0.9') });
      assert.deepEqual(checkVersion(root), ['Cargo.lock (fresher-core): 0.0.9, not 0.1.0']);
    });

    it('every_place_is_read', () => {
      const root = repository('0.1.0', {
        'app/tauri/tauri.conf.json': '{\n  "version": "0.0.1"\n}\n',
        'app/ui/package.json': packageJson('0.0.2'),
        'package-lock.json': npmLock('0.0.3', '0.0.4'),
        'package.json': packageJson('0.0.5', '\n  "workspaces": ["app/ui"],'),
      });
      assert.deepEqual(checkVersion(root), [
        'package.json: 0.0.5, not 0.1.0',
        'app/ui/package.json: 0.0.2, not 0.1.0',
        'app/tauri/tauri.conf.json: 0.0.1, not 0.1.0',
        'package-lock.json (root): 0.0.3, not 0.1.0',
        'package-lock.json (packages[""]): 0.0.3, not 0.1.0',
        'package-lock.json (packages["app/ui"]): 0.0.4, not 0.1.0',
      ]);
    });

    it('a_release_version_the_places_do_not_hold_is_refused', () => {
      const root = repository('0.1.0');
      assert.equal(checkVersion(root, '0.2.0').length, 9 + 2);
      assert.deepEqual(checkVersion(root, 'v0.1.0'), ['v0.1.0 is not a version of the form MAJOR.MINOR.PATCH']);
    });

    it('a_release_without_its_changelog_section_or_notes_is_refused', () => {
      assert.deepEqual(releaseProblems('0.2.0', CHANGELOG, false), [
        'docs/changelog.md: no section "## [0.2.0] - YYYY-MM-DD"',
        'docs/releases/0.2.0.md: the release notes are missing',
      ]);
      assert.equal(releaseProblems('0.1.0', '## [0.1.0]\n', true).length, 1);
      assert.equal(releaseProblems('0.1.0', '## [0x1x0] - 2026-11-25\n', true).length, 1);
    });

    it('a_changelog_without_the_versions_section_gives_no_section', () => {
      assert.equal(changelogSection('## [Unreleased]\n\n## [0.1.0] - 2026-11-25\n\n- one\n', '0.2.0'), undefined);
      assert.equal(changelogSection('## [0.1.0]\n\n- one\n', '0.1.0'), undefined);
    });

    it('a_cargo_toml_without_a_workspace_version_has_none', () => {
      assert.equal(cargoWorkspaceVersion('[package]\nversion = "0.1.0"\n'), undefined);
      assert.deepEqual(versionProblems([]), ['Cargo.toml: no version in [workspace.package]']);
    });

    it('a_new_version_that_is_not_major_minor_patch_writes_nothing', () => {
      const root = repository('0.1.0');
      const calls: string[] = [];
      assert.equal(setVersion(root, '0.2', refreshingLocks(root, '0.2', calls)).length, 1);
      assert.deepEqual(calls, []);
      assert.deepEqual(checkVersion(root), []);
    });

    it('lock_files_left_behind_are_named_after_a_new_version', () => {
      const root = repository('0.1.0');
      const problems = setVersion(root, '0.2.0', () => undefined);
      assert.deepEqual(problems, [
        'package-lock.json (root): 0.1.0, not 0.2.0',
        'package-lock.json (packages[""]): 0.1.0, not 0.2.0',
        'package-lock.json (packages["app/ui"]): 0.1.0, not 0.2.0',
        'Cargo.lock (fresher-app): 0.1.0, not 0.2.0',
        'Cargo.lock (fresher-core): 0.1.0, not 0.2.0',
      ]);
    });
  });

  describe('positive cases', () => {
    it('the_versions_changelog_section_ends_at_the_next_version', () => {
      const changelog =
        '# Changelog\n\n## [Unreleased]\n\n## [0.2.0] - 2026-12-01\n\n### Fixed\n\n- two\n\n## [0.1.0] - 2026-11-25\n\n- one\n';
      assert.equal(changelogSection(changelog, '0.2.0'), '### Fixed\n\n- two');
      assert.equal(changelogSection(changelog, '0.1.0'), '- one');
    });

    it('the_release_body_is_the_notes_then_whats_new', () => {
      assert.equal(
        releaseBody('# Fresher 0.2.0\n\nNotes.\n\n', '### Fixed\n\n- two', '0.2.0', '## This build\n\n- x\n'),
        "# Fresher 0.2.0\n\nNotes.\n\n## What's new in 0.2.0\n\n### Fixed\n\n- two\n\n## This build\n\n- x\n",
      );
    });

    it('the_body_ends_with_the_builds_flags', () => {
      const flags = readFileSync(join(import.meta.dirname, '..', 'flags.toml'), 'utf8');
      const { section } = buildSection(flags, undefined);
      assert.ok(section?.startsWith('## This build'));
      assert.ok(releaseBody('Notes.', '- one', '0.2.0', section ?? '').endsWith(section ?? 'x'));
    });

    it('the_same_version_everywhere_passes', () => {
      assert.deepEqual(checkVersion(repository('0.1.0')), []);
    });

    it('a_release_with_its_version_changelog_section_and_notes_passes', () => {
      const root = repository('0.1.0', { 'docs/releases/0.1.0.md': '# Fresher 0.1.0\n' });
      assert.deepEqual(checkVersion(root, '0.1.0'), []);
    });

    it('a_new_version_is_written_everywhere_and_the_locks_are_refreshed_offline', () => {
      const root = repository('0.1.0');
      const calls: string[] = [];
      assert.deepEqual(setVersion(root, '0.2.0', refreshingLocks(root, '0.2.0', calls)), []);
      assert.deepEqual(checkVersion(root), []);
      assert.deepEqual(calls, [
        'npm install --package-lock-only --ignore-scripts --no-audit --no-fund --offline',
        'cargo update --workspace --offline',
      ]);
      const cargoToml = readFileSync(join(root, 'Cargo.toml'), 'utf8');
      assert.equal(cargoToml, CARGO_TOML.replace('version = "0.1.0"', 'version = "0.2.0"'));
      assert.match(readFileSync(join(root, 'package.json'), 'utf8'), /"version": "19\.0\.0"/u);
    });

    it('only_the_top_level_version_of_a_json_file_changes', () => {
      const text = packageJson('0.1.0');
      assert.equal(jsonVersion(text), '0.1.0');
      assert.equal(setJsonVersion(text, '1.2.3'), text.replace('"version": "0.1.0"', '"version": "1.2.3"'));
    });

    it('the_locks_list_the_root_the_workspaces_and_the_workspace_crates', () => {
      assert.deepEqual(npmLockPlaces(npmLock('0.1.0', '0.1.0'), ['app/ui']).length, 3);
      assert.deepEqual(cargoLockPlaces(cargoLock('0.1.0', '0.1.0')), [
        { name: 'Cargo.lock (fresher-app)', version: '0.1.0' },
        { name: 'Cargo.lock (fresher-core)', version: '0.1.0' },
      ]);
    });
  });
});
