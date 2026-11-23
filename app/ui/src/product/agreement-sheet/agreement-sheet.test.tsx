import { fireEvent, render, type RenderResult, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AgreementView } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import {
  AGREEMENT_CHANGED,
  AGREEMENT_CHANGES_DICTIONARY,
  AGREEMENT_DICTIONARY,
  AGREEMENT_NOT_ACCEPTED,
} from '../../test/agreement';
import { fakeReducedMotion, setMotion } from '../../test/animation';
import { AgreementSheet } from './agreement-sheet';

beforeEach(async () => {
  setCoreDictionary({ ...AGREEMENT_DICTIONARY, ...AGREEMENT_CHANGES_DICTIONARY });
  fakeReducedMotion(false);
  await setMotion('off');
});

afterEach(async () => {
  vi.unstubAllGlobals();
  await setMotion(null);
});

type Handlers = Partial<Record<'onAccept' | 'onDecline' | 'onLicence', () => void>>;

const open = (view: AgreementView = AGREEMENT_NOT_ACCEPTED, handlers: Handlers = {}): RenderResult =>
  render(
    <AgreementSheet
      onAccept={handlers.onAccept ?? vi.fn()}
      onDecline={handlers.onDecline ?? vi.fn()}
      onLicence={handlers.onLicence ?? vi.fn()}
      open
      view={view}
    />,
  );

describe('AgreementSheet', () => {
  describe('negative cases', () => {
    it('shows no changes notice without changes', () => {
      open();

      expect(screen.queryByText('What changed since you last accepted it:')).toBeNull();
    });

    it('Escape declines and accepts nothing', () => {
      const onAccept = vi.fn();
      const onDecline = vi.fn();
      open(AGREEMENT_NOT_ACCEPTED, { onAccept, onDecline });

      fireEvent(screen.getByRole('dialog'), new Event('cancel', { cancelable: true }));

      expect(onDecline).toHaveBeenCalledOnce();
      expect(onAccept).not.toHaveBeenCalled();
    });

    it('Not now declines and accepts nothing', () => {
      const onAccept = vi.fn();
      const onDecline = vi.fn();
      open(AGREEMENT_NOT_ACCEPTED, { onAccept, onDecline });

      fireEvent.click(screen.getByRole('button', { name: 'Not now' }));

      expect(onDecline).toHaveBeenCalledOnce();
      expect(onAccept).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('shows the title, the intro and the five lines in order', () => {
      open();
      const dialog = screen.getByRole('dialog', { name: 'Before your first cleanup' });

      expect(within(dialog).getByText('Five things to know before Fresher removes anything:')).toBeInTheDocument();
      expect(
        within(dialog)
          .getAllByRole('listitem')
          .map((item) => item.textContent),
      ).toEqual([
        'Fresher is free and open source, and it comes with no warranty.',
        'It removes only what you confirm, so each removal is your decision.',
        "A danger level is Fresher's best judgement, not a guarantee.",
        'Keep a backup of anything you cannot afford to lose.',
        "Items go to the Trash, where you can put them back until the Trash is emptied. A permanent delete, and what an app's own tool removes, cannot be undone.",
      ]);
    });

    it('shows what changed when the core says', () => {
      open(AGREEMENT_CHANGED);

      expect(screen.getByText('What changed since you last accepted it:')).toBeInTheDocument();
      expect(screen.getByText('A line about backups was added.')).toBeInTheDocument();
    });

    it('Not now has the first focus', () => {
      open();

      expect(screen.getByRole('button', { name: 'Not now' })).toHaveFocus();
    });

    it('I understand accepts', () => {
      const onAccept = vi.fn();
      open(AGREEMENT_NOT_ACCEPTED, { onAccept });

      fireEvent.click(screen.getByRole('button', { name: 'I understand' }));

      expect(onAccept).toHaveBeenCalledOnce();
    });

    it('the licence button asks for the full licence', () => {
      const onLicence = vi.fn();
      open(AGREEMENT_NOT_ACCEPTED, { onLicence });

      fireEvent.click(screen.getByRole('button', { name: 'Read the full licence' }));

      expect(onLicence).toHaveBeenCalledOnce();
    });
  });
});
