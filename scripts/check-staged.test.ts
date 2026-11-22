import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { findViolations, MAX_FILE_SIZE, MAX_GIF_SIZE } from './check-staged.ts';

describe('findViolations', () => {
  describe('negative cases', () => {
    it('rejects a file over 1 MiB', () => {
      const violations = findViolations([{ path: 'docs/big.png', size: MAX_FILE_SIZE + 1 }]);

      assert.equal(violations.length, 1);
      assert.match(violations[0]?.reason ?? '', /over 1 MiB/);
    });

    it('rejects a demo GIF over 5 MiB', () => {
      const violations = findViolations([{ path: 'assets/demo/overview.gif', size: MAX_GIF_SIZE + 1 }]);

      assert.match(violations[0]?.reason ?? '', /demo GIF is over 5 MiB/);
    });

    it('rejects a 2 MiB GIF outside the demo folder', () => {
      assert.equal(findViolations([{ path: 'docs/overview.gif', size: 2 * MAX_FILE_SIZE }]).length, 1);
    });

    it('rejects a 2 MiB PNG in the demo folder', () => {
      assert.equal(findViolations([{ path: 'assets/demo/overview.png', size: 2 * MAX_FILE_SIZE }]).length, 1);
    });

    it('rejects a database file of any size', () => {
      assert.equal(findViolations([{ path: 'snapshots.sqlite', size: 10 }]).length, 1);
      assert.equal(findViolations([{ path: 'data/state.db', size: 10 }]).length, 1);
    });

    it('rejects an environment file', () => {
      assert.equal(findViolations([{ path: '.env', size: 10 }]).length, 1);
      assert.equal(findViolations([{ path: 'app/.env.local', size: 10 }]).length, 1);
    });

    it('rejects a signing key', () => {
      assert.equal(findViolations([{ path: 'certs/developer.p12', size: 10 }]).length, 1);
    });
  });

  describe('positive cases', () => {
    it('accepts ordinary files', () => {
      assert.deepEqual(
        findViolations([
          { path: 'crates/core/src/lib.rs', size: 2000 },
          { path: 'docs/goal.md', size: MAX_FILE_SIZE },
        ]),
        [],
      );
    });

    it('accepts a 4 MiB GIF in the demo folder', () => {
      assert.deepEqual(findViolations([{ path: 'assets/demo/overview.gif', size: 4 * MAX_FILE_SIZE }]), []);
    });

    it('accepts the environment example', () => {
      assert.deepEqual(findViolations([{ path: '.env.example', size: 10 }]), []);
    });

    it('accepts the recorded size exception', () => {
      assert.deepEqual(findViolations([{ path: 'app/tauri/icons/icon.icns', size: MAX_FILE_SIZE * 2 }]), []);
    });
  });
});
