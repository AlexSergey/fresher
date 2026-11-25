import type { CategoryIcon, CategoryId, ModuleInfo } from '../../ipc/bindings.generated';

// The icon the core's descriptor names for a category, found in the module that lists it; `generic` for a category no
// listed module has (the core's own) or while the modules are not loaded yet.
export const iconOf = (modules: null | readonly ModuleInfo[], category: CategoryId): CategoryIcon => {
  for (const module of modules ?? []) {
    const found = module.categories.find((info) => info.descriptor.id === category);

    if (found !== undefined) {
      return found.descriptor.icon;
    }
  }

  return 'generic';
};
