import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { type JSX, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CleanupPlanView } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { fakeReducedMotion, setMotion } from '../../test/animation';
import {
  ALL_LEFT_OUT_PLAN,
  CLEANUP_DICTIONARY,
  DEMO_PLAN,
  HIGH_PLAN,
  LEFT_OUT_PLAN,
  LONG_PLAN,
  ONE_ITEM_PLAN,
  TOOL_ONLY_PLAN,
  TOOL_PLAN,
} from '../../test/cleanup-plan';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { fakeViewport } from '../../test/viewport';
import { ConfirmDialog, type ConfirmDialogProps } from './confirm-dialog';

const here = import.meta.dirname;
const sources = ['confirm-dialog.tsx', 'confirm-dialog.module.css'].map((file) =>
  readFileSync(path.join(here, file), 'utf8'),
);
const css = readFileSync(path.join(here, 'confirm-dialog.module.css'), 'utf8');

type Owner = Partial<Pick<ConfirmDialogProps, 'onCancel' | 'onConfirm'>> & { readonly plan?: CleanupPlanView };

// A screen that opens the confirmation from a button and closes it when asked, as the review screen would.
const Screen = ({ onCancel = vi.fn(), onConfirm = vi.fn(), plan = DEMO_PLAN }: Owner): JSX.Element => {
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
        onCancel={() => {
          onCancel();
          setOpen(false);
        }}
        onConfirm={(method) => {
          onConfirm(method);
          setOpen(false);
        }}
        open={open}
        plan={plan}
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
  setCoreDictionary({ ...FINDING_DICTIONARY, ...CLEANUP_DICTIONARY });
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
      const dialog = openDialog({ plan: HIGH_PLAN });

      // eslint-disable-next-line testing-library/no-node-access -- the badge's icon is decoration beside its word
      const badge = dialog.querySelector('[data-size="large"][data-level="high"]');

      expect(badge).toHaveTextContent('High');
      expect(badge).toHaveTextContent('May hold the only copy of your data. Removing it can lose work.');
      // eslint-disable-next-line testing-library/no-node-access -- the icon has no role
      expect(badge?.querySelector('svg')).not.toBeNull();
    });

    it('the dialog shows no sum of its own', () => {
      // Groups that do not add up to the summary: the figures shown are the plan's, never a sum.
      const uneven: CleanupPlanView = { ...DEMO_PLAN, groups: DEMO_PLAN.groups.slice(0, 1) };
      const dialog = openDialog({ plan: uneven });

      expect(dialog).toHaveAccessibleName('Move 3 items to the Trash?');
      // eslint-disable-next-line testing-library/no-node-access -- the size is in the display type, it has no role
      expect(dialog.querySelector('[data-size="display"]')).toHaveTextContent('2.6 GB');
    });

    it('left out items are listed and not counted', () => {
      const dialog = openDialog({ plan: LEFT_OUT_PLAN });
      const leftOut = screen.getByRole('region', { name: 'These will not be removed' });

      expect(dialog).toHaveAccessibleName('Move 2 items to the Trash?');
      expect(leftOut).toHaveTextContent('Example: browser-cache');
      expect(leftOut).toHaveTextContent('An app is using it, so Fresher left it alone. Quit Safari and try again.');
      for (const items of screen.getAllByRole('list', { name: /^The items of / })) {
        expect(items).not.toHaveTextContent('browser-cache');
      }
      expect(screen.getByRole('list', { name: 'The items of Example caches' })).toHaveTextContent('editor-cache');
    });

    it('a plan whose items were all left out offers nothing to confirm', () => {
      const onConfirm = vi.fn();
      const dialog = openDialog({ onConfirm, plan: ALL_LEFT_OUT_PLAN });

      expect(dialog).toHaveAccessibleName('Nothing here can be removed');
      expect(screen.getAllByRole('button').map((button) => button.textContent)).not.toContainEqual(
        expect.stringMatching(/Move|Delete|Remove/u),
      );
      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.getByRole('region', { name: 'These will not be removed' })).toHaveTextContent(
        'Quit Xcode and try again.',
      );
      expect(screen.getByRole('button', { name: 'Back to the review' })).toHaveFocus();
    });

    it('a plan of tool items only offers no permanent choice and no Trash', () => {
      const dialog = openDialog({ plan: TOOL_ONLY_PLAN });

      expect(dialog).toHaveAccessibleName('Remove 1 item?');
      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByText('You get the space back when you empty the Trash.')).toBeNull();
      expect(screen.getByRole('button', { name: 'Remove' })).toHaveAttribute('data-variant', 'danger');
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

    it('a tool group says it cannot be put back', () => {
      openDialog({ plan: TOOL_PLAN });

      const images = screen.getByRole('region', { name: 'Example images' });

      expect(images).toHaveTextContent('Removed by its own tool, such as Docker. It cannot be put back.');
      expect(screen.getByRole('region', { name: 'Example caches' })).toHaveTextContent('Goes to the Trash.');
    });

    it('the trash sentence is shown for the trash only', () => {
      openDialog();
      const sentence = 'You get the space back when you empty the Trash.';

      expect(screen.getByText(sentence)).toBeInTheDocument();

      fireEvent.click(permanentChoice());

      expect(screen.queryByText(sentence)).toBeNull();
      expect(screen.getByRole('region', { name: 'Example caches' })).toHaveTextContent('Deleted permanently.');
      expect(permanentChoice()).toHaveAccessibleDescription('Frees the space now. You cannot put these items back.');
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
      const dialog = openDialog({ plan: ONE_ITEM_PLAN });

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
      openDialog({ plan: LONG_PLAN });

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
