import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { cargoLockProblems, npmLockProblems } from './check-locks.ts';

const REACT = {
  integrity: 'sha512-abc',
  resolved: 'https://registry.npmjs.org/react/-/react-19.2.0.tgz',
  version: '19.2.0',
};

const npmLock = (packages: Record<string, unknown>): string =>
  JSON.stringify({ lockfileVersion: 3, packages: { '': { name: 'fresher' }, ...packages } });

const CHECKSUM = 'a'.repeat(64);

const crate = (lines: string): string => `[[package]]\nname = "serde"\nversion = "1.0.0"\n${lines}\n`;

describe('check-locks', () => {
  describe('negative cases', () => {
    it('an_npm_package_resolved_elsewhere_is_refused', () => {
      const lock = npmLock({
        'node_modules/react': { ...REACT, resolved: 'https://evil.example/react-19.2.0.tgz' },
      });
      const problems = npmLockProblems(lock, []);
      assert.equal(problems.length, 1);
      assert.match(problems[0]?.message ?? '', /node_modules\/react: resolved from https:\/\/evil\.example/u);
    });

    it('an_npm_package_without_integrity_is_refused', () => {
      const lock = npmLock({ 'node_modules/react': { ...REACT, integrity: undefined } });
      assert.deepEqual(
        npmLockProblems(lock, []).map((problem) => problem.message),
        ['node_modules/react: no sha512 integrity'],
      );
    });

    it('an_npm_package_from_a_registry_lookalike_is_refused', () => {
      const lock = npmLock({
        'node_modules/react': { ...REACT, resolved: 'https://registry.npmjs.org.evil.example/react.tgz' },
      });
      assert.equal(npmLockProblems(lock, []).length, 1);
    });

    it('a_link_or_a_local_package_outside_the_workspaces_is_refused', () => {
      const lock = npmLock({
        elsewhere: { name: 'elsewhere' },
        'node_modules/elsewhere': { link: true, resolved: 'elsewhere' },
      });
      assert.equal(npmLockProblems(lock, ['app/ui']).length, 2);
    });

    it('an_old_npm_lock_is_refused', () => {
      assert.equal(npmLockProblems(JSON.stringify({ lockfileVersion: 1 }), []).length, 1);
    });

    it('a_crate_from_another_registry_or_git_is_refused', () => {
      const git = crate(`source = "git+https://example.com/serde#abc"\nchecksum = "${CHECKSUM}"`);
      const other = crate(`source = "registry+https://example.com/index"\nchecksum = "${CHECKSUM}"`);
      assert.equal(cargoLockProblems(git + other).length, 2);
    });

    it('a_crate_without_a_checksum_is_refused', () => {
      const problems = cargoLockProblems(crate('source = "registry+https://github.com/rust-lang/crates.io-index"'));
      assert.deepEqual(
        problems.map((problem) => problem.message),
        ['serde 1.0.0: no checksum'],
      );
    });
  });

  describe('positive cases', () => {
    it('registry_packages_and_workspaces_pass', () => {
      const lock = npmLock({
        'app/ui': { name: '@fresher/ui' },
        'node_modules/@fresher/ui': { link: true, resolved: 'app/ui' },
        'node_modules/react': REACT,
        'node_modules/react/node_modules/scheduler': { ...REACT, resolved: `${REACT.resolved}?s` },
      });
      assert.deepEqual(npmLockProblems(lock, ['app/ui']), []);
    });

    it('crates_io_crates_and_workspace_crates_pass', () => {
      const registry = crate(
        `source = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "${CHECKSUM}"`,
      );
      const sparse = crate(`source = "sparse+https://index.crates.io/"\nchecksum = "${CHECKSUM}"`);
      const workspace = crate('dependencies = [\n "serde",\n]');
      assert.deepEqual(cargoLockProblems(`version = 4\n\n${registry}${sparse}${workspace}`), []);
    });
  });
});
