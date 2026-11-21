import { createContext, use } from 'react';

import type { CategoryId, ModuleInfo } from '../ipc/bindings.generated';

// The modules the core lists (`get_modules`), in registration order; `null` until it has answered.
export const ModulesContext = createContext<null | readonly ModuleInfo[]>(null);

export const useModules = (): null | readonly ModuleInfo[] => use(ModulesContext);

// The module that lists a category; `undefined` while the modules are not known or when none lists it.
export const moduleOfCategory = (modules: null | readonly ModuleInfo[], category: CategoryId): ModuleInfo | undefined =>
  modules?.find(({ categories }) => categories.some(({ descriptor }) => descriptor.id === category));
