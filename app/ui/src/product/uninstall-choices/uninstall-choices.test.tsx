import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { VIDEO_STUDIO_DETAIL } from '../../test/app-detail';
import { APPS_DICTIONARY } from '../../test/apps-list';
import { UninstallChoices, type UninstallChoicesProps } from './uninstall-choices';

beforeEach(() => {
  setCoreDictionary(APPS_DICTIONARY);
});

const props = (own: Partial<UninstallChoicesProps> = {}): UninstallChoicesProps => ({
  choices: VIDEO_STUDIO_DETAIL.choices,
  onChoose: vi.fn(),
  onUninstall: vi.fn(),
  preparing: false,
  value: 'keep-data',
  ...own,
});

describe('UninstallChoices', () => {
  describe('negative cases', () => {
    it('does not choose everything while keep-data is the value', () => {
      render(<UninstallChoices {...props()} />);

      expect(screen.getByRole('radio', { name: /Remove the app and everything it left/u })).not.toBeChecked();
    });

    it('does not uninstall when a choice is made', () => {
      const own = props();
      render(<UninstallChoices {...own} />);

      fireEvent.click(screen.getByRole('radio', { name: /Remove the app and everything it left/u }));

      expect(own.onUninstall).not.toHaveBeenCalled();
    });

    it('does nothing for a key that is not a movement', () => {
      const own = props();
      render(<UninstallChoices {...own} />);

      fireEvent.keyDown(screen.getByRole('radio', { name: /keep its settings and data/u }), { key: 'a' });
      fireEvent.keyDown(screen.getByRole('radio', { name: /keep its settings and data/u }), { key: 'Home' });

      expect(own.onChoose).not.toHaveBeenCalled();
    });

    it('ignores a click on the button while a plan is being prepared', () => {
      const own = props({ preparing: true });
      render(<UninstallChoices {...own} />);

      fireEvent.click(screen.getByRole('button', { name: 'Uninstall…' }));

      expect(own.onUninstall).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('shows each choice with the core’s label, its sentence and its size', () => {
      render(<UninstallChoices {...props()} />);

      expect(screen.getByText(/Moves the app, its caches, logs, saved windows and login items/u)).toBeVisible();
      expect(screen.getAllByText('3.5 GB')).not.toHaveLength(0);
      expect(screen.getAllByText('4 GB')).not.toHaveLength(0);
    });

    it('has the value chosen', () => {
      render(<UninstallChoices {...props()} />);

      expect(screen.getByRole('radio', { name: /Remove the app, keep its settings and data/u })).toBeChecked();
    });

    it('reports the choice the user makes', () => {
      const own = props();
      render(<UninstallChoices {...own} />);

      fireEvent.click(screen.getByRole('radio', { name: /Remove the app and everything it left/u }));

      expect(own.onChoose).toHaveBeenCalledWith('everything');
    });

    it('moves the choice with the arrow keys', () => {
      const own = props();
      render(<UninstallChoices {...own} />);

      fireEvent.keyDown(screen.getByRole('radio', { name: /keep its settings and data/u }), { key: 'ArrowDown' });

      expect(own.onChoose).toHaveBeenCalledWith('everything');
    });

    it.each([
      ['wraps from the last to the first with ArrowDown', /everything it left/u, 'ArrowDown', 'keep-data'],
      ['wraps from the first to the last with ArrowUp', /keep its settings and data/u, 'ArrowUp', 'everything'],
      ['goes to the last with End', /keep its settings and data/u, 'End', 'everything'],
      ['goes to the first with Home', /everything it left/u, 'Home', 'keep-data'],
    ])('%s', (_name, from, key, expected) => {
      const own = props({ value: 'everything' });
      render(<UninstallChoices {...own} />);

      fireEvent.keyDown(screen.getByRole('radio', { name: from }), { key });

      expect(own.onChoose).toHaveBeenCalledWith(expected);
    });

    it('names each choice by its label and describes it by its sentence and size', () => {
      render(<UninstallChoices {...props()} />);

      const radio = screen.getByRole('radio', { name: 'Remove the app, keep its settings and data' });

      expect(radio).toHaveAccessibleDescription(/Moves the app, its caches.*3\.5 GB/su);
    });

    it('asks to uninstall with its button', () => {
      const own = props();
      render(<UninstallChoices {...own} />);

      fireEvent.click(screen.getByRole('button', { name: 'Uninstall…' }));

      expect(own.onUninstall).toHaveBeenCalledOnce();
    });
  });
});
