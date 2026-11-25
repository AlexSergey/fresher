import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

// The crate dependency guard (docs/quality.md, "Whole repository"): every edge between workspace crates must be in the
// table of docs/plans/002-base-architecture/00-design.md, "Crates", mirrored in docs/architecture/README.md; and every
// crate root but fresher-macos's forbids unsafe code (plan 004/03/01); no module's source names `std::process` or
// `std::env`, reaches them another way (an alias of `std`, `extern crate`, `include!`, `#[path]`), depends on a
// third-party crate outside its short list, or has a build script (plan 007/04/02, plan 041); and no code of `crates/`
// or `modules/` starts an external program by its bare name, which the system would look up in `PATH` (plan 041).

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
  ['fresher-sandbox', ['fresher-core', 'fresher-macos', 'fresher-store', 'fresher-testing']],
  ['fresher-scanner', ['fresher-core', 'fresher-macos']],
  ['fresher-store', ['fresher-core']],
  ['fresher-system', ['fresher-core', 'fresher-macos']],
  ['fresher-testing', ['fresher-core']],
]);

const MODULE_ALLOWED: Allowed = ['fresher-core', 'fresher-rules'];
// The third-party crates a module may use outside its tests: none of them starts a process or reads the environment.
const MODULE_THIRD_PARTY: ReadonlySet<string> = new Set(['serde', 'serde_json', 'thiserror', 'toml', 'tracing']);

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

const checkModuleExtras = (pkg: MetadataPackage, workspace: ReadonlySet<string>): Violation[] => {
  if (!isModule(pkg.name)) {
    return [];
  }
  const thirdParty = pkg.dependencies
    .filter((dependency) => dependency.kind !== 'dev' && !workspace.has(dependency.name))
    .filter((dependency) => !MODULE_THIRD_PARTY.has(dependency.name))
    .map((dependency) => ({
      crate: pkg.name,
      reason: `a module depends on no third-party crate outside its list (${dependency.name}, scripts/check-crate-deps.ts)`,
    }));
  const buildScripts = pkg.targets
    .filter((target) => target.kind.includes('custom-build'))
    .map((target) => ({ crate: pkg.name, reason: `a module has no build script (${target.path})` }));

  return [...thirdParty, ...buildScripts];
};

export const findViolations = (metadata: Metadata): Violation[] => {
  const workspace = new Set(metadata.packages.map((pkg) => pkg.name));

  return metadata.packages.flatMap((pkg) => [...checkPackage(pkg, workspace), ...checkModuleExtras(pkg, workspace)]);
};

// The inner attributes that open a crate root, comments left out: they come before any item, so a string or a
// commented-out line is never taken for one.
const leadingInnerAttributes = (source: string): string[] => {
  const attributes: string[] = [];
  let rest = withoutComments(source).trimStart();

  while (rest.startsWith('#![')) {
    let depth = 0;
    let end = 2;

    for (; end < rest.length; end += 1) {
      depth += rest[end] === '[' ? 1 : rest[end] === ']' ? -1 : 0;
      if (depth === 0) {
        break;
      }
    }
    attributes.push(rest.slice(0, end + 1).replace(/\s+/g, ''));
    rest = rest.slice(end + 1).trimStart();
  }

  return attributes;
};

const forbidsUnsafe = (source: string): boolean => leadingInnerAttributes(source).includes(FORBID_UNSAFE);

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

// Other ways to reach `std::process` or `std::env`, or code from outside the module's `src`: an alias of `std`
// (`use std as s;`, `extern crate std as s;`, `use std::{self as s}`), any `extern crate`, `include!` and `#[path]`.
const FORBIDDEN_WAYS: readonly RegExp[] = [
  /\bstd\s+as\b/,
  /\bextern\s+crate\b/,
  /\binclude\s*!/,
  /#\s*(?:!\s*)?\[\s*path\b/,
];

const namesForbidden = (source: string): boolean => {
  const code = withoutComments(source);

  return (
    FORBIDDEN_PATH.test(code) ||
    FORBIDDEN_WAYS.some((way) => way.test(code)) ||
    stdGroups(code).some((group) => FORBIDDEN_NAME.test(group) || /\bself\s+as\b/.test(group))
  );
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
        .map((file) => ({
          crate: pkg.name,
          reason: `${file.path} names std::process or std::env, or reaches them or other code another way`,
        })),
    );

// The program of `Command::new(...)` or of the command port's `.run(...)`: a string, or a constant of the same file.
const PROGRAM_CALL = /(?:\bCommand::new|\.run)\(\s*(?:"([^"]*)"|([A-Z][A-Z0-9_]*))\s*[,)]/g;
const STR_CONSTANT = /\bconst\s+([A-Z][A-Z0-9_]*)\s*:\s*&\s*(?:'static\s+)?str\s*=\s*"([^"]*)"/g;
const TEST_MODULE = /#\[cfg\(test\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{/g;

// The code without its `#[cfg(test)] mod … { … }` blocks: tests may run a program by name.
const withoutTestModules = (code: string): string => {
  let kept = '';
  let from = 0;

  for (const match of code.matchAll(TEST_MODULE)) {
    if (match.index < from) {
      continue;
    }
    let depth = 1;
    let end = match.index + match[0].length;

    while (depth > 0 && end < code.length) {
      depth += code[end] === '{' ? 1 : code[end] === '}' ? -1 : 0;
      end += 1;
    }
    kept += code.slice(from, match.index);
    from = end;
  }

  return kept + code.slice(from);
};

const isTestFile = (path: string): boolean => /(?:^|\/)tests(?:\.rs$|\/)/.test(path);

// The programs a source starts by their bare name.
export const bareProgramsOf = (source: string): string[] => {
  const code = withoutTestModules(withoutComments(source));
  const constants = new Map([...code.matchAll(STR_CONSTANT)].map((match) => [match[1] ?? '', match[2] ?? '']));

  return [...code.matchAll(PROGRAM_CALL)]
    .map((match) => match[1] ?? constants.get(match[2] ?? ''))
    .filter((program): program is string => program !== undefined && !program.startsWith('/'));
};

const isCoreOrModule = (path: string): boolean => /(?:^|\/)(?:crates|modules)\/[^/]+\/src\//.test(path);

export const findBareProgramViolations = (
  metadata: Metadata,
  sourcesOf: (srcFolder: string) => readonly SourceFile[],
): Violation[] =>
  metadata.packages.flatMap((pkg) =>
    [
      ...new Set(
        pkg.targets
          .filter((target) => target.kind.some((kind) => ROOT_KINDS.has(kind)) && isCoreOrModule(target.path))
          .map((target) => dirname(target.path)),
      ),
    ]
      .flatMap((folder) => sourcesOf(folder))
      .filter((file) => !isTestFile(file.path))
      .flatMap((file) =>
        bareProgramsOf(file.text).map((program) => ({
          crate: pkg.name,
          reason: `${file.path} starts \`${program}\` by its bare name, looked up in PATH: give its absolute path`,
        })),
      ),
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
    ...findBareProgramViolations(metadata, rustFiles),
  ];

  for (const violation of violations) {
    process.stderr.write(`check-crate-deps: ${violation.crate}: ${violation.reason}\n`);
  }

  process.exitCode = violations.length > 0 ? 1 : 0;
}
