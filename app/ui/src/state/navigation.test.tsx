import { act, fireEvent, render, screen } from '@testing-library/react';
import { type JSX } from 'react';
import { describe, expect, it, vi } from 'vitest';

import type { CategoryId } from '../ipc/bindings.generated';

import {
  createNavigation,
  type Navigation,
  NavigationContext,
  screenToShow,
  useNavigation,
  useScreen,
} from './navigation';

// The current screen, and a button that opens the review, as a screen of the app would.
const Current = ({ categories }: { readonly categories: null | readonly CategoryId[] }): JSX.Element => {
  const shown = useScreen(categories);
  const { navigate } = useNavigation();

  return (
    <>
      <p>{shown.kind === 'category' ? `category ${shown.category}` : shown.kind}</p>
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

const renderScreen = (navigation: Navigation, categories: null | readonly CategoryId[] = ['demo.caches']): void => {
  render(
    <NavigationContext value={navigation}>
      <Current categories={categories} />
    </NavigationContext>,
  );
};

describe('navigation', () => {
  describe('negative cases', () => {
    it('refuses to be read outside its context', () => {
      vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => render(<Current categories={null} />)).toThrow('useScreen needs a NavigationContext');
    });

    it('a category screen of a category no listed module has falls back to the scan screen', () => {
      const navigation = createNavigation();
      navigation.navigate({ category: 'gone.category', kind: 'category' });

      renderScreen(navigation);

      expect(screen.getByText('scan')).toBeInTheDocument();
    });

    it('keeps a category screen while the modules are not known yet', () => {
      expect(screenToShow({ category: 'demo.caches', kind: 'category' }, null)).toEqual({
        category: 'demo.caches',
        kind: 'category',
      });
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
    it('starts on the scan screen', () => {
      expect(createNavigation().getScreen()).toEqual({ kind: 'scan' });
    });

    it('keeps a category screen of a category a listed module has', () => {
      expect(screenToShow({ category: 'demo.caches', kind: 'category' }, ['demo.caches'])).toEqual({
        category: 'demo.caches',
        kind: 'category',
      });
    });

    it('navigate changes the screen', () => {
      const navigation = createNavigation();
      renderScreen(navigation);

      act(() => {
        navigation.navigate({ category: 'demo.caches', kind: 'category' });
      });
      expect(screen.getByText('category demo.caches')).toBeInTheDocument();

      act(() => {
        navigation.navigate({ kind: 'settings' });
      });
      expect(screen.getByText('settings')).toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: 'Review' }));
      expect(screen.getByText('review')).toBeInTheDocument();
    });
  });
});
