import { createContext, use, useSyncExternalStore } from 'react';

import type { CategoryId, ModuleId } from '../ipc/bindings.generated';

export type Navigation = {
  readonly getScreen: () => Screen;
  readonly navigate: (screen: Screen) => void;
  readonly subscribe: (listener: () => void) => () => void;
};

// The screen the app shows; no router: the app frame renders the one that is current. A module's page may be opened
// at one of its categories (from the overview's cards): the page scrolls to it and expands it.
export type Screen =
  | { readonly category?: CategoryId; readonly kind: 'module'; readonly module: ModuleId }
  | { readonly kind: 'overview' }
  | { readonly kind: 'review' }
  | { readonly kind: 'settings' };

const OVERVIEW: Screen = { kind: 'overview' };

export const createNavigation = (): Navigation => {
  let screen: Screen = OVERVIEW;
  const listeners = new Set<() => void>();

  return {
    getScreen: () => screen,
    navigate: (next): void => {
      screen = next;
      for (const listener of listeners) {
        listener();
      }
    },
    subscribe: (listener): (() => void) => {
      listeners.add(listener);

      return (): void => {
        listeners.delete(listener);
      };
    },
  };
};

// The screen to show: a module screen of a module the core no longer lists (`modules`, `null` while they are not
// known yet) falls back to the overview.
export const screenToShow = (screen: Screen, modules: null | readonly ModuleId[]): Screen =>
  screen.kind === 'module' && modules !== null && !modules.includes(screen.module) ? OVERVIEW : screen;

export const NavigationContext = createContext<Navigation | null>(null);

export const useNavigation = (): Navigation => {
  const navigation = use(NavigationContext);

  if (navigation === null) {
    throw new Error('useScreen needs a NavigationContext around it');
  }

  return navigation;
};

// The current screen, among the modules the core lists; the component renders again when it changes.
export const useScreen = (modules: null | readonly ModuleId[]): Screen => {
  const navigation = useNavigation();

  return screenToShow(useSyncExternalStore(navigation.subscribe, navigation.getScreen), modules);
};
