import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { TreeRow } from './tree-row';

const css = readFileSync(path.join(import.meta.dirname, 'tree-row.module.css'), 'utf8');

describe('TreeRow', () => {
  describe('negative cases', () => {
    it('a collapsed row says so', () => {
      render(
        <TreeRow expanded={false} onExpandedChange={vi.fn()} title="Xcode" toggleLabel="Show the items of Xcode" />,
      );

      expect(screen.getByRole('button', { name: 'Show the items of Xcode' })).toHaveAttribute('aria-expanded', 'false');
    });

    it('a disabled row does not expand', () => {
      const onExpandedChange = vi.fn();
      render(
        <TreeRow
          disabled
          expanded={false}
          onExpandedChange={onExpandedChange}
          title="Xcode"
          toggleLabel="Show the items of Xcode"
        />,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Show the items of Xcode' }));

      expect(onExpandedChange).not.toHaveBeenCalled();
      expect(screen.getByRole('listitem')).toHaveAttribute('data-disabled');
    });
  });

  describe('positive cases', () => {
    it('expands and collapses from its disclosure', () => {
      const onExpandedChange = vi.fn();
      const { rerender } = render(
        <TreeRow expanded={false} onExpandedChange={onExpandedChange} title="Xcode" toggleLabel="Show the items" />,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Show the items' }));

      expect(onExpandedChange).toHaveBeenLastCalledWith(true);

      rerender(<TreeRow expanded onExpandedChange={onExpandedChange} title="Xcode" toggleLabel="Show the items" />);
      const disclosure = screen.getByRole('button', { name: 'Show the items' });
      fireEvent.click(disclosure);

      expect(disclosure).toHaveAttribute('aria-expanded', 'true');
      expect(onExpandedChange).toHaveBeenLastCalledWith(false);
    });

    it('indents by its level and keeps the caller leading slot after the disclosure', () => {
      render(
        <TreeRow
          expanded={false}
          leading={<span>icon</span>}
          level={2}
          onExpandedChange={vi.fn()}
          title="DerivedData"
          toggleLabel="Show the items of DerivedData"
        />,
      );

      const disclosure = screen.getByRole('button', { name: 'Show the items of DerivedData' });
      // eslint-disable-next-line testing-library/no-node-access -- the indent is layout, it has no role to query by
      const indent = disclosure.parentElement;

      expect(indent?.style.getPropertyValue('--tree-level')).toBe('2');
      expect(indent).toHaveTextContent('icon');
    });

    it('turns the chevron by transform, fast', () => {
      expect(css).toMatch(/transition: transform var\(--duration-fast\) var\(--ease-standard\)/u);
      expect(css).toMatch(/transform: rotate\(90deg\)/u);
    });
  });
});
