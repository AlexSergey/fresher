import { act, fireEvent, render, screen } from '@testing-library/react';
import { type JSX } from 'react';
import { describe, expect, it, vi } from 'vitest';

import type { ModuleId } from '../ipc/bindings.generated';

import {
  createNavigation,
  type Navigation,
  NavigationContext,
  placeTop,
  screenToShow,
  useNavigation,
  useScreen,
} from './navigation';

// The current screen, and a button that opens the review, as a screen of the app would.
const Current = ({ modules }: { readonly modules: null | readonly ModuleId[] }): JSX.Element => {
  const shown = useScreen(modules);
  const { navigate } = useNavigation();

  return (
    <>
      <p>{shown.kind === 'module' ? `module ${shown.module}` : shown.kind}</p>
      <button
        onClick={() => {
          navigate({ kind: 'review' });
        }}
        type="button"
      >
        Review
      </button>
    </>
  );
};

const renderScreen = (navigation: Navigation, modules: null | readonly ModuleId[] = ['demo']): void => {
  render(
    <NavigationContext value={navigation}>
      <Current modules={modules} />
    </NavigationContext>,
  );
};

describe('navigation', () => {
  describe('negative cases', () => {
    it('refuses to be read outside its context', () => {
      vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => render(<Current modules={null} />)).toThrow('useScreen needs a NavigationContext');
    });

    it('a module screen of an unknown module falls back to the overview', () => {
      const navigation = createNavigation();
      navigation.navigate({ kind: 'module', module: 'gone' });

      renderScreen(navigation);

      expect(screen.getByText('overview')).toBeInTheDocument();
    });

    it('keeps a module screen while the modules are not known yet', () => {
      expect(screenToShow({ kind: 'module', module: 'demo' }, null)).toEqual({ kind: 'module', module: 'demo' });
    });

    it('stops telling a listener that unsubscribed', () => {
      const navigation = createNavigation();
      const listener = vi.fn();
      const unsubscribe = navigation.subscribe(listener);

      unsubscribe();
      navigation.navigate({ kind: 'review' });

      expect(listener).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('starts on the overview', () => {
      expect(createNavigation().getScreen()).toEqual({ kind: 'overview' });
    });

    it('starts by meaning and keeps the chosen view of the overview across screens', () => {
      const navigation = createNavigation();
      expect(navigation.getOverviewView()).toEqual({ kind: 'meaning' });

      navigation.showOverviewView(placeTop(0));
      navigation.navigate({ kind: 'settings' });
      navigation.navigate({ kind: 'overview' });

      expect(navigation.getOverviewView()).toEqual(placeTop(0));
    });

    it('keeps the folder the place view has open across screens', () => {
      const navigation = createNavigation();
      const opened = { ...placeTop(0), descent: 'stay', node: { index: 4, state: 1 } } as const;

      navigation.showOverviewView(opened);
      navigation.navigate({ kind: 'settings' });
      navigation.navigate({ kind: 'overview' });

      expect(navigation.getOverviewView()).toStrictEqual(opened);
    });

    it('navigate changes the screen', () => {
      const navigation = createNavigation();
      renderScreen(navigation);

      act(() => {
        navigation.navigate({ kind: 'module', module: 'demo' });
      });
      expect(screen.getByText('module demo')).toBeInTheDocument();

      act(() => {
        navigation.navigate({ kind: 'settings' });
      });
      expect(screen.getByText('settings')).toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: 'Review' }));
      expect(screen.getByText('review')).toBeInTheDocument();
    });
  });
});
