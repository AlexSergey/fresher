import { createContext, use, useSyncExternalStore } from 'react';

import type { CategoryId, CleanupId, Descent, ModuleId, NodeId, NodeInfo } from '../ipc/bindings.generated';

export type Navigation = {
  readonly getOverviewView: () => OverviewView;
  readonly getScreen: () => Screen;
  readonly navigate: (screen: Screen) => void;
  readonly showOverviewView: (view: OverviewView) => void;
  readonly subscribe: (listener: () => void) => () => void;
};

// How the overview shows the space, kept for the session: by meaning, or by place at a folder of a volume.
export type OverviewView = PlaceAt | { readonly kind: 'meaning' };

// The folder the place view has open: `node` (`null`: the volume's first root, or its roots when it holds several),
// opened through its single-child folders or exactly at it, under `trail`, the folders the user came down through.
export type PlaceAt = {
  readonly descent: Descent;
  readonly kind: 'place';
  readonly node: NodeId | null;
  readonly trail: readonly NodeInfo[];
  readonly volume: number;
};

// The place view at the top of volume `volume`.
export const placeTop = (volume: number): PlaceAt => ({
  descent: 'through',
  kind: 'place',
  node: null,
  trail: [],
  volume,
});

// The screen the app shows; no router: the app frame renders the one that is current. A module's page may be opened
// at one of its categories (from the overview's cards): the page scrolls to it and expands it. `cleanups` is the
// history of cleanups, `cleanup` one of them.
export type Screen =
  | { readonly category?: CategoryId; readonly kind: 'module'; readonly module: ModuleId }
  | { readonly cleanup: CleanupId; readonly kind: 'cleanup' }
  | { readonly kind: 'cleanups' }
  | { readonly kind: 'overview' }
  | { readonly kind: 'review' }
  | { readonly kind: 'settings' };

const OVERVIEW: Screen = { kind: 'overview' };

const BY_MEANING: OverviewView = { kind: 'meaning' };

export const createNavigation = (): Navigation => {
  let screen: Screen = OVERVIEW;
  let overviewView: OverviewView = BY_MEANING;
  const listeners = new Set<() => void>();

  const changed = (): void => {
    for (const listener of listeners) {
      listener();
    }
  };

  return {
    getOverviewView: () => overviewView,
    getScreen: () => screen,
    navigate: (next): void => {
      screen = next;
      changed();
    },
    showOverviewView: (view): void => {
      overviewView = view;
      changed();
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

// How the overview shows the space; the component renders again when it changes.
export const useOverviewView = (): OverviewView => {
  const navigation = useNavigation();

  return useSyncExternalStore(navigation.subscribe, navigation.getOverviewView);
};
