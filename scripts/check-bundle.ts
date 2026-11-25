import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// The bundle check (docs/restrictions/privacy.md, "No foreign code under the app's access" and "No network"): the built
// `Fresher.app` is signed as a whole (ad hoc, no Apple account for 0.1.0) with the hardened runtime and no
// entitlements, so the system refuses injected code (`DYLD_INSERT_LIBRARIES`) in a process that holds Full Disk
// Access, and a downloaded copy is not reported as damaged; and its executable imports nothing that opens a network
// connection or resolves a host name, nor the builder's home folder (`npm run build` remaps it to `~`). `npm run
// build` runs it after `tauri build`.

export type Run = (args: readonly string[]) => {
  readonly status: number;
  readonly stderr: string;
  readonly stdout: string;
};

export const CODESIGN = '/usr/bin/codesign';
export const NM = '/usr/bin/nm';

// The app's executable inside the bundle (`mainBinaryName` defaults to the crate's binary).
const EXECUTABLE = join('Contents', 'MacOS', 'fresher-app');

// Imports of the executable that would open a connection or resolve a name: the socket calls, the resolver, and the
// system's networking APIs. The web view's own loading is WebKit's, not the executable's, and is held by the CSP.
export const NETWORK_IMPORTS: ReadonlySet<string> = new Set([
  '_CFSocketCreate',
  '_CFStreamCreatePairWithSocketToHost',
  '_connect',
  '_getaddrinfo',
  '_gethostbyname',
  '_nw_connection_create',
  '_OBJC_CLASS_$_NSURLConnection',
  '_OBJC_CLASS_$_NSURLSession',
  '_socket',
]);

const USAGE = 'usage: npm run check:bundle [-- <path to Fresher.app>]';

// The flags of the code directory in `codesign --display`: `CodeDirectory v=20500 … flags=0x10002(adhoc,runtime) …`.
const FLAGS = /^CodeDirectory .*\bflags=0x[0-9a-f]+\(([^)]*)\)/mu;

export const signatureFlags = (display: string): readonly string[] =>
  FLAGS.exec(display)?.[1]
    ?.split(',')
    .filter((flag) => flag !== '') ?? [];

export const bundleProblems = (codesign: Run, app: string): readonly string[] => {
  const problems: string[] = [];
  const verify = codesign(['--verify', '--deep', '--strict', '--verbose=2', app]);
  if (verify.status !== 0) {
    problems.push(`the signature does not verify (codesign --verify --deep --strict): ${verify.stderr.trim()}`);
  }
  const display = codesign(['--display', '--verbose=2', app]);
  if (display.status !== 0) {
    problems.push(`the bundle is not signed: ${display.stderr.trim()}`);
  } else if (!signatureFlags(display.stderr).includes('runtime')) {
    problems.push('the signature has no hardened runtime (codesign --display shows no `runtime` flag)');
  }
  const entitlements = codesign(['--display', '--entitlements', '-', '--xml', app]);
  if (entitlements.status === 0 && entitlements.stdout.trim() !== '') {
    problems.push('the signature carries entitlements; none is allowed without a reason in the docs');
  }

  return problems;
};

// `nm -u` lists one undefined symbol per line.
export const networkImports = (undefinedSymbols: string): readonly string[] =>
  undefinedSymbols
    .split('\n')
    .map((line) => line.trim())
    .filter((symbol) => NETWORK_IMPORTS.has(symbol));

export const networkProblems = (nm: Run, app: string): readonly string[] => {
  const listed = nm(['-u', executableOf(app)]);
  if (listed.status !== 0) {
    return [`the executable's imports could not be listed (nm -u): ${listed.stderr.trim()}`];
  }

  return networkImports(listed.stdout).map((symbol) => `the executable imports ${symbol}, a network call`);
};

// The builder's home folder in the executable (source paths of panics): it names the account and makes the build
// depend on where it ran.
export const homeProblems = (executable: Buffer, home: string | undefined): readonly string[] =>
  home !== undefined && home !== '' && executable.includes(Buffer.from(`${home}/`))
    ? ['the executable holds paths of the home folder it was built in; build it with npm run build']
    : [];

export const executableOf = (app: string): string => join(app, EXECUTABLE);

export const defaultBundle = (root: string, env: NodeJS.ProcessEnv): string =>
  join(env.CARGO_TARGET_DIR ?? join(root, 'target'), 'release', 'bundle', 'macos', 'Fresher.app');

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const args = process.argv.slice(2);
  if (args.length > 1) {
    process.stderr.write(`check-bundle: ${USAGE}\n`);
    process.exitCode = 2;
  } else {
    const app = args[0] ?? defaultBundle(root, process.env);
    const runner =
      (program: string): Run =>
      (programArgs) => {
        const result = spawnSync(program, [...programArgs], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });

        return { status: result.status ?? 1, stderr: result.stderr, stdout: result.stdout };
      };
    const problems = [...bundleProblems(runner(CODESIGN), app), ...networkProblems(runner(NM), app)];
    try {
      problems.push(...homeProblems(readFileSync(executableOf(app)), process.env.HOME));
    } catch (error) {
      problems.push(`the executable could not be read: ${String(error)}`);
    }
    for (const problem of problems) {
      process.stderr.write(`check-bundle: ${app}: ${problem}\n`);
    }
    process.exitCode = problems.length > 0 ? 1 : 0;
  }
}
