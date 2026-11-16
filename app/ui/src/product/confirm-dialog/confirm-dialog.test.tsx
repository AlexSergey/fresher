import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { type JSX, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { fakeReducedMotion, setMotion } from '../../test/animation';
import { confirmContentOf, LONG_CONTENT, SHORT_CONTENT } from '../../test/confirm-content';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { fakeViewport } from '../../test/viewport';
import { type ConfirmContent, ConfirmDialog, type ConfirmDialogProps } from './confirm-dialog';

const here = import.meta.dirname;
const sources = ['confirm-dialog.tsx', 'confirm-dialog.module.css'].map((file) =>
  readFileSync(path.join(here, file), 'utf8'),
);
const css = readFileSync(path.join(here, 'confirm-dialog.module.css'), 'utf8');

type Owner = Partial<Pick<ConfirmDialogProps, 'onCancel' | 'onConfirm'>> & { readonly content?: ConfirmContent };

// A screen that opens the confirmation from a button and closes it when asked, as the review screen would.
const Screen = ({ content = SHORT_CONTENT, onCancel = vi.fn(), onConfirm = vi.fn() }: Owner): JSX.Element => {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        onClick={() => {
          setOpen(true);
        }}
        type="button"
      >
        Review
      </button>
      <ConfirmDialog
        content={content}
        onCancel={() => {
          onCancel();
          setOpen(false);
        }}
        onConfirm={(method) => {
          onConfirm(method);
          setOpen(false);
        }}
        open={open}
      />
    </>
  );
};

const opener = (): HTMLElement => screen.getByRole('button', { name: 'Review' });

const openDialog = (owner: Owner = {}): HTMLElement => {
  render(<Screen {...owner} />);
  opener().focus();
  fireEvent.click(opener());

  return screen.getByRole('dialog');
};

const permanentChoice = (): HTMLElement =>
  screen.getByRole('checkbox', { name: 'Delete permanently instead — this cannot be undone' });

beforeEach(async () => {
  setCoreDictionary(FINDING_DICTIONARY);
  fakeViewport(320);
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

describe('ConfirmDialog', () => {
  describe('negative cases', () => {
    it('Cancel has the initial focus', () => {
      openDialog();

      expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus();
    });

    it('Enter on the dialog does not confirm', () => {
      const onConfirm = vi.fn();
      const dialog = openDialog({ onConfirm });

      fireEvent.keyDown(dialog, { key: 'Enter' });
      fireEvent.keyDown(screen.getByRole('button', { name: 'Cancel' }), { key: 'Enter' });
      fireEvent.keyDown(permanentChoice(), { key: 'Enter' });

      expect(onConfirm).not.toHaveBeenCalled();
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    it('the confirming button is not the danger variant until the permanent choice is ticked', () => {
      openDialog();

      expect(permanentChoice()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Move to Trash' })).toHaveAttribute('data-variant', 'primary');
      expect(screen.queryByRole('button', { name: 'Delete permanently' })).not.toBeInTheDocument();
    });

    it('the permanent choice is off when the dialog opens again', () => {
      openDialog();
      fireEvent.click(permanentChoice());
      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

      fireEvent.click(opener());

      expect(permanentChoice()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Move to Trash' })).toBeInTheDocument();
    });

    it('nothing in the dialog animates but its entrance', () => {
      for (const source of sources) {
        expect(source).not.toMatch(/AnimatedNumber|--stagger|--ease-settle|--i\b/u);
      }
      // The long list's rows do not enter, and the permanent choice's mark does not settle: both are set still.
      expect(css).toMatch(/\[data-entering\] \{\s*animation: none;/u);
      expect(css).toMatch(/path \{\s*transition: none;/u);
    });

    it('the highest level is shown with its icon and word', () => {
      const dialog = openDialog({ content: confirmContentOf('high') });

      // eslint-disable-next-line testing-library/no-node-access -- the badge's icon is decoration beside its word
      const badge = dialog.querySelector('[data-size="large"][data-level="high"]');

      expect(badge).toHaveTextContent('High');
      expect(badge).toHaveTextContent('May hold the only copy of your data. Removing it can lose work.');
      // eslint-disable-next-line testing-library/no-node-access -- the icon has no role
      expect(badge?.querySelector('svg')).not.toBeNull();
    });

    it('a short list is not virtualised', () => {
      openDialog();

      expect(screen.getAllByRole('list')).toHaveLength(2);
      expect(screen.getAllByRole('listitem')).toHaveLength(3);
      expect(screen.getAllByRole('listitem')[0]).not.toHaveAttribute('tabindex');
    });
  });

  describe('positive cases', () => {
    it('confirm calls onConfirm with trash', () => {
      const onConfirm = vi.fn();
      openDialog({ onConfirm });

      fireEvent.click(screen.getByRole('button', { name: 'Move to Trash' }));

      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('trash');
    });

    it('with the permanent choice it calls onConfirm with permanent', () => {
      const onConfirm = vi.fn();
      const dialog = openDialog({ onConfirm });

      fireEvent.click(permanentChoice());

      expect(dialog).toHaveAccessibleName('Delete 3 items permanently?');
      expect(screen.queryByRole('button', { name: 'Move to Trash' })).not.toBeInTheDocument();

      const confirm = screen.getByRole('button', { name: 'Delete permanently' });

      expect(confirm).toHaveAttribute('data-variant', 'danger');

      fireEvent.click(confirm);

      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('permanent');
    });

    it('Escape and Cancel call onCancel and return the focus', () => {
      const onCancel = vi.fn();
      const dialog = openDialog({ onCancel });

      fireEvent(dialog, new Event('cancel', { cancelable: true }));

      expect(onCancel).toHaveBeenCalledOnce();
      expect(opener()).toHaveFocus();

      fireEvent.click(opener());
      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

      expect(onCancel).toHaveBeenCalledTimes(2);
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      expect(opener()).toHaveFocus();
    });

    it("the dialog's description reads count, size and level", () => {
      const dialog = openDialog();

      expect(dialog).toHaveAccessibleName('Move 3 items to the Trash?');
      expect(dialog).toHaveAccessibleDescription('3 items, 2.6 GB in all, highest danger level Low.');
    });

    it('one item reads in the singular', () => {
      const dialog = openDialog({ content: confirmContentOf('safe') });

      expect(dialog).toHaveAccessibleName('Move 1 item to the Trash?');
      expect(dialog).toHaveAccessibleDescription('1 item, 1.5 GB, danger level Safe.');
    });

    it('every group shows what is lost', () => {
      openDialog();

      expect(screen.getByText('The apps rebuild their caches the next time they need them.')).toBeInTheDocument();
      expect(screen.getByText('You download the installers again if you need them.')).toBeInTheDocument();
      expect(screen.getByRole('list', { name: 'The items of Example caches' })).toBeInTheDocument();
    });

    it('a long list is virtualised', () => {
      openDialog({ content: LONG_CONTENT });

      const items = screen.getByRole('list', { name: 'The items of Example caches' });

      expect(screen.getAllByRole('listitem').length).toBeLessThan(40);
      expect(screen.getAllByRole('listitem')[0]).toHaveAttribute('aria-setsize', '240');
      expect(items).toBeInTheDocument();
    });

    it('the total size is static text', () => {
      const dialog = openDialog();

      // eslint-disable-next-line testing-library/no-node-access -- the size is in the display type, it has no role
      expect(dialog.querySelector('[data-size="display"]')).toHaveTextContent('2.6 GB');
    });
  });
});
