import { act, renderHook } from '@testing-library/react';
import { type JSX, type ReactNode } from 'react';
import { describe, expect, it } from 'vitest';

import { mockCore, refuse } from '../ipc/mock';
import { CoreContext, type CoreStore, createCoreStore } from '../state/store';
import { BUSY } from '../test/demo-scan';
import { type ScanActions, useScanActions } from './use-scan-actions';

const renderActions = (store: CoreStore): ScanActions => {
  const wrapper = ({ children }: { readonly children: ReactNode }): JSX.Element => (
    <CoreContext value={store}>{children}</CoreContext>
  );

  return renderHook(useScanActions, { wrapper }).result.current;
};

// Lets the rejected command reach the store.
const settle = async (): Promise<void> => {
  await act(async () => {
    await Promise.resolve();
  });
};

describe('useScanActions', () => {
  describe('negative cases', () => {
    it('keeps a refused scan for the frame', async () => {
      mockCore({ startScan: refuse(BUSY) });
      const store = createCoreStore();

      renderActions(store).scan();
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: BUSY });
    });

    it('keeps a refused stop for the frame', async () => {
      mockCore({ cancelScan: refuse(BUSY) });
      const store = createCoreStore();

      renderActions(store).stop();
      await settle();

      expect(store.getState().failure).toStrictEqual({ kind: 'refused', refusal: BUSY });
    });
  });

  describe('positive cases', () => {
    it('asks the core for a scan and to stop it', async () => {
      const asked: string[] = [];
      mockCore({
        cancelScan: () => {
          asked.push('cancel');

          return null;
        },
        startScan: () => {
          asked.push('start');

          return 1;
        },
      });
      const store = createCoreStore();
      const { scan, stop } = renderActions(store);

      scan();
      stop();
      await settle();

      expect(asked).toStrictEqual(['start', 'cancel']);
      expect(store.getState().failure).toBeNull();
    });
  });
});
