import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  bundleProblems,
  defaultBundle,
  homeProblems,
  networkImports,
  networkProblems,
  type Run,
  signatureFlags,
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

    it('the_default_bundle_is_under_the_cargo_target_folder', () => {
      assert.equal(defaultBundle('/repo', {}), '/repo/target/release/bundle/macos/Fresher.app');
      assert.equal(defaultBundle('/repo', { CARGO_TARGET_DIR: '/t' }), '/t/release/bundle/macos/Fresher.app');
    });
  });
});
