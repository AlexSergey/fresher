import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { type Command, parseArgs, plan, run, UsageError } from './sandbox-dev.ts';

const ROOT = '/repo';
const CWD = '/repo/work';
const at = (commands: readonly Command[], index: number): Command => {
  const command = commands[index];
  assert.ok(command);

  return command;
};
const missing = (): boolean => false;
const present = (): boolean => true;

describe('sandbox-dev', () => {
  describe('negative cases', () => {
    it('an_unknown_flag_is_refused', () => {
      assert.throws(() => parseArgs(['trial', '--fast']), UsageError);
    });

    it('no_argument_is_refused', () => {
      assert.throws(() => parseArgs([]), UsageError);
      assert.throws(() => parseArgs(['--real-trash']), UsageError);
    });

    it('two_targets_are_refused', () => {
      assert.throws(() => parseArgs(['a', 'b']), UsageError);
    });

    it('dot_and_dot_dot_are_not_feature_names', () => {
      assert.throws(() => parseArgs(['.']), UsageError);
      assert.throws(() => parseArgs(['..']), UsageError);
      assert.throws(() => parseArgs(['a b']), UsageError);
    });

    it('a_failing_command_stops_the_rest', () => {
      const seen: Command[] = [];
      const commands = plan({ realTrash: false, target: 'trial' }, ROOT, CWD, missing);
      const code = run(commands, (command) => {
        seen.push(command);

        return 3;
      });
      assert.equal(code, 3);
      assert.equal(seen.length, 1);
    });
  });

  describe('positive cases', () => {
    it('a_feature_name_points_into_the_sandbox_folder', () => {
      const dev = at(plan(parseArgs(['trial']), ROOT, CWD, present), 0);
      assert.deepEqual(dev.args, ['run', 'dev']);
      assert.equal(dev.env.FRESHER_SANDBOX, '/repo/sandbox/trial');
    });

    it('a_missing_feature_is_generated_first', () => {
      const commands = plan(parseArgs(['trial']), ROOT, CWD, missing);
      assert.deepEqual(at(commands, 0).args, ['run', 'sandbox', '--', 'trial']);
      assert.equal(commands.length, 2);
    });

    it('a_path_is_used_as_it_is_and_never_generated', () => {
      const commands = plan(parseArgs(['./mine']), ROOT, CWD, missing);
      assert.equal(commands.length, 1);
      assert.equal(at(commands, 0).env.FRESHER_SANDBOX, '/repo/work/mine');
    });

    it('without_real_trash_the_trash_variable_is_removed_from_the_child', () => {
      const dev = at(plan(parseArgs(['trial']), ROOT, CWD, present), 0);
      assert.ok('FRESHER_SANDBOX_TRASH' in dev.env);
      assert.equal(dev.env.FRESHER_SANDBOX_TRASH, undefined);
    });

    it('a_relative_path_resolves_against_the_given_start_folder', () => {
      const dev = at(plan(parseArgs(['./mine']), ROOT, '/start', missing), 0);
      assert.equal(dev.env.FRESHER_SANDBOX, '/start/mine');
    });

    it('real_trash_sets_its_variable', () => {
      const dev = at(plan(parseArgs(['trial', '--real-trash']), ROOT, CWD, present), 0);
      assert.equal(dev.env.FRESHER_SANDBOX_TRASH, 'real');
    });
  });
});
