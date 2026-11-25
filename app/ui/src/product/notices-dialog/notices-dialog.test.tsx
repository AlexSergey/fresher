import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { fakeReducedMotion, setMotion } from '../../test/animation';
import { type Notices, NoticesDialog } from './notices-dialog';

beforeEach(async () => {
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

const NOTICES: Notices = {
  licences: [
    { id: 'MIT', name: 'MIT License', text: 'Permission is hereby granted', usedBy: ['react 1.0.0', 'serde 1.0.0'] },
    { id: 'MPL-2.0', name: 'Mozilla Public License 2.0', text: 'Mozilla Public License', usedBy: ['selectors 0.1.0'] },
  ],
  sources: [
    { licence: 'MPL-2.0', name: 'selectors', url: 'https://crates.io/crates/selectors/0.1.0', version: '0.1.0' },
  ],
};

describe('NoticesDialog', () => {
  describe('negative cases', () => {
    it('shows nothing while closed', () => {
      render(<NoticesDialog notices={NOTICES} onClose={vi.fn()} open={false} />);

      expect(screen.queryByRole('dialog')).toBeNull();
    });

    it('says the licences are being read until they arrive', () => {
      render(<NoticesDialog notices={null} onClose={vi.fn()} open />);

      expect(screen.getByText('Reading the licences.')).toBeInTheDocument();
      expect(screen.queryByText('MIT License', { exact: false })).toBeNull();
    });

    it('has no source section when no part needs one', () => {
      render(<NoticesDialog notices={{ ...NOTICES, sources: [] }} onClose={vi.fn()} open />);

      expect(screen.queryByRole('heading', { name: 'Source code of the MPL-2.0 parts' })).toBeNull();
    });

    it('Escape asks to close it', () => {
      const onClose = vi.fn();
      render(<NoticesDialog notices={NOTICES} onClose={onClose} open />);

      fireEvent(screen.getByRole('dialog'), new Event('cancel', { cancelable: true }));

      expect(onClose).toHaveBeenCalledOnce();
    });
  });

  describe('positive cases', () => {
    it('names every licence with how many parts use it', () => {
      render(<NoticesDialog notices={NOTICES} onClose={vi.fn()} open />);

      const dialog = screen.getByRole('dialog', { name: 'Open-source licences' });
      expect(within(dialog).getByText('Used by 2 parts')).toBeInTheDocument();
      expect(within(dialog).getByText('Used by 1 part')).toBeInTheDocument();
    });

    it("shows a licence's parts and its full text", () => {
      render(<NoticesDialog notices={NOTICES} onClose={vi.fn()} open />);

      expect(screen.getByText('react 1.0.0, serde 1.0.0')).toBeInTheDocument();
      expect(screen.getByText('Permission is hereby granted')).toBeInTheDocument();
    });

    it('gives the source address of an MPL-2.0 part as plain text, not a link', () => {
      render(<NoticesDialog notices={NOTICES} onClose={vi.fn()} open />);

      expect(screen.getByRole('heading', { name: 'Source code of the MPL-2.0 parts' })).toBeInTheDocument();
      expect(screen.getByText('https://crates.io/crates/selectors/0.1.0')).toBeInTheDocument();
      expect(screen.queryByRole('link')).toBeNull();
    });
  });
});
