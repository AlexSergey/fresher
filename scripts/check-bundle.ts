import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

// The bundle check (docs/restrictions/privacy.md, "No foreign code under the app's access" and "No network"): the built
// `Fresher.app` is signed as a whole (ad hoc, no Apple account for 0.1.0) with the hardened runtime and no
// entitlements, so the system refuses injected code (`DYLD_INSERT_LIBRARIES`) in a process that holds Full Disk
// Access, and a downloaded copy is not reported as damaged; and its executable imports nothing that opens a network
// connection or resolves a host name, nor the builder's home folder (`npm run build` remaps it to `~`). `npm run
// build` runs it after `tauri build`. `npm run build:release` runs it with `--release` on the universal build (plan
// 013/02): the executable holds both architectures, and the disk image verifies and holds an app that passes every
// check above.

export type Run = (args: readonly string[]) => {
  readonly status: number;
  readonly stderr: string;
  readonly stdout: string;
};

export const CODESIGN = '/usr/bin/codesign';
export const HDIUTIL = '/usr/bin/hdiutil';
export const LIPO = '/usr/bin/lipo';
export const NM = '/usr/bin/nm';

// The download runs on Apple silicon and on Intel Macs.
export const UNIVERSAL_ARCHS: readonly string[] = ['arm64', 'x86_64'];

export type Tools = { readonly codesign: Run; readonly lipo: Run; readonly nm: Run };

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

const USAGE = 'usage: npm run check:bundle [-- <path to Fresher.app> | --release]';

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

// `lipo -archs` prints the architectures of the executable on one line: `x86_64 arm64`.
export const archProblems = (lipo: Run, app: string): readonly string[] => {
  const listed = lipo(['-archs', executableOf(app)]);
  if (listed.status !== 0) {
    return [`the executable's architectures could not be listed (lipo -archs): ${listed.stderr.trim()}`];
  }
  const archs = listed.stdout.trim().split(/\s+/u);

  return UNIVERSAL_ARCHS.filter((arch) => !archs.includes(arch)).map(
    (arch) =>
      `the executable has no ${arch} code (lipo -archs: ${listed.stdout.trim()}); build it with npm run build:release`,
  );
};

export const appProblems = (
  tools: Tools,
  app: string,
  readExecutable: (path: string) => Buffer,
  home: string | undefined,
  universal: boolean,
): readonly string[] => {
  const problems = [...bundleProblems(tools.codesign, app), ...networkProblems(tools.nm, app)];
  if (universal) {
    problems.push(...archProblems(tools.lipo, app));
  }
  try {
    problems.push(...homeProblems(readExecutable(executableOf(app)), home));
  } catch (error) {
    problems.push(`the executable could not be read: ${String(error)}`);
  }

  return problems;
};

// The disk image verifies, opens read-only without showing in Finder, and holds `Fresher.app`, which `checkApp`
// checks where it lies in the image; the image is detached again whatever the checks say.
export const dmgProblems = (
  hdiutil: Run,
  dmg: string,
  mountpoint: string,
  checkApp: (app: string) => readonly string[],
): readonly string[] => {
  const verify = hdiutil(['verify', '-quiet', dmg]);
  if (verify.status !== 0) {
    return [`the disk image does not verify (hdiutil verify): ${verify.stderr.trim()}`];
  }
  const attach = hdiutil(['attach', '-readonly', '-nobrowse', '-noautoopen', '-mountpoint', mountpoint, dmg]);
  if (attach.status !== 0) {
    return [`the disk image could not be opened (hdiutil attach): ${attach.stderr.trim()}`];
  }
  const problems = checkApp(join(mountpoint, 'Fresher.app')).map((problem) => `inside the disk image: ${problem}`);
  const detach = hdiutil(['detach', mountpoint]);
  if (detach.status !== 0) {
    problems.push(`the disk image could not be closed (hdiutil detach): ${detach.stderr.trim()}`);
  }

  return problems;
};

// The one disk image of a release build: `Fresher_<version>_universal.dmg`.
export const releaseImage = (names: readonly string[]): string | undefined => {
  const images = names.filter((name) => /^Fresher_\d+\.\d+\.\d+_universal\.dmg$/u.test(name));

  return images.length === 1 ? images[0] : undefined;
};

export const executableOf = (app: string): string => join(app, EXECUTABLE);

export const defaultBundle = (root: string, env: NodeJS.ProcessEnv): string =>
  join(env.CARGO_TARGET_DIR ?? join(root, 'target'), 'release', 'bundle', 'macos', 'Fresher.app');

// Where `npm run build:release` writes the universal build.
export const releaseBundle = (root: string, env: NodeJS.ProcessEnv): string =>
  join(env.CARGO_TARGET_DIR ?? join(root, 'target'), 'universal-apple-darwin', 'release', 'bundle');

const runner =
  (program: string): Run =>
  (programArgs) => {
    const result = spawnSync(program, [...programArgs], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });

    return { status: result.status ?? 1, stderr: result.stderr, stdout: result.stdout };
  };

const hostProblems = (tools: Tools, app: string, home: string | undefined): readonly string[] =>
  appProblems(tools, app, readFileSync, home, false).map((problem) => `${app}: ${problem}`);

const releaseProblems = (tools: Tools, bundle: string, home: string | undefined): readonly string[] => {
  const app = join(bundle, 'macos', 'Fresher.app');
  const problems = appProblems(tools, app, readFileSync, home, true).map((problem) => `${app}: ${problem}`);
  const folder = join(bundle, 'dmg');
  let names: readonly string[] = [];
  try {
    names = readdirSync(folder);
  } catch {
    // No folder: no image, said below.
  }
  const image = releaseImage(names);
  if (image === undefined) {
    return [...problems, `${folder}: not one Fresher_<version>_universal.dmg (found: ${names.join(', ') || 'none'})`];
  }
  const dmg = join(folder, image);
  const mountpoint = mkdtempSync(join(tmpdir(), 'fresher-check-bundle-'));
  try {
    problems.push(
      ...dmgProblems(runner(HDIUTIL), dmg, mountpoint, (inside) =>
        appProblems(tools, inside, readFileSync, home, true),
      ).map((problem) => `${dmg}: ${problem}`),
    );
  } finally {
    // Only the empty folder: never anything inside an image left open (its failed detach is already said).
    try {
      rmdirSync(mountpoint);
    } catch {
      // Left to the system's cleaning of the temporary folder.
    }
  }

  return problems;
};

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const args = process.argv.slice(2);
  const tools: Tools = { codesign: runner(CODESIGN), lipo: runner(LIPO), nm: runner(NM) };
  if (args.length > 1) {
    process.stderr.write(`check-bundle: ${USAGE}\n`);
    process.exitCode = 2;
  } else {
    const problems =
      args[0] === '--release'
        ? releaseProblems(tools, releaseBundle(root, process.env), process.env.HOME)
        : hostProblems(tools, args[0] ?? defaultBundle(root, process.env), process.env.HOME);
    for (const problem of problems) {
      process.stderr.write(`check-bundle: ${problem}\n`);
    }
    process.exitCode = problems.length > 0 ? 1 : 0;
  }
}
