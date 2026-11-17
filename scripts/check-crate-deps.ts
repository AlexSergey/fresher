import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

// The crate dependency guard (docs/quality.md, "Whole repository"): every edge between workspace crates must be in the
// table of docs/plans/002-base-architecture/00-design.md, "Crates", mirrored in docs/architecture/README.md; and every
// crate root but fresher-macos's forbids unsafe code (plan 004/03/01); and no module's source names `std::process` or
// `std::env` (plan 007/04/02).

export type DependencyKind = 'build' | 'dev' | 'normal';

export type Metadata = {
  readonly packages: readonly MetadataPackage[];
};

export type MetadataDependency = {
  readonly kind: 'build' | 'dev' | null;
  readonly name: string;
};

export type MetadataPackage = {
  readonly dependencies: readonly MetadataDependency[];
  readonly name: string;
  readonly targets: readonly MetadataTarget[];
};

// A target of a package: its kinds (`lib`, `bin`, `test`, ...) and its source file (`src_path` in `cargo metadata`).
export type MetadataTarget = {
  readonly kind: readonly string[];
  readonly path: string;
};

export type Violation = {
  readonly crate: string;
  readonly reason: string;
};

type Allowed = 'anything' | readonly string[];

type CargoMetadata = {
  readonly packages: readonly (Omit<MetadataPackage, 'targets'> & {
    readonly targets: readonly { readonly kind: readonly string[]; readonly src_path: string }[];
  })[];
};

const MODULE_PREFIX = 'fresher-module-';
// The only crate allowed unsafe code, in its `ffi` module; every other crate root carries FORBID_UNSAFE.
const UNSAFE_CRATE = 'fresher-macos';
const FORBID_UNSAFE = '#![forbid(unsafe_code)]';
// The targets whose source file is a crate root; tests, benches, examples and build scripts are not checked.
const ROOT_KINDS: ReadonlySet<string> = new Set(['bin', 'cdylib', 'dylib', 'lib', 'proc-macro', 'rlib', 'staticlib']);
const TESTING = 'fresher-testing';

// Who may depend on whom; a crate that does not exist yet keeps its row, so later plans add crates without editing it.
const TABLE: ReadonlyMap<string, Allowed> = new Map<string, Allowed>([
  ['fresher-app', 'anything'],
  ['fresher-cleaner', ['fresher-core', 'fresher-macos']],
  ['fresher-core', []],
  ['fresher-e2e', 'anything'],
  ['fresher-engine', ['fresher-core']],
  ['fresher-macos', ['fresher-core']],
  ['fresher-rules', ['fresher-core']],
  ['fresher-sandbox', ['fresher-core', 'fresher-macos', 'fresher-testing']],
  ['fresher-scanner', ['fresher-core', 'fresher-macos']],
  ['fresher-store', ['fresher-core']],
  ['fresher-system', ['fresher-core', 'fresher-macos']],
  ['fresher-testing', ['fresher-core']],
]);

const MODULE_ALLOWED: Allowed = ['fresher-core', 'fresher-rules'];

const isModule = (name: string): boolean => name.startsWith(MODULE_PREFIX);

const allowedFor = (name: string): Allowed | undefined => (isModule(name) ? MODULE_ALLOWED : TABLE.get(name));

const isAllowed = (allowed: Allowed, dependency: string, kind: DependencyKind): boolean => {
  if (allowed === 'anything') {
    return true;
  }

  return allowed.includes(dependency) || (kind === 'dev' && dependency === TESTING);
};

const checkPackage = (pkg: MetadataPackage, workspace: ReadonlySet<string>): Violation[] => {
  const allowed = allowedFor(pkg.name);

  if (allowed === undefined) {
    return [{ crate: pkg.name, reason: 'the crate is not in the dependency table (scripts/check-crate-deps.ts)' }];
  }

  return pkg.dependencies
    .filter((dependency) => workspace.has(dependency.name))
    .map((dependency) => ({ kind: dependency.kind ?? ('normal' as const), name: dependency.name }))
    .filter(({ kind, name }) => !isAllowed(allowed, name, kind))
    .map(({ kind, name }) => ({
      crate: pkg.name,
      reason:
        isModule(pkg.name) && isModule(name)
          ? `a module never depends on another module (${name})`
          : `the ${kind} dependency on ${name} is outside the dependency table`,
    }));
};

