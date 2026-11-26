import { fireEvent, render, screen, within } from '@testing-library/react';
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
  LONG_TOGETHER_PLAN,
  ONE_ITEM_PLAN,
  planOf,
  TOGETHER_FINDING,
  TOGETHER_PLAN,
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
        onConfirm={(method, highRiskUnderstood) => {
          onConfirm(method, highRiskUnderstood);
          setOpen(false);
        }}
        open={open}
        plan={plan}
      />
    </>
  );
};

const MORE_FILES = 'and 2 more files that go with it: Example: demo-model';

const opener = (): HTMLElement => screen.getByRole('button', { name: 'Review' });

const openDialog = (owner: Owner = {}): HTMLElement => {
  render(<Screen {...owner} />);
  opener().focus();
  fireEvent.click(opener());

  return screen.getByRole('dialog');
};

const highTick = (): HTMLElement =>
  screen.getByRole('checkbox', { name: 'I understand these may be the only copy of my data' });

const queryHighTick = (): HTMLElement | null =>
  screen.queryByRole('checkbox', { name: 'I understand these may be the only copy of my data' });

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
    it('shows no more-files line when an item has one path', () => {
      openDialog();

      expect(screen.queryByRole('button', { name: /more files? that/u })).not.toBeInTheDocument();
    });

    it('keeps the other paths of a together item hidden until asked for', () => {
      openDialog({ plan: TOGETHER_PLAN });

      expect(screen.getByRole('button', { name: MORE_FILES })).toHaveAttribute('aria-expanded', 'false');
      expect(screen.queryByRole('list', { name: /The other files/u })).not.toBeInTheDocument();
    });

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

    it('a plan below High shows no High tick', () => {
      openDialog({ plan: DEMO_PLAN });

      expect(queryHighTick()).toBeNull();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toBeEnabled();
    });

    it('a High plan cannot be confirmed without the tick', () => {
      const onConfirm = vi.fn();
      openDialog({ onConfirm, plan: HIGH_PLAN });
      const confirm = screen.getByRole('button', { name: 'Yes, move to the Trash' });

      expect(highTick()).not.toBeChecked();
      expect(confirm).toBeDisabled();
      expect(confirm).toHaveAccessibleDescription(/Fresher cannot tell whether you keep a copy elsewhere/u);

      fireEvent.click(confirm);

      expect(onConfirm).not.toHaveBeenCalled();
    });

    it('the High tick is cleared on reopening', () => {
      openDialog({ plan: HIGH_PLAN });
      fireEvent.click(highTick());
      expect(highTick()).toBeChecked();
      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

      fireEvent.click(opener());

      expect(highTick()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toBeDisabled();
    });

    it('the High tick and the permanent tick are separate', () => {
      openDialog({ plan: HIGH_PLAN });

      fireEvent.click(permanentChoice());

      expect(highTick()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Delete permanently' })).toBeDisabled();

      fireEvent.click(permanentChoice());
      fireEvent.click(highTick());

      expect(permanentChoice()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toBeEnabled();
    });

    it('the confirming button is not the danger variant until the permanent choice is ticked', () => {
      openDialog();

      expect(permanentChoice()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toHaveAttribute('data-variant', 'primary');
      expect(screen.queryByRole('button', { name: 'Delete permanently' })).not.toBeInTheDocument();
    });

    it('the permanent choice is off when the dialog opens again', () => {
      openDialog();
      fireEvent.click(permanentChoice());
      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

      fireEvent.click(opener());

      expect(permanentChoice()).not.toBeChecked();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toBeInTheDocument();
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
      expect(badge).toHaveTextContent('May hold the only copy of your data');
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
      expect(screen.getByRole('button', { name: 'Back to Review' })).toHaveFocus();
    });

    it('a plan of tool items only offers no permanent choice and no Trash', () => {
      const dialog = openDialog({ plan: TOOL_ONLY_PLAN });

      expect(dialog).toHaveAccessibleName('Remove 1 item?');
      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByText('You get the space back when you empty the Trash.')).toBeNull();
      expect(screen.getByRole('button', { name: 'Remove' })).toHaveAttribute('data-variant', 'danger');
    });

    it('a plan that only goes to the Trash offers no permanent choice', () => {
      const onConfirm = vi.fn();
      openDialog({ onConfirm, plan: { ...ONE_ITEM_PLAN, permanentAvailable: false } });

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByRole('button', { name: 'Remove' })).toBeNull();
      expect(screen.getByText('You get the space back when you empty the Trash.')).toBeVisible();
      const trash = screen.getByRole('button', { name: 'Yes, move to the Trash' });
      expect(trash).not.toHaveAttribute('data-variant', 'danger');
      fireEvent.click(trash);
      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('trash', false);
    });

    it('a short list is not virtualised', () => {
      openDialog();

      expect(screen.getAllByRole('list')).toHaveLength(2);
      expect(screen.getAllByRole('listitem')).toHaveLength(3);
      expect(screen.getAllByRole('listitem')[0]).not.toHaveAttribute('tabindex');
    });
  });

  describe('positive cases', () => {
    it('names how many more files go with an item', () => {
      openDialog({ plan: TOGETHER_PLAN });

      expect(screen.getByRole('button', { name: MORE_FILES })).toBeInTheDocument();
    });

    it('lists every other path on demand', () => {
      openDialog({ plan: TOGETHER_PLAN });

      fireEvent.click(screen.getByRole('button', { name: MORE_FILES }));

      const list = screen.getByRole('list', { name: 'The other files that go with Example: demo-model' });
      const [, ...others] = TOGETHER_FINDING.removal.kind === 'move-together' ? TOGETHER_FINDING.removal.value : [];

      expect(screen.getByRole('button', { name: MORE_FILES })).toHaveAttribute('aria-expanded', 'true');
      expect(within(list).getAllByRole('listitem')).toHaveLength(2);
      for (const [at, other] of others.entries()) {
        expect(within(list).getAllByRole('listitem')[at]).toHaveTextContent(other);
      }

      fireEvent.click(screen.getByRole('button', { name: MORE_FILES }));

      expect(screen.queryByRole('list', { name: /The other files/u })).not.toBeInTheDocument();
    });

    it('the toggle in a virtual row is left to the button: its keys are not taken by the row', () => {
      openDialog({ plan: LONG_TOGETHER_PLAN });
      const toggle = screen.getByRole('button', { name: MORE_FILES });
      toggle.focus();

      // The list does not preventDefault a key of a control inside a row, so Enter and Space activate the button.
      expect(fireEvent.keyDown(toggle, { key: 'Enter' })).toBe(true);
      expect(fireEvent.keyDown(toggle, { key: ' ' })).toBe(true);
      expect(toggle).toHaveAttribute('aria-expanded', 'false');

      fireEvent.click(toggle);

      expect(toggle).toHaveAttribute('aria-expanded', 'true');
    });

    it('the buttons of two together items name their own item', () => {
      const other = {
        ...TOGETHER_FINDING,
        id: 31,
        title: { args: { name: { kind: 'text', value: 'second' } }, key: 'demo.finding.title' },
      } as const;
      openDialog({ plan: planOf([TOGETHER_FINDING, other]) });

      expect(screen.getByRole('button', { name: MORE_FILES })).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: 'and 2 more files that go with it: Example: second' }),
      ).toBeInTheDocument();
    });

    it('in a long group the other paths open below the list, one item at a time', () => {
      openDialog({ plan: LONG_TOGETHER_PLAN });

      fireEvent.click(screen.getByRole('button', { name: MORE_FILES }));

      const list = screen.getByRole('list', { name: 'The other files that go with Example: demo-model' });

      expect(within(list).getAllByRole('listitem')).toHaveLength(2);
      expect(screen.getByRole('list', { name: 'The items of Example caches' })).not.toContainElement(list);
    });

    it('a ticked High plan confirms with the risk understood', () => {
      const onConfirm = vi.fn();
      openDialog({ onConfirm, plan: HIGH_PLAN });

      fireEvent.click(highTick());
      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));

      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('trash', true);
    });

    it('a High plan without the permanent choice still shows the tick', () => {
      const onConfirm = vi.fn();
      openDialog({ onConfirm, plan: { ...HIGH_PLAN, permanentAvailable: false } });

      expect(screen.queryByRole('checkbox', { name: /^Delete permanently instead/u })).toBeNull();
      expect(screen.getByRole('button', { name: 'Yes, move to the Trash' })).toBeDisabled();

      fireEvent.click(highTick());
      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));

      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('trash', true);
    });

    it('confirm calls onConfirm with trash', () => {
      const onConfirm = vi.fn();
      openDialog({ onConfirm });

      fireEvent.click(screen.getByRole('button', { name: 'Yes, move to the Trash' }));

      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('trash', false);
    });

    it('with the permanent choice it calls onConfirm with permanent', () => {
      const onConfirm = vi.fn();
      const dialog = openDialog({ onConfirm });

      fireEvent.click(permanentChoice());

      expect(dialog).toHaveAccessibleName('Delete 3 items permanently?');
      expect(screen.queryByRole('button', { name: 'Yes, move to the Trash' })).not.toBeInTheDocument();

      const confirm = screen.getByRole('button', { name: 'Delete permanently' });

      expect(confirm).toHaveAttribute('data-variant', 'danger');

      fireEvent.click(confirm);

      expect(onConfirm).toHaveBeenCalledExactlyOnceWith('permanent', false);
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

    it('a long plan scrolls only its list: the method and the buttons are outside it', () => {
      const dialog = openDialog({ plan: LONG_PLAN });
      const items = screen.getByRole('list', { name: 'The items of Example caches' });

      expect(items.contains(permanentChoice())).toBe(false);
      expect(items.contains(screen.getByRole('button', { name: 'Yes, move to the Trash' }))).toBe(false);
      expect(items.contains(screen.getByRole('heading', { name: 'Move 240 items to the Trash?' }))).toBe(false);
      expect(dialog).toContainElement(permanentChoice());
      // The list takes the height the dialog leaves, so the dialog's own body has nothing to scroll.
      expect(css).toMatch(/\.long \{[^}]*min-height: 96px;[^}]*flex: 1 1 320px;/u);
      expect(css).toMatch(/\.content \{[^}]*height: 100%;[^}]*flex-direction: column;/u);
    });

    it('the High tick and the method sit in the footer with the buttons, out of the items that scroll', () => {
      openDialog({ plan: HIGH_PLAN });
      const confirm = screen.getByRole('button', { name: 'Yes, move to the Trash' });
      // eslint-disable-next-line testing-library/no-node-access -- the footer is the dialog's fixed region, it has no role
      const footer = confirm.closest('footer');
      const items = screen.getByRole('list', { name: 'The items of Example caches' });

      expect(footer).not.toBeNull();
      expect(footer).toContainElement(highTick());
      expect(footer).toContainElement(permanentChoice());
      expect(footer?.contains(items)).toBe(false);
      expect(highTick().compareDocumentPosition(permanentChoice()) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
      expect(permanentChoice().compareDocumentPosition(confirm) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    });

    it('the total size is static text', () => {
      const dialog = openDialog();

      // eslint-disable-next-line testing-library/no-node-access -- the size is in the display type, it has no role
      expect(dialog.querySelector('[data-size="display"]')).toHaveTextContent('2.6 GB');
    });
  });
});
