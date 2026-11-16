import { createContext, use } from 'react';

import type { ModuleInfo, Overview, Sizes } from '../ipc/bindings.generated';

import { summarize } from '../state/selection-summary';

// The modules the core lists (`get_modules`), in registration order; `null` until it has answered.
export const ModulesContext = createContext<null | readonly ModuleInfo[]>(null);

export const useModules = (): null | readonly ModuleInfo[] => use(ModulesContext);

// What a module found: the overview's totals of its categories, summed with the core's rule. `null` before any scan
// (nothing was looked for yet, so no size, not 0); during a scan it grows with each new overview.
export const moduleFound = (module: ModuleInfo, overview: null | Overview): null | Sizes => {
  if (overview === null || (overview.lastScan === null && overview.status.state !== 'scanning')) {
    return null;
  }

  // The module's categories, each selected whole: the summary's sum is the core's sum of their totals.
  const categories = new Map(module.categories.map((category) => [category.id, new Map()] as const));

  return summarize({ categories, findings: new Map() }, overview.totals).sizes;
};
