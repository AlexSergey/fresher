import type { ComponentType } from 'react';

import type { CategoryId, ModuleId, ModuleInfo } from '../ipc/bindings.generated';

// What a module's page is given by the app frame: the module as the core describes it, and the category it was
// opened at, if any (the page scrolls to it and expands it).
export type ModuleViewProps = {
  readonly category?: CategoryId | undefined;
  readonly module: ModuleInfo;
};

// The modules whose page is their own view instead of the generic one (screens/module/module-screen.tsx). A module
// with a custom view registers it here, by its id; every module uses the generic page for now.
export const customViews: Partial<Readonly<Record<ModuleId, ComponentType<ModuleViewProps>>>> = {};
