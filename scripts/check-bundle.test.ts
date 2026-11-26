import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  appProblems,
  archProblems,
  bundleProblems,
  defaultBundle,
  dmgProblems,
  homeProblems,
  networkImports,
  networkProblems,
  releaseBundle,
  releaseImage,
  type Run,
  signatureFlags,
  type Tools,
} from './check-bundle.ts';

type Answer = { readonly status: number; readonly stderr: string; readonly stdout: string };

const ok = (stderr = '', stdout = ''): Answer => ({ status: 0, stderr, stdout });

const SIGNED = [
  'Executable=/x/Fresher.app/Contents/MacOS/fresher-app',
  'CodeDirectory v=20500 size=21085 flags=0x10002(adhoc,runtime) hashes=652+3 location=embedded',
  'Executable Segment flags=0x1',
  'Signature=adhoc',
].join('\n');
const LINKER_SIGNED = [
  'Executable=/x/fresher-app',
  'CodeDirectory v=20400 size=21085 flags=0x20002(adhoc,linker-signed) hashes=652+0 location=embedded',
  'Executable Segment flags=0x1',
].join('\n');

// A fake codesign: `verify`, `display` and `entitlements` answer the three calls.
const codesign =
  (answers: { display?: Answer; entitlements?: Answer; verify?: Answer }): Run =>
  (args) => {
    if (args[0] === '--verify') {
      return answers.verify ?? ok();
    }

    return args.includes('--entitlements') ? (answers.entitlements ?? ok()) : (answers.display ?? ok(SIGNED));
  };

const lipoSaying =
  (archs: string): Run =>
  () =>
    ok('', `${archs}\n`);

// A fake hdiutil that answers each subcommand and records the calls.
const hdiutil =
  (calls: string[], answers: { attach?: Answer; detach?: Answer; verify?: Answer } = {}): Run =>
  (args) => {
    calls.push(args.join(' '));
    const answer = answers[args[0] as 'attach' | 'detach' | 'verify'];

    return answer ?? ok();
  };

const TOOLS: Tools = { codesign: codesign({}), lipo: lipoSaying('x86_64 arm64'), nm: () => ok('', '_open\n') };
const EXECUTABLE = (): Buffer => Buffer.from('\0~/.cargo/registry/src/x.rs\0');
const ALL_FILES = (): boolean => true;

