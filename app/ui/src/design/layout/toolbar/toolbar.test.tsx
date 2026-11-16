import { act, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { Toolbar } from './toolbar';

const css = readFileSync(path.join(import.meta.dirname, 'toolbar.module.css'), 'utf8');

type Report = (entries: readonly Pick<IntersectionObserverEntry, 'isIntersecting'>[]) => void;

// The last IntersectionObserver made: what it watches, and its callback to report as the browser would.
let observed: { disconnected: boolean; report: Report; targets: Element[] };

beforeEach(() => {
  vi.stubGlobal(
    'IntersectionObserver',
    class {
      constructor(callback: Report) {
        observed = { disconnected: false, report: callback, targets: [] };
      }

      disconnect(): void {
        observed.disconnected = true;
      }

      observe(target: Element): void {
        observed.targets.push(target);
      }
    },
  );

  return (): void => {
    vi.unstubAllGlobals();
  };
});

const report = (isIntersecting: boolean): void => {
  act(() => {
    observed.report([{ isIntersecting }]);
  });
};

// eslint-disable-next-line testing-library/no-node-access -- a header inside a page has no role to query by
const toolbar = (): HTMLElement => screen.getByText('Overview').closest('header') ?? document.body;

describe('Toolbar', () => {
  describe('negative cases', () => {
    it("the toolbar's controls are not part of the drag region", () => {
      render(<Toolbar actions={<button type="button">Scan again</button>} title="Overview" />);
      const control = screen.getByRole('button', { name: 'Scan again' });

      // eslint-disable-next-line testing-library/no-node-access -- the drag region is the nearest marked ancestor
      expect(control.closest('[data-tauri-drag-region]')).toHaveAttribute('data-tauri-drag-region', 'false');
    });

    it('shows no hairline at rest', () => {
      render(<Toolbar title="Overview" />);
      report(true);

      expect(toolbar()).not.toHaveAttribute('data-scrolled');
    });

    it('stops watching when unmounted', () => {
      const { unmount } = render(<Toolbar title="Overview" />);

      unmount();

      expect(observed.disconnected).toBe(true);
    });
  });

  describe('positive cases', () => {
    it('is a drag region around its controls', () => {
      render(<Toolbar title="Overview" />);

      expect(toolbar()).toHaveAttribute('data-tauri-drag-region', 'deep');
      expect(css).toMatch(/\.toolbar \{[^}]*height: var\(--size-toolbar\);/u);
    });

    it('the toolbar shows its hairline when content scrolls under it', () => {
      render(<Toolbar title="Overview" />);

      report(false);

      expect(observed.targets).toHaveLength(1);
      // eslint-disable-next-line testing-library/no-node-access -- the sentinel sits just above the toolbar
      expect(observed.targets[0]?.nextElementSibling).toBe(toolbar());
      expect(toolbar()).toHaveAttribute('data-scrolled');
      expect(css).toMatch(/\.toolbar\[data-scrolled\]::before,\s*\.toolbar\[data-scrolled\]::after \{\s*opacity: 1;/u);

      report(true);

      expect(toolbar()).not.toHaveAttribute('data-scrolled');
    });
  });
});
