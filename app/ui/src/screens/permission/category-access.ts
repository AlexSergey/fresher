import type {
  AccessStatus,
  Availability,
  Capability,
  CategoryId,
  CategoryTotal,
  ModuleInfo,
} from '../../ipc/bindings.generated';

import { t, tText } from '../../i18n';
import { accessOf } from '../../state/access';
import { idOf } from './capability-id';

// How a category's availability reads: `limited` still has a size, partial, and a tooltip naming what was not seen;
// `unavailable` is shown disabled with the one sentence that says why. `missing` is what the core says is missing.
export type CategoryAccess = {
  readonly kind: 'limited' | 'unavailable';
  readonly missing: readonly Capability[];
  readonly text: string;
};

const text = (id: string, part: string): string => tText({ args: {}, key: `core.access.${id}.${part}` });

const toolName = (capability: Capability): string => (capability.kind === 'tool' ? capability.value : '');

// One capability's name, for "Some of it could not be seen: ...".
const nameOf = (capability: Capability): string => {
  const id = idOf(capability);

  return id === null ? toolName(capability) : text(id, 'name');
};

// One capability's sentence of why it is missing: the core's text, or for a tool the note of its state.
export const missingSentence = (capability: Capability, status: AccessStatus | null): string => {
  const id = idOf(capability);

  if (id !== null) {
    return text(id, 'missing');
  }
  const note = status?.capabilities.find(
    (state) => state.capability.kind === 'tool' && state.capability.value === toolName(capability),
  )?.note;

  return note === null || note === undefined
    ? t('ui.access.tool-missing', { tool: { kind: 'text', value: toolName(capability) } })
    : tText(note);
};

// The sentences of everything missing, as one text.
export const missingText = (missing: readonly Capability[], status: AccessStatus | null): string =>
  missing.map((capability) => missingSentence(capability, status)).join(' ');

// `undefined` for a ready category, so a card or section with no access text shows none.
export const categoryAccess = (
  availability: Availability | undefined,
  status: AccessStatus | null,
): CategoryAccess | undefined => {
  if (availability === undefined || availability.kind === 'ready') {
    return undefined;
  }
  const { missing } = availability;

  return availability.kind === 'limited'
    ? {
        kind: 'limited',
        missing,
        text: t('ui.access.limited', { names: { kind: 'text', value: missing.map(nameOf).join(', ') } }),
      }
    : { kind: 'unavailable', missing, text: missingText(missing, status) };
};

// The access of every category of every module that is not ready, from the core's availability.
export const accessOfCategories = (
  modules: null | readonly ModuleInfo[],
  status: AccessStatus | null,
): ReadonlyMap<CategoryId, CategoryAccess> => {
  const all = new Map<CategoryId, CategoryAccess>();

  for (const { categories } of modules ?? []) {
    for (const { availability, descriptor } of categories) {
      const access = categoryAccess(availability, status);

      if (access !== undefined) {
        all.set(descriptor.id, access);
      }
    }
  }

  return all;
};

// A zero total for each unavailable category that has no total of its own (it found nothing, so the core has none),
// so it is never silently absent.
export const unavailableTotals = (
  modules: null | readonly ModuleInfo[],
  totals: readonly CategoryTotal[],
  access: ReadonlyMap<CategoryId, CategoryAccess>,
): readonly CategoryTotal[] =>
  (modules ?? []).flatMap(({ categories }) =>
    categories.flatMap(({ descriptor }) =>
      access.get(descriptor.id)?.kind === 'unavailable' && !totals.some(({ category }) => category === descriptor.id)
        ? [
            {
              category: descriptor.id,
              count: 0,
              highestDanger: descriptor.danger,
              sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
            } satisfies CategoryTotal,
          ]
        : [],
    ),
  );

// Whether the state of the capability is not worth showing: it does not exist on this Mac.
export const notApplicable = (status: AccessStatus | null, capability: Capability): boolean =>
  accessOf(status, capability) === 'not-applicable';
