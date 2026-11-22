import { createContext, use, useSyncExternalStore } from 'react';

import type { CategoryId, CleanupId, ModuleId } from '../ipc/bindings.generated';

// The tabs of Settings' Ignored section.
export type IgnoredTab = 'groups' | 'items' | 'modules';

export type Navigation = {
  readonly getScreen: () => Screen;
  readonly navigate: (screen: Screen) => void;
  readonly subscribe: (listener: () => void) => () => void;
};

// The screen the app shows; no router: the app frame renders the one that is current. `category` is a category's
// details, opened from its card on Scan; Scan's `from` is the category whose details the user comes back from, so the
// focus returns to that card (`fromInsight`: the same for an insight, `module/identity`). `cleanups` is the history of cleanups, `cleanup` one of them.
// Settings may be opened on a tab of its Ignored section.
export type Screen =
  | { readonly category: CategoryId; readonly kind: 'category' }
  | { readonly cleanup: CleanupId; readonly kind: 'cleanup' }
  | { readonly from?: CategoryId; readonly fromInsight?: string; readonly kind: 'scan' }
  | { readonly identity: string; readonly kind: 'insight'; readonly module: ModuleId }
  | { readonly kind: 'cleanups' }
  | { readonly kind: 'review' }
  | { readonly kind: 'settings'; readonly tab?: IgnoredTab };

const START: Screen = { kind: 'scan' };

export const createNavigation = (): Navigation => {
  let screen: Screen = START;
  const listeners = new Set<() => void>();

  const changed = (): void => {
    for (const listener of listeners) {
      listener();
    }
  };

  return {
    getScreen: () => screen,
    navigate: (next): void => {
      screen = next;
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

// An insight a module described in the last scan.
export type InsightRef = { readonly identity: string; readonly module: ModuleId };

// The screen to show: the details of a category no module the core lists has (`categories`, every listed module's,
// `null` while they are not known yet) and an insight the last scan did not describe (`insights`) fall back to Scan.
export const screenToShow = (
  screen: Screen,
  categories: null | readonly CategoryId[],
  insights: readonly InsightRef[] = [],
): Screen => {
  if (screen.kind === 'category') {
    return categories !== null && !categories.includes(screen.category) ? START : screen;
  }

  if (screen.kind === 'insight') {
    return insights.some(({ identity, module }) => identity === screen.identity && module === screen.module)
      ? screen
      : START;
  }

  return screen;
};

export const NavigationContext = createContext<Navigation | null>(null);

export const useNavigation = (): Navigation => {
  const navigation = use(NavigationContext);

  if (navigation === null) {
    throw new Error('useScreen needs a NavigationContext around it');
  }

  return navigation;
};

// The current screen, among the categories of the modules the core lists and the insights of the last scan; the
// component renders again when it changes.
export const useScreen = (categories: null | readonly CategoryId[], insights: readonly InsightRef[] = []): Screen => {
  const navigation = useNavigation();

  return screenToShow(useSyncExternalStore(navigation.subscribe, navigation.getScreen), categories, insights);
};
