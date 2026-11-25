import type { Access, AccessStatus, Capability, Coverage } from '../ipc/bindings.generated';

// A capability that kept the scan from seeing something: its state when the scan ran (`null` for a capability known
// only from unreadable folders) and how many folders it would let the app read.
export type NotSeen = {
  readonly access: Access | null;
  readonly capability: Capability;
  readonly folders: number;
};

const valueOf = (capability: Capability): string | undefined => ('value' in capability ? capability.value : undefined);

const same = (a: Capability, b: Capability): boolean => a.kind === b.kind && valueOf(a) === valueOf(b);

// The state of one capability, or `null` when the core does not list it (or has not answered yet).
export const accessOf = (status: AccessStatus | null, capability: Capability): Access | null =>
  status?.capabilities.find((state) => same(state.capability, capability))?.access ?? null;

// What the scan did not see because of a capability, one entry per capability, in the core's order: the places it did
// not enter, then the capabilities named by unreadable folders (those the app could read with it).
export const notSeen = (coverage: Coverage | null): readonly NotSeen[] => {
  const found: NotSeen[] = [];

  for (const { access, capability } of coverage?.notEntered ?? []) {
    found.push({ access, capability, folders: 0 });
  }
  for (const { folders, unlockedBy } of coverage?.unreadable ?? []) {
    if (unlockedBy === null) {
      continue;
    }
    const index = found.findIndex((entry) => same(entry.capability, unlockedBy));
    const entry = found[index];

    if (entry === undefined) {
      found.push({ access: null, capability: unlockedBy, folders });
    } else {
      found[index] = { ...entry, folders: entry.folders + folders };
    }
  }

  return found;
};

// How many folders could not be read for a reason no capability fixes (another user's, or changed during the scan).
export const unexplainedFolders = (coverage: Coverage | null): number =>
  (coverage?.unreadable ?? []).reduce((sum, { folders, unlockedBy }) => (unlockedBy === null ? sum + folders : sum), 0);
