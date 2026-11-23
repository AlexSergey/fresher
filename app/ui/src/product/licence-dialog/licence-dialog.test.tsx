import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fakeReducedMotion, setMotion } from '../../test/animation';
import { LicenceDialog } from './licence-dialog';

beforeEach(async () => {
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

const root = path.join(import.meta.dirname, '../../../../..');

// A licence file's first line: its text is shown whole, so the line is in the dialog.
const firstLine = (file: string): string =>
  readFileSync(path.join(root, file), 'utf8')
    .split('\n')
    .find((line) => line.trim() !== '')
    ?.trim() ?? '';

describe('LicenceDialog', () => {
  describe('negative cases', () => {
    it('shows nothing while closed', () => {
      render(<LicenceDialog onClose={vi.fn()} open={false} />);

      expect(screen.queryByRole('dialog')).toBeNull();
    });

    it('Escape asks to close it', () => {
      const onClose = vi.fn();
      render(<LicenceDialog onClose={onClose} open />);

      fireEvent(screen.getByRole('dialog'), new Event('cancel', { cancelable: true }));

      expect(onClose).toHaveBeenCalledOnce();
    });
  });

  describe('positive cases', () => {
    it("shows both licences' texts under their names", () => {
      render(<LicenceDialog onClose={vi.fn()} open />);

      expect(screen.getByRole('dialog', { name: 'Licence' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: 'MIT licence' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: 'Apache License 2.0' })).toBeInTheDocument();
      expect(screen.getByText(firstLine('LICENSE-MIT'), { exact: false })).toBeInTheDocument();
      expect(screen.getByText(/Apache License\s+Version 2\.0/u)).toBeInTheDocument();
    });
  });
});