export const findViolations = (metadata: Metadata): Violation[] => {
  const workspace = new Set(metadata.packages.map((pkg) => pkg.name));

  return metadata.packages.flatMap((pkg) => checkPackage(pkg, workspace));
};

const forbidsUnsafe = (source: string): boolean => source.split('\n').some((line) => line.trim() === FORBID_UNSAFE);

export const findUnsafeViolations = (metadata: Metadata, readSource: (path: string) => string): Violation[] =>
  metadata.packages
    .filter((pkg) => pkg.name !== UNSAFE_CRATE)
    .flatMap((pkg) =>
      pkg.targets
        .filter((target) => target.kind.some((kind) => ROOT_KINDS.has(kind)))
        .filter((target) => !forbidsUnsafe(readSource(target.path)))
        .map((target) => ({ crate: pkg.name, reason: `the crate root ${target.path} lacks ${FORBID_UNSAFE}` })),
    );

// A module starts no process and reads no environment: it runs a tool through `CommandRunner` and gets its places from
// the context. Comment lines are skipped, so a doc may name them.
const FORBIDDEN_PATH = /\bstd::\s*(?:process|env)\b/;
const FORBIDDEN_NAME = /\b(?:process|env)\b/;

const withoutComments = (source: string): string => source.replace(/\/\*[\s\S]*?\*\//g, ' ').replace(/\/\/.*$/gm, '');

// The text of every `std::{ ... }` group, nested groups included.
const stdGroups = (source: string): string[] => {
  const groups: string[] = [];

  for (const match of source.matchAll(/\bstd::\s*\{/g)) {
    let depth = 1;
    let end = match.index + match[0].length;

    while (depth > 0 && end < source.length) {
      depth += source[end] === '{' ? 1 : source[end] === '}' ? -1 : 0;
      end += 1;
    }
    groups.push(source.slice(match.index + match[0].length, end));
  }

  return groups;
};

const namesForbidden = (source: string): boolean => {
  const code = withoutComments(source);

  return FORBIDDEN_PATH.test(code) || stdGroups(code).some((group) => FORBIDDEN_NAME.test(group));
};

// `sourcesOf` lists a module's `src` folder: every `.rs` file, as its path and its text.
export type SourceFile = { readonly path: string; readonly text: string };

export const findModuleViolations = (
  metadata: Metadata,
  sourcesOf: (srcFolder: string) => readonly SourceFile[],
): Violation[] =>
  metadata.packages
    .filter((pkg) => isModule(pkg.name))
    .flatMap((pkg) =>
      [
        ...new Set(
          pkg.targets.filter((target) => target.kind.some((kind) => ROOT_KINDS.has(kind))).map((t) => dirname(t.path)),
        ),
      ]
        .flatMap((folder) => sourcesOf(folder))
        .filter((file) => namesForbidden(file.text))
        .map((file) => ({ crate: pkg.name, reason: `${file.path} names std::process or std::env` })),
    );

const rustFiles = (folder: string): SourceFile[] =>
  readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
    const path = join(folder, entry.name);

    if (entry.isDirectory()) {
      return rustFiles(path);
    }

    return entry.name.endsWith('.rs') ? [{ path, text: readFileSync(path, 'utf8') }] : [];
  });

const readMetadata = (): Metadata => {
  const cargo = JSON.parse(
    execFileSync('cargo', ['metadata', '--format-version', '1', '--no-deps', '--locked'], {
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
    }),
  ) as CargoMetadata;

  return {
    packages: cargo.packages.map((pkg) => ({
      ...pkg,
      targets: pkg.targets.map((target) => ({ kind: target.kind, path: target.src_path })),
    })),
  };
};

if (import.meta.main) {
  const metadata = readMetadata();
  const violations = [
    ...findViolations(metadata),
    ...findUnsafeViolations(metadata, (path) => readFileSync(path, 'utf8')),
    ...findModuleViolations(metadata, rustFiles),
  ];

  for (const violation of violations) {
    process.stderr.write(`check-crate-deps: ${violation.crate}: ${violation.reason}\n`);
  }

  process.exitCode = violations.length > 0 ? 1 : 0;
}
