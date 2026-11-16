import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { type JSX, useRef, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fakeReducedMotion, recordAnimations, setMotion } from '../../../test/animation';
import { Dialog, type DialogProps } from './dialog';

const css = readFileSync(path.join(import.meta.dirname, 'modal.module.css'), 'utf8');

type Owner = Partial<Pick<DialogProps, 'dismissible' | 'onClose'>> & { readonly focusCancel?: boolean };

// A screen that opens the dialog from a button and closes it when asked, as a caller would.
const Screen = ({ dismissible, focusCancel = false, onClose = vi.fn() }: Owner): JSX.Element => {
  const [open, setOpen] = useState(false);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const onDelete = vi.fn();

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
      <Dialog
        closeLabel="Close"
        description="The items go to the Trash; you can put them back."
        {...(dismissible === undefined ? {} : { dismissible })}
        footer={
          <>
            <button ref={cancelRef} type="button">
              Cancel
            </button>
            <button onClick={onDelete} type="button">
              Move to Trash
            </button>
          </>
        }
        {...(focusCancel ? { initialFocus: cancelRef } : {})}
        onClose={() => {
          onClose();
          setOpen(false);
        }}
        open={open}
        title="Move 3 items to the Trash?"
      >
        <p>Xcode caches</p>
        <a href="#details">Details</a>
      </Dialog>
    </>
  );
};

const openDialog = (owner: Owner = {}): HTMLDialogElement => {
  render(<Screen {...owner} />);
  const opener = screen.getByRole('button', { name: 'Review' });

  opener.focus();
  fireEvent.click(opener);

  return screen.getByRole<HTMLDialogElement>('dialog', { name: 'Move 3 items to the Trash?' });
};

// A dialog open with a long text and no control in its body, its body showing this much of what it holds.
const openLongText = ({ shown, tall }: { readonly shown: number; readonly tall: number }): void => {
  vi.spyOn(HTMLElement.prototype, 'scrollHeight', 'get').mockReturnValue(tall);
  vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(shown);
  render(
    <Dialog closeLabel="Close" onClose={vi.fn()} open title="What is removed">
      <p>Every item, one by one.</p>
    </Dialog>,
  );
};

// Escape, as the browser reports it to an open modal dialog.
const pressEscape = (dialog: HTMLDialogElement): void => {
  fireEvent(dialog, new Event('cancel', { cancelable: true }));
};

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  await setMotion(null);
});

describe('Dialog', () => {
  describe('negative cases', () => {
    it('a body that fits is not a tab stop', () => {
      openLongText({ shown: 300, tall: 300 });

      expect(screen.queryByRole('region')).not.toBeInTheDocument();
    });

    it('a long body with a control of its own leaves the focus to the control', () => {
      vi.spyOn(HTMLElement.prototype, 'scrollHeight', 'get').mockReturnValue(1200);
      vi.spyOn(HTMLElement.prototype, 'clientHeight', 'get').mockReturnValue(300);
      openDialog();

      expect(screen.queryByRole('region')).not.toBeInTheDocument();
    });

    it('Escape closes and calls onClose once', async () => {
      const animations = recordAnimations();
      const onClose = vi.fn();
      const dialog = openDialog({ onClose });

      pressEscape(dialog);
      pressEscape(dialog);
      await animations[0]?.finish();

      expect(onClose).toHaveBeenCalledOnce();
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('keeps the native close on Escape from running', () => {
      const dialog = openDialog();
      const escape = new Event('cancel', { cancelable: true });

      fireEvent(dialog, escape);

      expect(escape.defaultPrevented).toBe(true);
    });

    it('a click inside the panel does not close', () => {
      const onClose = vi.fn();
      openDialog({ dismissible: true, onClose });

      fireEvent.click(screen.getByText('Xcode caches'));

      expect(onClose).not.toHaveBeenCalled();
    });

    it('a click on the backdrop does not close a dialog by default', () => {
      const onClose = vi.fn();
      const dialog = openDialog({ onClose });

      fireEvent.click(dialog);

      expect(onClose).not.toHaveBeenCalled();
    });

    it("Enter does not trigger the footer's buttons unless one has the focus", () => {
      const dialog = openDialog();
      const deleted = vi.fn();
      screen.getByRole('button', { name: 'Move to Trash' }).addEventListener('click', deleted);

      fireEvent.keyDown(dialog, { key: 'Enter' });
      fireEvent.keyDown(screen.getByText('Xcode caches'), { key: 'Enter' });

      expect(deleted).not.toHaveBeenCalled();
    });

    it('the page behind is inert while open', () => {
      const showModal = vi.spyOn(HTMLDialogElement.prototype, 'showModal');
      const show = vi.spyOn(HTMLDialogElement.prototype, 'show');

      openDialog();

      expect(showModal).toHaveBeenCalled();
      expect(show).not.toHaveBeenCalled();
    });

    it('motion off closes without waiting for an animation', async () => {
      await setMotion('off');
      const animations = recordAnimations();
      const close = vi.spyOn(HTMLDialogElement.prototype, 'close');
      openDialog();

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(animations).toHaveLength(0);
      expect(close).toHaveBeenCalled();
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });

    it('never settles or bounces', () => {
      expect(css).not.toMatch(/--ease-settle/u);
      expect(css).toMatch(/animation: modal-in var\(--duration-base\) var\(--ease-standard\);/u);
    });
  });

  describe('positive cases', () => {
    it('a long body without a control takes the focus, named by the title, so the keyboard can scroll it', () => {
      openLongText({ shown: 300, tall: 1200 });

      expect(screen.getByRole('region', { name: 'What is removed' })).toHaveAttribute('tabindex', '0');
    });

    it('the title names the dialog and the description describes it', () => {
      const dialog = openDialog();

      expect(dialog).toHaveAccessibleName('Move 3 items to the Trash?');
      expect(dialog).toHaveAccessibleDescription('The items go to the Trash; you can put them back.');
    });

    it('the initial focus goes to the given element', () => {
      openDialog({ focusCancel: true });

      expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus();
    });

    it('the focus goes to the first focusable element otherwise', () => {
      openDialog();

      expect(screen.getByRole('button', { name: 'Close' })).toHaveFocus();
    });

    it('the focus returns to the opener on close', async () => {
      const animations = recordAnimations();
      openDialog();

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));
      await animations[0]?.finish();

      expect(screen.getByRole('button', { name: 'Review' })).toHaveFocus();
    });

    it('a click on the backdrop closes when dismissible', () => {
      const onClose = vi.fn();
      const dialog = openDialog({ dismissible: true, onClose });

      fireEvent.click(dialog);

      expect(onClose).toHaveBeenCalledOnce();
    });

    it('the exit animation runs before close', async () => {
      const animations = recordAnimations();
      const close = vi.spyOn(HTMLDialogElement.prototype, 'close');
      const dialog = openDialog();

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(dialog).toHaveAttribute('data-closing');
      expect(animations[0]?.keyframes).toStrictEqual([{ opacity: 1 }, { opacity: 0 }]);
      expect(animations[0]?.options.duration).toBe(120);
      expect(close).not.toHaveBeenCalled();

      await animations[0]?.finish();

      expect(close).toHaveBeenCalled();
    });

    it('blurs the page behind, with the prefixed pair', () => {
      expect(css).toMatch(
        /\.modal::backdrop \{\s*-webkit-backdrop-filter: blur\(16px\) saturate\(140%\);\s*backdrop-filter: blur\(16px\) saturate\(140%\);/u,
      );
    });
  });
});
