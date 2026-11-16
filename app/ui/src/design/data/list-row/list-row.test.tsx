import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { ListRow } from './list-row';

const css = readFileSync(path.join(import.meta.dirname, 'list-row.module.css'), 'utf8');

describe('ListRow', () => {
  describe('negative cases', () => {
    it('renders no empty slots', () => {
      render(<ListRow title="Xcode caches" />);

      const row = screen.getByRole('listitem');

      // eslint-disable-next-line testing-library/no-node-access -- an empty slot would be an element with no role
      expect(row.children).toHaveLength(1);
      expect(row).toHaveTextContent('Xcode caches');
    });

    it('is neither selected nor disabled by default, and is not a tab stop', () => {
      render(<ListRow title="Xcode caches" />);

      const row = screen.getByRole('listitem');

      expect(row).not.toHaveAttribute('data-selected');
      expect(row).not.toHaveAttribute('data-disabled');
      expect(row).not.toHaveAttribute('tabindex');
    });

    it('keeps the accent edge without a level', () => {
      render(<ListRow selected title="Xcode caches" />);

      expect(screen.getByRole('listitem')).not.toHaveAttribute('data-level');
    });
  });

  describe('positive cases', () => {
    it('takes the accessible name and description a caller gives it', () => {
      render(
        <>
          <ListRow aria-describedby="why" aria-label="Xcode caches, Safe, 12.4 GB" title="Xcode caches" />
          <span id="why">Rebuilt on the next build.</span>
        </>,
      );

      const row = screen.getByRole('listitem', { name: 'Xcode caches, Safe, 12.4 GB' });

      expect(row).toHaveAccessibleDescription('Rebuilt on the next build.');
    });

    it('shows its title, subtitle and both slots in order', () => {
      render(
        <ListRow
          leading={<input aria-label="Select Xcode caches" type="checkbox" />}
          subtitle="~/Library/Developer/Xcode/DerivedData"
          title="Xcode caches"
          trailing="12.4 GB"
        />,
      );

      expect(screen.getByRole('listitem')).toHaveTextContent(
        'Xcode caches~/Library/Developer/Xcode/DerivedData12.4 GB',
      );
      expect(screen.getByRole('checkbox', { name: 'Select Xcode caches' })).toBeInTheDocument();
    });

    it('marks a selected row with its level and a disabled row', () => {
      render(
        <>
          <ListRow edgeLevel="high" selected title="Mail downloads" />
          <ListRow disabled title="System caches" />
        </>,
      );

      const [selected, disabled] = screen.getAllByRole('listitem');

      expect(selected).toHaveAttribute('data-selected');
      expect(selected).toHaveAttribute('data-level', 'high');
      expect(disabled).toHaveAttribute('data-disabled');
    });

    it('carries its place in a virtual list and its tab stop', () => {
      render(<ListRow aria-posinset={3} aria-setsize={40} tabIndex={0} title="Xcode caches" />);

      const row = screen.getByRole('listitem');

      expect(row).toHaveAttribute('aria-posinset', '3');
      expect(row).toHaveAttribute('aria-setsize', '40');
      expect(row).toHaveAttribute('tabindex', '0');
    });

    it('takes the forced states of the gallery', () => {
      render(<ListRow data-focus data-hover title="Xcode caches" />);

      expect(screen.getByRole('listitem')).toHaveAttribute('data-hover');
      expect(screen.getByRole('listitem')).toHaveAttribute('data-focus');
    });

    it('lightens on hover by its background colour, fast', () => {
      expect(css).toMatch(/transition: background-color var\(--duration-fast\) var\(--ease-standard\)/u);
    });
  });
});
