import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { buildFacts, devCommand, type Git } from './dev.ts';
import { UsageError } from './sandbox-dev.ts';

const FACTS = { commit: '26e4cb0', dirty: false };
const answering =
  (head: { status: number; stdout: string }, tree: { status: number; stdout: string }): Git =>
  (args) =>
    args[0] === 'rev-parse' ? head : tree;

describe('dev', () => {
  describe('negative cases', () => {
    it('a_failing_git_gives_unknown_and_clean', () => {
      const failing = answering({ status: 128, stdout: 'x\n' }, { status: 128, stdout: ' M a\n' });
      assert.deepEqual(buildFacts(failing), { commit: 'unknown', dirty: false });
    });

    it('the_command_is_the_fixed_tauri_dev_and_never_npm_run_dev', () => {
      const command = devCommand('/repo', FACTS, []);
      assert.deepEqual(command.args, ['exec', '--', 'tauri', 'dev', '--config', 'app/tauri/tauri.dev.conf.json']);
      assert.notDeepEqual(command.args.slice(0, 2), ['run', 'dev']);
    });

    it('extra_arguments_are_refused', () => {
      assert.throws(() => devCommand('/repo', FACTS, ['--release']), UsageError);
      assert.throws(() => devCommand('/repo', FACTS, ['--foo']), UsageError);
    });
  });

  describe('positive cases', () => {
    it('the_record_folder_is_inside_the_repository', () => {
      assert.equal(devCommand('/repo', FACTS, []).env.FRESHER_DEV_RUNS, '/repo/NO_COMMIT/dev-runs');
    });

    it('the_commit_is_the_trimmed_short_head', () => {
      const facts = buildFacts(answering({ status: 0, stdout: '26e4cb0\n' }, { status: 0, stdout: '' }));
      assert.equal(facts.commit, '26e4cb0');
      assert.equal(devCommand('/repo', facts, []).env.FRESHER_BUILD_COMMIT, '26e4cb0');
    });

    it('a_changed_tree_is_dirty_and_a_clean_one_is_not', () => {
      const head = { status: 0, stdout: '26e4cb0\n' };
      const dirty = buildFacts(answering(head, { status: 0, stdout: ' M a\n' }));
      const clean = buildFacts(answering(head, { status: 0, stdout: '' }));
      assert.equal(devCommand('/repo', dirty, []).env.FRESHER_BUILD_DIRTY, '1');
      assert.equal(devCommand('/repo', clean, []).env.FRESHER_BUILD_DIRTY, '0');
    });
  });
});