describe('check-bundle', () => {
  describe('negative cases', () => {
    it('a_bundle_whose_strict_verify_fails_is_refused', () => {
      const failing = {
        status: 1,
        stderr: 'code has no resources but signature indicates they must be present',
        stdout: '',
      };
      const problems = bundleProblems(codesign({ verify: failing }), '/x/Fresher.app');
      assert.equal(problems.length, 1);
      assert.match(problems[0] ?? '', /does not verify.*no resources/u);
    });

    it('a_signature_without_the_hardened_runtime_is_refused', () => {
      const problems = bundleProblems(codesign({ display: ok(LINKER_SIGNED) }), '/x/Fresher.app');
      assert.deepEqual(problems, [
        'the signature has no hardened runtime (codesign --display shows no `runtime` flag)',
      ]);
    });

    it('an_unsigned_bundle_is_refused', () => {
      const unsigned = { status: 1, stderr: 'code object is not signed at all', stdout: '' };
      const problems = bundleProblems(codesign({ display: unsigned, verify: unsigned }), '/x/Fresher.app');
      assert.equal(problems.length, 2);
    });

    it('entitlements_are_refused', () => {
      const plist =
        '<?xml version="1.0"?><plist><dict><key>com.apple.security.cs.allow-jit</key><true/></dict></plist>';
      const problems = bundleProblems(codesign({ entitlements: ok('', plist) }), '/x/Fresher.app');
      assert.equal(problems.length, 1);
      assert.match(problems[0] ?? '', /entitlements/u);
    });

    it('an_executable_importing_a_network_call_is_refused', () => {
      const nm: Run = () => ok('', '_open\n_getaddrinfo\n_OBJC_CLASS_$_NSURLSession\n_read\n');
      assert.deepEqual(networkProblems(nm, '/x/Fresher.app'), [
        'the executable imports _getaddrinfo, a network call',
        'the executable imports _OBJC_CLASS_$_NSURLSession, a network call',
      ]);
    });

    it('an_executable_whose_imports_cannot_be_listed_is_refused', () => {
      const nm: Run = () => ({ status: 1, stderr: 'No such file', stdout: '' });
      assert.equal(networkProblems(nm, '/x/Fresher.app').length, 1);
    });

    it('an_executable_holding_the_home_folder_is_refused', () => {
      const executable = Buffer.from('\0panicked at /Users/someone/.cargo/registry/src/x.rs\0');
      assert.equal(homeProblems(executable, '/Users/someone').length, 1);
    });

    it('an_executable_without_intel_or_apple_silicon_code_is_refused', () => {
      assert.deepEqual(archProblems(lipoSaying('arm64'), '/x/Fresher.app'), [
        'the executable has no x86_64 code (lipo -archs: arm64); build it with npm run build:release',
      ]);
      assert.equal(archProblems(lipoSaying('x86_64'), '/x/Fresher.app').length, 1);
    });

    it('an_executable_whose_architectures_cannot_be_listed_is_refused', () => {
      const lipo: Run = () => ({ status: 1, stderr: 'not a Mach-O file', stdout: '' });
      assert.match(archProblems(lipo, '/x/Fresher.app')[0] ?? '', /lipo -archs.*not a Mach-O/u);
    });

    it('a_release_app_is_held_to_both_architectures_and_every_other_check', () => {
      const tools = { ...TOOLS, lipo: lipoSaying('arm64') };
      assert.equal(appProblems(tools, '/x/Fresher.app', EXECUTABLE, '/Users/someone', true).length, 1);
      assert.deepEqual(appProblems(tools, '/x/Fresher.app', EXECUTABLE, '/Users/someone', false), []);
      const unreadable = (): Buffer => {
        throw new Error('ENOENT');
      };
      assert.match(appProblems(TOOLS, '/x/Fresher.app', unreadable, '/u', true)[0] ?? '', /could not be read/u);
    });

    it('a_disk_image_that_does_not_verify_is_refused_and_not_opened', () => {
      const calls: string[] = [];
      const failing = { status: 1, stderr: 'checksum mismatch', stdout: '' };
      const problems = dmgProblems(hdiutil(calls, { verify: failing }), '/x/F.dmg', '/m', () => [], ALL_FILES);
      assert.deepEqual(problems, ['the disk image does not verify (hdiutil verify): checksum mismatch']);
      assert.deepEqual(calls, ['verify -quiet /x/F.dmg']);
    });

    it('a_disk_image_that_cannot_be_opened_is_refused', () => {
      const calls: string[] = [];
      const failing = { status: 1, stderr: 'no mountable file systems', stdout: '' };
      const problems = dmgProblems(hdiutil(calls, { attach: failing }), '/x/F.dmg', '/m', () => [], ALL_FILES);
      assert.equal(problems.length, 1);
      assert.equal(calls.length, 2);
    });

    it('an_app_in_the_disk_image_that_fails_a_check_is_refused_and_the_image_is_closed', () => {
      const calls: string[] = [];
      const checked: string[] = [];
      const problems = dmgProblems(
        hdiutil(calls),
        '/x/F.dmg',
        '/m',
        (app) => {
          checked.push(app);

          return ['the executable has no x86_64 code'];
        },
        ALL_FILES,
      );
      assert.deepEqual(problems, ['inside the disk image: the executable has no x86_64 code']);
      assert.deepEqual(checked, ['/m/Fresher.app']);
      assert.equal(calls.at(-1), 'detach /m');
    });

    it('a_disk_image_whose_window_is_not_laid_out_is_refused_and_the_image_is_closed', () => {
      const calls: string[] = [];
      const problems = dmgProblems(
        hdiutil(calls),
        '/x/F.dmg',
        '/m',
        () => [],
        (path) => path !== '/m/.DS_Store',
      );
      assert.deepEqual(problems, ["the disk image's window is not laid out: no .DS_Store (did Finder's step run?)"]);
      assert.equal(calls.at(-1), 'detach /m');
      const asked: string[] = [];
      dmgProblems(
        hdiutil([]),
        '/x/F.dmg',
        '/m',
        () => [],
        (path) => {
          asked.push(path);

          return false;
        },
      );
      assert.deepEqual(asked, ['/m/.DS_Store', '/m/.background/background.png']);
    });

    it('a_disk_image_left_open_is_said', () => {
      const failing = { status: 1, stderr: 'resource busy', stdout: '' };
      assert.equal(dmgProblems(hdiutil([], { detach: failing }), '/x/F.dmg', '/m', () => [], ALL_FILES).length, 1);
    });

    it('no_disk_image_or_more_than_one_is_not_a_release_image', () => {
      assert.equal(releaseImage([]), undefined);
      assert.equal(releaseImage(['Fresher_0.1.0_aarch64.dmg', 'notes.txt']), undefined);
      assert.equal(releaseImage(['Fresher_0.1.0_universal.dmg', 'Fresher_0.2.0_universal.dmg']), undefined);
    });

    it('a_display_without_flags_has_none', () => {
      assert.deepEqual(signatureFlags('Executable=/x\nExecutable Segment flags=0x1\n'), []);
    });
  });

  describe('positive cases', () => {
    it('an_ad_hoc_bundle_with_the_hardened_runtime_passes', () => {
      assert.deepEqual(bundleProblems(codesign({}), '/x/Fresher.app'), []);
    });

    it('an_executable_without_network_imports_passes_and_its_path_is_inside_the_bundle', () => {
      const asked: string[] = [];
      const nm: Run = (args) => {
        asked.push(...args);

        return ok('', '_open\n_read\n_socketpair\n_OBJC_CLASS_$_NSURL\n');
      };
      assert.deepEqual(networkProblems(nm, '/x/Fresher.app'), []);
      assert.deepEqual(asked, ['-u', '/x/Fresher.app/Contents/MacOS/fresher-app']);
      assert.deepEqual(networkImports(''), []);
    });

    it('an_executable_with_remapped_paths_passes', () => {
      const executable = Buffer.from('\0panicked at ~/.cargo/registry/src/x.rs\0/Users/someone-else/a\0');
      assert.deepEqual(homeProblems(executable, '/Users/someone'), []);
      assert.deepEqual(homeProblems(executable, undefined), []);
    });

    it('the_flags_are_read_from_the_display', () => {
      assert.deepEqual(signatureFlags(SIGNED), ['adhoc', 'runtime']);
    });

    it('a_universal_executable_passes', () => {
      assert.deepEqual(archProblems(lipoSaying('x86_64 arm64'), '/x/Fresher.app'), []);
      assert.deepEqual(appProblems(TOOLS, '/x/Fresher.app', EXECUTABLE, '/Users/someone', true), []);
    });

    it('a_disk_image_that_verifies_and_holds_a_passing_app_passes', () => {
      const calls: string[] = [];
      assert.deepEqual(
        dmgProblems(hdiutil(calls), '/x/F.dmg', '/m', () => [], ALL_FILES),
        [],
      );
      assert.deepEqual(calls, [
        'verify -quiet /x/F.dmg',
        'attach -readonly -nobrowse -noautoopen -mountpoint /m /x/F.dmg',
        'detach /m',
      ]);
    });

    it('the_one_universal_disk_image_is_the_release_image', () => {
      assert.equal(
        releaseImage(['Fresher_0.1.0_universal.dmg', 'bundle_dmg.sh', 'icon.icns']),
        'Fresher_0.1.0_universal.dmg',
      );
    });

    it('the_default_bundle_is_under_the_cargo_target_folder', () => {
      assert.equal(defaultBundle('/repo', {}), '/repo/target/release/bundle/macos/Fresher.app');
      assert.equal(defaultBundle('/repo', { CARGO_TARGET_DIR: '/t' }), '/t/release/bundle/macos/Fresher.app');
      assert.equal(releaseBundle('/repo', {}), '/repo/target/universal-apple-darwin/release/bundle');
      assert.equal(releaseBundle('/repo', { CARGO_TARGET_DIR: '/t' }), '/t/universal-apple-darwin/release/bundle');
    });
  });
});
