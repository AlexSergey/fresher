import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import * as prettier from 'prettier';

import {
  buildNotices,
  checkAboutConfig,
  npmEntries,
  type NpmPackage,
  parsePolicy,
  type Problem,
  rustEntries,
} from './notices-build.ts';

// `npm run notices:generate` writes the licence notices the app shows under Settings → About, from the Rust crates
// the app is built from (cargo-about, offline) and the UI's npm production packages (node_modules); `npm run
// check:notices` fails when the committed file differs or a licence is outside deny.toml's allowlist (plan
// 013/03/01). The only file-system, process and Prettier code.

// cargo-about's JSON report of the app crate's tree; injected by the tests.
export type CargoAboutRun = (root: string) => string;

export type Mode = 'check' | 'generate';

const SCRIPT = 'notices';

export const NOTICES_PATH = 'app/ui/src/screens/settings/notices.generated.json';

// Offline (`--frozen`): no crate is fetched and no licence is looked up on the network, so the texts are the ones of
// the crates in the local registry and the result is the same on every machine.
const cargoAbout: CargoAboutRun = (root) =>
  execFileSync(
    'cargo',
    [
      'about',
      'generate',
      '--format',
      'json',
      '--frozen',
      '--fail',
      '--config',
      'about.toml',
      '--manifest-path',
      'app/tauri/Cargo.toml',
    ],
    { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, stdio: ['ignore', 'pipe', 'inherit'] },
  );

type LockPackage = {
  readonly dev?: boolean;
  readonly devOptional?: boolean;
  readonly link?: boolean;
  readonly version?: string;
};

const LICENCE_FILE = /^(?:licen[cs]e|copying)/iu;

// The packages of package-lock.json that are no development package: the UI's production tree, as installed.
const readNpmPackages = (root: string): { packages: NpmPackage[]; problems: Problem[] } => {
  const lock = JSON.parse(readFileSync(path.join(root, 'package-lock.json'), 'utf8')) as {
    readonly packages: Readonly<Record<string, LockPackage>>;
  };
  const packages: NpmPackage[] = [];
  const problems: Problem[] = [];

  for (const [key, entry] of Object.entries(lock.packages)) {
    if (!key.startsWith('node_modules/') || entry.dev === true || entry.devOptional === true || entry.link === true) {
      continue;
    }
    const name = key.slice(key.lastIndexOf('node_modules/') + 'node_modules/'.length);
    const folder = path.join(root, key);
    const manifestPath = path.join(folder, 'package.json');
    const manifest = existsSync(manifestPath)
      ? (JSON.parse(readFileSync(manifestPath, 'utf8')) as { readonly license?: unknown; readonly version?: string })
      : undefined;
    if (manifest === undefined || entry.version === undefined || manifest.version !== entry.version) {
      problems.push({
        kind: 'not-installed',
        message: `node_modules does not hold ${entry.version ?? 'its version'}: run \`npm ci\``,
        where: `npm ${name}`,
      });
      continue;
    }
    const files = new Map(
      readdirSync(folder)
        .filter((file) => LICENCE_FILE.test(file))
        .map((file) => [file, readFileSync(path.join(folder, file), 'utf8')] as const),
    );
    packages.push({ files, licence: manifest.license, name, version: entry.version });
  }

  return { packages, problems };
};

const formatJson = async (text: string, filePath: string): Promise<string> => {
  const config = await prettier.resolveConfig(filePath);

  return prettier.format(text, { ...config, filepath: filePath });
};

export const run = async (
  mode: Mode,
  root: string,
  runCargoAbout: CargoAboutRun = cargoAbout,
): Promise<readonly Problem[]> => {
  const policy = parsePolicy(readFileSync(path.join(root, 'deny.toml'), 'utf8'));
  const configProblems = checkAboutConfig(readFileSync(path.join(root, 'about.toml'), 'utf8'), policy);
  const rust = rustEntries(JSON.parse(runCargoAbout(root)) as Parameters<typeof rustEntries>[0], policy);
  const installed = readNpmPackages(root);
  const npm = npmEntries(installed.packages, policy);
  const problems = [...configProblems, ...rust.problems, ...installed.problems, ...npm.problems];

  // Notices built from a refused tree are neither written nor compared.
  if (problems.length > 0) {
    return problems;
  }

  const noticesPath = path.join(root, NOTICES_PATH);
  const formatted = await formatJson(
    JSON.stringify(buildNotices([...rust.entries, ...npm.entries], rust.sources)),
    noticesPath,
  );
  if (mode === 'generate') {
    writeFileSync(noticesPath, formatted);

    return [];
  }

  const committed = existsSync(noticesPath) ? readFileSync(noticesPath, 'utf8') : '';

  return committed === formatted
    ? []
    : [
        {
          kind: 'stale-notices',
          message: 'the licence notices differ from what `npm run notices:generate` writes',
          where: NOTICES_PATH,
        },
      ];
};

const parseMode = (argument: string | undefined): Mode | undefined =>
  argument === 'check' || argument === 'generate' ? argument : undefined;

if (import.meta.main) {
  const mode = parseMode(process.argv[2]);

  if (mode === undefined) {
    process.stderr.write(`${SCRIPT}: usage: node scripts/notices.ts <generate | check>\n`);
    process.exitCode = 2;
  } else {
    const problems = await run(mode, path.resolve(import.meta.dirname, '..'));

    for (const problem of problems) {
      process.stderr.write(`${problem.where}: ${problem.kind}: ${problem.message}\n`);
    }

    process.exitCode = problems.length > 0 ? 1 : 0;
  }
}
