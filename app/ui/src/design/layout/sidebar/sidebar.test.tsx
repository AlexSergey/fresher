import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { type JSX } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { LayoutGrid, ListChecks, Settings } from '../../icons/icons';
import { NavItem } from '../nav-item/nav-item';
import { NavSection } from './nav-section';
import { Sidebar } from './sidebar';

const css = readFileSync(path.join(import.meta.dirname, 'sidebar.module.css'), 'utf8');
const motionCss = readFileSync(path.join(import.meta.dirname, '../../tokens/motion.css'), 'utf8');

// Where each item sits in the sidebar, as the browser would lay it out.
const OFFSETS: Readonly<Record<string, number>> = { Overview: 60, Review: 92, Settings: 640 };

type Screen = 'Overview' | 'Review' | 'Settings' | null;

const Navigation = ({ active }: { readonly active: Screen }): JSX.Element => (
  <Sidebar
    footer={<NavItem active={active === 'Settings'} icon={Settings} label="Settings" onClick={vi.fn()} />}
    label="Screens"
  >
    <NavSection title="Library">
      <NavItem active={active === 'Overview'} icon={LayoutGrid} label="Overview" onClick={vi.fn()} />
      <NavItem active={active === 'Review'} icon={ListChecks} label="Review" onClick={vi.fn()} />
    </NavSection>
  </Sidebar>
);

const indicator = (): HTMLElement => {
  // eslint-disable-next-line testing-library/no-node-access -- the indicator is decoration with no role to query by
  const element = document.querySelector<HTMLElement>('[data-nav-indicator]');

  if (element === null) {
    throw new Error('The sidebar has no indicator');
  }

  return element;
};

const placeItems = (): void => {
  vi.spyOn(HTMLElement.prototype, 'offsetTop', 'get').mockImplementation(function (this: HTMLElement) {
    return OFFSETS[this.textContent] ?? 0;
  });
};

describe('Sidebar', () => {
  describe('negative cases', () => {
    it('the nav indicator does not animate under motion off', () => {
      const transition = /\.indicator \{[^}]*transition: ([^;]+);/u.exec(css)?.[1];

      expect(transition).toBe('transform var(--duration-move) var(--ease-standard)');
      expect(motionCss).toMatch(/:root\[data-motion='off'\] \{[^}]*--duration-move: 0ms;/u);
      expect(css).toMatch(/:root\[data-motion='reduced'\] \.indicator \{\s*transition: none;/u);
    });

    it('hides the indicator when no item is active', () => {
      placeItems();
      render(<Navigation active={null} />);

      expect(indicator()).not.toHaveAttribute('data-shown');
    });

    it('places the indicator without a slide the first time', () => {
      placeItems();
      render(<Navigation active="Review" />);

      expect(indicator()).toHaveAttribute('data-instant');
    });

    it('a section without a title has no heading', () => {
      render(
        <NavSection>
          <NavItem icon={LayoutGrid} label="Overview" onClick={vi.fn()} />
        </NavSection>,
      );

      expect(screen.queryByRole('heading')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the sidebar top strip is a drag region', () => {
      render(<Navigation active="Overview" />);
      // eslint-disable-next-line testing-library/no-node-access -- the strip is an empty region with no role to query by
      const strip = screen.getByRole('navigation', { name: 'Screens' }).firstElementChild;

      expect(strip).toHaveAttribute('data-tauri-drag-region');
      expect(css).toMatch(/\.strip \{[^}]*height: var\(--size-toolbar\);/u);
    });

    it('names its sections by their headings', () => {
      render(<Navigation active="Overview" />);

      expect(screen.getByRole('region', { name: 'Library' })).toContainElement(
        screen.getByRole('button', { name: 'Overview' }),
      );
    });

    it('the indicator moves to the active item', () => {
      placeItems();
      const { rerender } = render(<Navigation active="Overview" />);

      expect(indicator()).toHaveAttribute('data-shown');
      expect(indicator().style.getPropertyValue('--indicator-y')).toBe('60px');

      rerender(<Navigation active="Review" />);

      expect(indicator().style.getPropertyValue('--indicator-y')).toBe('92px');
      expect(indicator()).not.toHaveAttribute('data-instant');

      rerender(<Navigation active="Settings" />);

      expect(indicator().style.getPropertyValue('--indicator-y')).toBe('640px');
    });

    it('gives each item its place in the entrance stagger', () => {
      render(<Navigation active="Overview" />);

      expect(screen.getByRole('button', { name: 'Overview' }).style.getPropertyValue('--i')).toBe('0');
      expect(screen.getByRole('button', { name: 'Settings' }).style.getPropertyValue('--i')).toBe('2');
    });
  });
});
