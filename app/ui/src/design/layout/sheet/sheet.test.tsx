import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { Sheet } from './sheet';

const css = readFileSync(path.join(import.meta.dirname, '../dialog/modal.module.css'), 'utf8');

const showSheet = (dismissible?: boolean): { readonly onClose: () => void; readonly sheet: HTMLElement } => {
  const onClose = vi.fn();
  render(
    <Sheet
      closeLabel="Close"
      {...(dismissible === undefined ? {} : { dismissible })}
      onClose={onClose}
      open
      title="Xcode caches"
    >
      <p>Build products and indexes.</p>
    </Sheet>,
  );

  return { onClose, sheet: screen.getByRole('dialog', { name: 'Xcode caches' }) };
};

describe('Sheet', () => {
  describe('negative cases', () => {
    it('a click inside the sheet does not close it', () => {
      const { onClose } = showSheet();

      fireEvent.click(screen.getByText('Build products and indexes.'));

      expect(onClose).not.toHaveBeenCalled();
    });

    it('a sheet told not to be dismissible ignores the backdrop', () => {
      const { onClose, sheet } = showSheet(false);

      fireEvent.click(sheet);

      expect(onClose).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('a click on the backdrop closes it by default', () => {
      const { onClose, sheet } = showSheet();

      fireEvent.click(sheet);

      expect(onClose).toHaveBeenCalledOnce();
    });

    it('is a native modal dialog attached to the bottom edge, rising as it enters', () => {
      const { sheet } = showSheet();

      expect(sheet.tagName).toBe('DIALOG');
      expect(sheet).toHaveAttribute('data-kind', 'sheet');
      expect(css).toMatch(
        /margin: auto var\(--sheet-inset\) 0 calc\(var\(--size-sidebar\) \+ var\(--sheet-inset\)\);/u,
      );
      expect(css).toMatch(
        /@keyframes sheet-in \{\s*from \{\s*opacity: 0;\s*transform: translateY\(var\(--distance-3\)\);/u,
      );
    });
  });
});
