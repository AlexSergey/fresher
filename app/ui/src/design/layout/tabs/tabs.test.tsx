import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Tabs } from './tabs';

type Kind = 'groups' | 'items' | 'modules';

const TABS = [
  { label: 'Ignored groups', value: 'groups' },
  { count: 2, label: 'Not looked for', value: 'modules' },
  { count: 0, label: 'Ignored items', value: 'items' },
] as const;

const show = (value: Kind, onChange = vi.fn()): ReturnType<typeof vi.fn> => {
  render(
    <Tabs label="Ignored" onChange={onChange} tabs={TABS} value={value}>
      <p>The panel</p>
    </Tabs>,
  );

  return onChange;
};

describe('Tabs', () => {
  describe('negative cases', () => {
    it('makes only the selected tab a tab stop', () => {
      show('modules');

      expect(screen.getByRole('tab', { name: 'Ignored groups' })).toHaveAttribute('tabindex', '-1');
      expect(screen.getByRole('tab', { name: 'Not looked for · 2' })).toHaveAttribute('tabindex', '0');
      expect(screen.getByRole('tab', { name: 'Ignored items' })).toHaveAttribute('tabindex', '-1');
    });

    it('shows no count when it is zero or absent', () => {
      show('groups');

      expect(screen.getByRole('tab', { name: 'Ignored groups' })).toBeInTheDocument();
      expect(screen.getByRole('tab', { name: 'Ignored items' })).toBeInTheDocument();
    });

    it('ignores the keys it does not know', () => {
      const onChange = show('groups');

      fireEvent.keyDown(screen.getByRole('tab', { name: 'Ignored groups' }), { key: 'a' });
      fireEvent.keyDown(screen.getByRole('tab', { name: 'Ignored groups' }), { key: 'Home' });

      expect(onChange).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('has a tab list of tabs and one panel named by the selected tab', () => {
      show('modules');

      expect(screen.getByRole('tablist', { name: 'Ignored' })).toBeInTheDocument();
      expect(screen.getAllByRole('tab')).toHaveLength(3);
      expect(screen.getByRole('tab', { name: 'Not looked for · 2' })).toHaveAttribute('aria-selected', 'true');
      expect(screen.getByRole('tabpanel', { name: 'Not looked for · 2' })).toHaveTextContent('The panel');
    });

    it('moves with Left and Right, wrapping, and to the ends with End and Home', () => {
      const onChange = show('groups');
      const first = screen.getByRole('tab', { name: 'Ignored groups' });

      fireEvent.keyDown(first, { key: 'ArrowRight' });
      fireEvent.keyDown(first, { key: 'ArrowLeft' });
      fireEvent.keyDown(first, { key: 'End' });

      expect(onChange.mock.calls).toEqual([['modules'], ['items'], ['items']]);
    });

    it('moves focus to the tab a key selects', () => {
      show('groups');

      fireEvent.keyDown(screen.getByRole('tab', { name: 'Ignored groups' }), { key: 'ArrowRight' });

      expect(screen.getByRole('tab', { name: 'Not looked for · 2' })).toHaveFocus();
    });

    it('selects the tab that is clicked', () => {
      const onChange = show('groups');

      fireEvent.click(screen.getByRole('tab', { name: 'Ignored items' }));

      expect(onChange).toHaveBeenCalledWith('items');
    });
  });
});
