import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  appCommand,
  BUDGET,
  captureCommand,
  checkGif,
  encodeCommands,
  generateCommand,
  judge,
  killGroup,
  parseArgs,
  parseGenerated,
  pollWindow,
  readResult,
  readWindow,
  scenarioNames,
  trimSeconds,
} from './demo-record.ts';
import { UsageError } from './sandbox-dev.ts';

const RECT = { height: 1440, scale: 2, width: 2200, x: 160, y: 160 };
const MIB = 1024 * 1024;

describe('demo-record', () => {
  describe('negative cases', () => {
    it('no_target_is_refused', () => {
      assert.throws(() => parseArgs([]), UsageError);
      assert.throws(() => parseArgs(['--check']), UsageError);
    });

    it('two_targets_are_refused', () => {
      assert.throws(() => parseArgs(['a', 'b']), UsageError);
    });

    it('an_unknown_flag_is_refused', () => {
      assert.throws(() => parseArgs(['--fast', 'a']), UsageError);
    });

    it('a_name_with_a_slash_is_refused', () => {
      assert.throws(() => parseArgs(['a/b']), UsageError);
    });

    it('an_unknown_scenario_is_refused_by_name', () => {
      assert.throws(() => scenarioNames('nope', () => ['overview']), /nope/);
    });

    it('a_generator_without_a_folder_line_is_refused', () => {
      assert.throws(() => parseGenerated('nothing here\n'));
    });

    it('a_window_file_with_a_missing_field_is_refused', () => {
      assert.throws(() => readWindow('{"x":1,"y":1,"width":10,"height":10}'), /demo-window\.json/);
    });

    it('a_window_file_with_a_zero_size_is_refused', () => {
      assert.throws(() => readWindow('{"x":1,"y":1,"width":0,"height":10,"scale":2}'), /demo-window\.json/);
    });

    it('a_window_file_that_is_not_json_is_refused', () => {
      assert.throws(() => readWindow('not json'), /demo-window\.json/);
    });

    it('no_result_file_is_missing', () => {
      assert.deepEqual(readResult(null), { kind: 'missing' });
    });

    it('a_missing_result_fails_the_judge', () => {
      assert.deepEqual(judge({ kind: 'missing' }, BUDGET), { message: 'the app ended without a result', ok: false });
    });

    it('a_failed_result_names_the_step', () => {
      const verdict = judge({ kind: 'failed', reason: 'no such button', step: 3 }, BUDGET);
      assert.deepEqual(verdict, { message: 'step 3: no such button', ok: false });
    });

    it('a_run_over_twenty_seconds_fails_the_judge', () => {
      const verdict = judge({ elapsedMs: 20_001, kind: 'finished' }, BUDGET);
      assert.ok(!verdict.ok);
      assert.match(verdict.message, /over the 20 s budget/);
    });

    it('a_window_file_that_never_parses_is_reported_after_the_tries', async () => {
      let reads = 0;
      await assert.rejects(
        pollWindow(
          () => {
            reads += 1;

            return '{"x":';
          },
          () => Promise.resolve(),
          () => false,
          3,
        ),
        /demo-window\.json/,
      );
      assert.equal(reads, 3);
    });

    it('a_wait_that_is_over_without_a_file_gives_nothing', async () => {
      assert.equal(
        await pollWindow(
          () => null,
          () => Promise.resolve(),
          () => true,
        ),
        null,
      );
    });

    it('a_group_that_is_gone_is_not_an_error', () => {
      assert.doesNotThrow(() =>
        killGroup(10, () => {
          throw new Error('ESRCH');
        }),
      );
    });

    it('no_pid_signals_nothing', () => {
      killGroup(undefined, () => {
        assert.fail('must not signal');
      });
    });

    it('a_gif_over_five_mib_is_refused', () => {
      const verdict = checkGif(5 * MIB + 1);
      assert.ok(!verdict.ok);
      assert.match(verdict.message, /over the 5 MiB budget; shorten the scenario/);
    });
  });

  describe('positive cases', () => {
    it('a_half_written_window_file_is_read_again', async () => {
      const texts = ['{"x":', JSON.stringify(RECT)];
      const rect = await pollWindow(
        () => texts.shift() ?? null,
        () => Promise.resolve(),
        () => false,
      );
      assert.deepEqual(rect, RECT);
    });

    it('the_whole_process_group_is_signalled', () => {
      const seen: unknown[] = [];
      killGroup(42, (pid, signal) => {
        seen.push([pid, signal]);
      });
      assert.deepEqual(seen, [[-42, 'SIGTERM']]);
    });

    it('check_mode_is_parsed', () => {
      assert.deepEqual(parseArgs(['--check', 'overview']), { mode: 'check', target: 'overview' });
      assert.deepEqual(parseArgs(['all']), { mode: 'record', target: 'all' });
    });

    it('all_lists_the_stems_sorted', () => {
      assert.deepEqual(
        scenarioNames('all', () => ['b', 'a', 'c']),
        ['a', 'b', 'c'],
      );
    });

    it('the_plan_is_the_generator_then_the_app_with_the_exact_environment', () => {
      assert.deepEqual(generateCommand('overview'), {
        args: ['run', '-q', '-p', 'fresher-sandbox', '--locked', '--', 'demo', 'overview'],
        env: {},
        program: 'cargo',
      });
      const folder = parseGenerated('wrote the demo world developer-mac into /private/tmp/fresher-demo/overview-2\n');
      assert.equal(folder, '/private/tmp/fresher-demo/overview-2');
      assert.deepEqual(appCommand('/repo', 'overview', folder), {
        args: ['run', 'dev'],
        env: {
          FRESHER_DEMO: '/repo/fixtures/demos/scenarios/overview.toml',
          FRESHER_SANDBOX: '/private/tmp/fresher-demo/overview-2',
          FRESHER_SANDBOX_TRASH: undefined,
        },
        program: 'npm',
      });
    });

    it('the_window_file_is_read', () => {
      assert.deepEqual(readWindow(JSON.stringify(RECT)), RECT);
    });

    it('the_result_file_is_read', () => {
      assert.deepEqual(readResult('{"kind":"finished","elapsedMs":12000}'), { elapsedMs: 12_000, kind: 'finished' });
      assert.deepEqual(readResult('{"kind":"failed","step":2,"reason":"x"}'), { kind: 'failed', reason: 'x', step: 2 });
    });

    it('the_capture_command_is_exact', () => {
      assert.deepEqual(captureCommand('out.mkv').args, [
        '-hide_banner',
        '-loglevel',
        'error',
        '-y',
        '-f',
        'avfoundation',
        '-capture_cursor',
        '0',
        '-framerate',
        '30',
        '-i',
        'Capture screen 0:none',
        '-c:v',
        'libx264',
        '-preset',
        'ultrafast',
        '-qp',
        '0',
        '-pix_fmt',
        'yuv444p',
        'out.mkv',
      ]);
    });

    it('the_encode_commands_are_exact', () => {
      const commands = encodeCommands('r.mkv', RECT, 1.5, 'a.mp4', 'p.png', 'a.gif');
      assert.deepEqual(
        commands.map((command) => [command.program, ...command.args]),
        [
          [
            'ffmpeg',
            '-y',
            '-ss',
            '1.500',
            '-i',
            'r.mkv',
            '-vf',
            'crop=2200:1440:160:160',
            '-c:v',
            'libx264',
            '-pix_fmt',
            'yuv420p',
            '-crf',
            '18',
            '-movflags',
            '+faststart',
            'a.mp4',
          ],
          [
            'ffmpeg',
            '-y',
            '-i',
            'a.mp4',
            '-vf',
            'fps=12,scale=800:-1:flags=lanczos,palettegen=stats_mode=diff',
            'p.png',
          ],
          [
            'ffmpeg',
            '-y',
            '-i',
            'a.mp4',
            '-i',
            'p.png',
            '-lavfi',
            'fps=12,scale=800:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle',
            'a.gif',
          ],
        ],
      );
    });

    it('the_trim_keeps_half_a_second_of_the_lead_in', () => {
      assert.equal(trimSeconds(1000, 3000), 3);
      assert.equal(trimSeconds(1000, 1000), 1);
    });

    it('the_trim_is_never_negative', () => {
      assert.equal(trimSeconds(5000, 0), 0);
    });

    it('a_run_of_twelve_seconds_passes_the_judge', () => {
      assert.deepEqual(judge({ elapsedMs: 12_000, kind: 'finished' }, BUDGET), { ok: true, seconds: 12 });
    });

    it('a_gif_of_exactly_five_mib_passes', () => {
      assert.deepEqual(checkGif(5 * MIB), { ok: true });
    });
  });
});
