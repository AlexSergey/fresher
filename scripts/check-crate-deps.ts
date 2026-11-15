import { execFileSync } from 'node:child_process';

// The crate dependency guard (docs/quality.md, "Whole repository"): every edge between workspace crates must be in the
// table of docs/plans/002-base-architecture/00-design.md, "Crates", mirrored in docs/architecture/README.md.

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
};

export type Violation = {
  readonly crate: string;
  readonly reason: string;
};

type Allowed = 'anything' | readonly string[];

const MODULE_PREFIX = 'fresher-module-';
const TESTING = 'fresher-testing';

// Who may depend on whom; a crate that does not exist yet keeps its row, so later plans add crates without editing it.
const TABLE: ReadonlyMap<string, Allowed> = new Map<string, Allowed>([
  ['fresher-app', 'anything'],
  ['fresher-cleaner', ['fresher-core', 'fresher-macos']],
  ['fresher-core', []],
  ['fresher-e2e', 'anything'],
  ['fresher-engine', ['fresher-core']],
  ['fresher-macos', []],
  ['fresher-rules', ['fresher-core']],
  ['fresher-scanner', ['fresher-core', 'fresher-macos']],
  ['fresher-store', ['fresher-core']],
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

const readMetadata = (): Metadata =>
  JSON.parse(
    execFileSync('cargo', ['metadata', '--format-version', '1', '--no-deps', '--locked'], {
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
    }),
  ) as Metadata;

if (import.meta.main) {
  const violations = findViolations(readMetadata());

  for (const violation of violations) {
    process.stderr.write(`check-crate-deps: ${violation.crate}: ${violation.reason}\n`);
  }

  process.exitCode = violations.length > 0 ? 1 : 0;
}
