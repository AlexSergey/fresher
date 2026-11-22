import { execFileSync } from 'node:child_process';
import path from 'node:path';

// The staged-files guard of the pre-commit hook (docs/quality.md, "Whole repository";
// docs/restrictions/repository.md, "Generated and protected files").

export type StagedFile = {
  readonly path: string;
  readonly size: number;
};

export type Violation = {
  readonly path: string;
  readonly reason: string;
};

export const MAX_FILE_SIZE = 1024 * 1024;

// Demo GIFs recorded by `npm run demo:record` (docs/hacks/README.md, "Demo GIFs up to 5 MiB").
export const GIF_FOLDER = 'assets/demo/';
export const MAX_GIF_SIZE = 5 * 1024 * 1024;

// Files allowed over the size limit, each with its reason in docs/hacks/README.md.
const SIZE_EXCEPTIONS = new Set(['app/tauri/icons/icon.icns']);

const DATABASE_EXTENSIONS = new Set(['.db', '.sqlite', '.sqlite3']);
const SIGNING_EXTENSIONS = new Set(['.cer', '.mobileprovision', '.p8', '.p12', '.provisionprofile']);

const isEnvFile = (name: string): boolean => (name === '.env' || name.startsWith('.env.')) && name !== '.env.example';

const isDemoGif = (file: StagedFile): boolean =>
  path.dirname(file.path) === GIF_FOLDER.slice(0, -1) && path.extname(file.path).toLowerCase() === '.gif';

const checkFile = (file: StagedFile): null | Violation => {
  const name = path.basename(file.path);
  const extension = path.extname(name).toLowerCase();

  if (DATABASE_EXTENSIONS.has(extension)) {
    return { path: file.path, reason: 'a database file is never committed' };
  }

  if (SIGNING_EXTENSIONS.has(extension)) {
    return { path: file.path, reason: 'a signing key or certificate is never committed' };
  }

  if (isEnvFile(name)) {
    return { path: file.path, reason: 'an environment file is never committed (only .env.example)' };
  }

  if (isDemoGif(file)) {
    return file.size > MAX_GIF_SIZE ? { path: file.path, reason: 'a demo GIF is over 5 MiB' } : null;
  }

  if (file.size > MAX_FILE_SIZE && !SIZE_EXCEPTIONS.has(file.path)) {
    return { path: file.path, reason: `the file is over 1 MiB (${String(file.size)} bytes)` };
  }

  return null;
};

export const findViolations = (files: readonly StagedFile[]): Violation[] =>
  files.map((file) => checkFile(file)).filter((violation) => violation !== null);

const git = (args: readonly string[]): string => execFileSync('git', args, { encoding: 'utf8' });

const readStagedFiles = (): StagedFile[] =>
  git(['diff', '--cached', '--name-only', '--diff-filter=ACMR', '-z'])
    .split('\0')
    .filter((filePath) => filePath !== '')
    .map((filePath) => ({ path: filePath, size: Number(git(['cat-file', '-s', `:${filePath}`]).trim()) }));

if (import.meta.main) {
  const violations = findViolations(readStagedFiles());

  for (const violation of violations) {
    process.stderr.write(`check-staged: ${violation.path}: ${violation.reason}\n`);
  }

  process.exitCode = violations.length > 0 ? 1 : 0;
}
