import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Settings } from '../../icons/icons';
import { Menu, type MenuItem } from './menu';

// eslint-disable-next-line testing-library/no-node-access -- the focused element has no role to query by
const focusedElement = (): Element | null => document.activeElement;

const setup = (onSelect: () => void = vi.fn()): HTMLElement => {
  const items: readonly MenuItem[] = [
    { description: 'Hides it and never removes it.', key: 'a', label: 'Ignore this category', onSelect },
    { key: 'b', label: 'Ignore Caches', onSelect },
    { key: 'c', label: 'Ignore all', onSelect },
  ];

  render(
    <>
      <Menu icon={Settings} items={items} label="Options for Caches" />
      <button type="button">Elsewhere</button>
    </>,
  );

  return screen.getByRole('button', { name: 'Options for Caches' });
};

describe('Menu', () => {
  describe('negative cases', () => {
    it('is closed until the trigger is used', () => {
      const trigger = setup();

      expect(screen.queryByRole('menu')).toBeNull();
      expect(trigger).toHaveAttribute('aria-expanded', 'false');
    });

    it('a trigger with tabIndex -1 is out of the tab order but still opens on a click', () => {
      render(
        <Menu
          icon={Settings}
          items={[{ key: 'a', label: 'Ignore', onSelect: vi.fn() }]}
          label="Options"
          tabIndex={-1}
        />,
      );
      const trigger = screen.getByRole('button', { name: 'Options' });

      expect(trigger).toHaveAttribute('tabindex', '-1');

      fireEvent.click(trigger);

      expect(screen.getByRole('menu')).toBeInTheDocument();
    });

    it('closes on Escape and gives the focus back to the trigger', () => {
      const trigger = setup();
      fireEvent.click(trigger);

      fireEvent.keyDown(screen.getAllByRole('menuitem')[0] as HTMLElement, { key: 'Escape' });

      expect(screen.queryByRole('menu')).toBeNull();
      expect(trigger).toHaveFocus();
    });

    it('closes on Tab and hands the focus to the trigger, to move on from there', () => {
      const trigger = setup();
      fireEvent.click(trigger);

      fireEvent.keyDown(screen.getAllByRole('menuitem')[0] as HTMLElement, { key: 'Tab' });

      expect(screen.queryByRole('menu')).toBeNull();
      expect(trigger).toHaveFocus();
    });

    it('closes on a pointer down outside without moving the focus', () => {
      setup();
      fireEvent.click(screen.getByRole('button', { name: 'Options for Caches' }));
      const other = screen.getByRole('button', { name: 'Elsewhere' });
      other.focus();

      fireEvent.pointerDown(other);

      expect(screen.queryByRole('menu')).toBeNull();
      expect(other).toHaveFocus();
    });

    it('does not call an item when the menu closes without a choice', () => {
      const onSelect = vi.fn();
      const trigger = setup(onSelect);
      fireEvent.click(trigger);

      fireEvent.keyDown(screen.getAllByRole('menuitem')[0] as HTMLElement, { key: 'Escape' });
      fireEvent.click(trigger);
      fireEvent.pointerDown(document.body);

      expect(onSelect).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('opens on Enter, Space and ArrowDown on the first item, on ArrowUp on the last', () => {
      const trigger = setup();

      // Enter and Space reach the button as a click.
      fireEvent.click(trigger);
      expect(screen.getAllByRole('menuitem')[0]).toHaveFocus();
      fireEvent.keyDown(screen.getAllByRole('menuitem')[0] as HTMLElement, { key: 'Escape' });

      fireEvent.keyDown(trigger, { key: 'ArrowDown' });
      expect(screen.getAllByRole('menuitem')[0]).toHaveFocus();
      fireEvent.keyDown(screen.getAllByRole('menuitem')[0] as HTMLElement, { key: 'Escape' });

      fireEvent.keyDown(trigger, { key: 'ArrowUp' });
      expect(screen.getAllByRole('menuitem')[2]).toHaveFocus();
    });

    it('moves with the arrows, wraps, and jumps with Home and End', () => {
      setup();
      fireEvent.click(screen.getByRole('button', { name: 'Options for Caches' }));
      const press = (key: string): void => {
        fireEvent.keyDown(screen.getByRole('menu'), { key });
      };
      const focused = (): number => screen.getAllByRole('menuitem').findIndex((item) => item === focusedElement());

      press('ArrowDown');
      expect(focused()).toBe(1);
      press('ArrowDown');
      press('ArrowDown');
      expect(focused()).toBe(0);
      press('ArrowUp');
      expect(focused()).toBe(2);
      press('Home');
      expect(focused()).toBe(0);
      press('End');
      expect(focused()).toBe(2);
    });

    it('selecting an item closes the menu, focuses the trigger, then calls the item', () => {
      const order: string[] = [];
      const trigger = setup(() => {
        order.push(focusedElement()?.getAttribute('aria-label') ?? 'elsewhere');
      });
      fireEvent.click(trigger);

      fireEvent.click(screen.getByRole('menuitem', { name: 'Ignore Caches' }));

      expect(screen.queryByRole('menu')).toBeNull();
      expect(trigger).toHaveFocus();
      expect(order).toEqual(['Options for Caches']);
    });

    it('selects the active item on Enter and on Space', () => {
      const onSelect = vi.fn();
      setup(onSelect);

      fireEvent.click(screen.getByRole('button', { name: 'Options for Caches' }));
      fireEvent.keyDown(screen.getByRole('menu'), { key: 'Enter' });
      fireEvent.click(screen.getByRole('button', { name: 'Options for Caches' }));
      fireEvent.keyDown(screen.getByRole('menu'), { key: ' ' });

      expect(onSelect).toHaveBeenCalledTimes(2);
    });

    it('names each item by its label and describes it by its description', () => {
      setup();
      fireEvent.click(screen.getByRole('button', { name: 'Options for Caches' }));

      expect(screen.getByRole('menuitem', { name: 'Ignore Caches' })).toBeInTheDocument();
      expect(screen.getByRole('menuitem', { name: 'Ignore this category' })).toHaveAccessibleDescription(
        'Hides it and never removes it.',
      );
      expect(screen.getByRole('menuitem', { name: 'Ignore Caches' })).not.toHaveAccessibleDescription();
    });

    it('ties the trigger to the menu with aria-controls and aria-expanded', () => {
      const trigger = setup();
      fireEvent.click(trigger);

      expect(trigger).toHaveAttribute('aria-haspopup', 'menu');
      expect(trigger).toHaveAttribute('aria-expanded', 'true');
      expect(trigger.getAttribute('aria-controls')).toBe(screen.getByRole('menu').id);
    });
  });
});
