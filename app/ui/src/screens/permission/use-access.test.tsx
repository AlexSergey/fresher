import type { JSX } from 'react';

import { act, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { Access } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse } from '../../ipc/mock';
import { PermissionCard } from '../../product/permission-card/permission-card';
import { type CoreStore, createCoreStore } from '../../state/store';
import { allFiles, BUSY, DEMO_DICTIONARY, DEMO_MODULES, IDLE } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { useAccess } from './use-access';

const core = (answers: Partial<CoreAnswers>): void => {
  mockCore({
    getDictionary: () => DEMO_DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => IDLE,
    ...answers,
  });
};

const answering = (access: Access): Partial<CoreAnswers> => ({ getAccessStatus: () => allFiles(access) });

// What the overview does with the access (07/02): the card while Full Disk Access is missing.
const Probe = (): JSX.Element => {
  const access = useAccess();

  return (
    <>
      <p>{`Access: ${access ?? 'asking'}`}</p>
      {access === 'denied' || access === 'unknown' ? <PermissionCard access={access} /> : null}
    </>
  );
};

const showProbe = (store: CoreStore = createCoreStore()): CoreStore => {
  render(
    <AppProviders store={store}>
      <Probe />
      <Probe />
    </AppProviders>,
  );

  return store;
};

describe('useAccess', () => {
  describe('negative cases', () => {
    it('an access-changed event of granted hides the permission card', async () => {
      core(answering('denied'));
      showProbe();
      expect(await screen.findAllByText('Access: denied')).toHaveLength(2);
      expect(screen.getAllByText('Fresher cannot see every folder')).toHaveLength(2);

      act(() => {
        pushEvents({ status: allFiles('granted'), type: 'access-changed' });
      });

      expect(screen.getAllByText('Access: granted')).toHaveLength(2);
      expect(screen.queryByText('Fresher cannot see every folder')).toBeNull();
    });

    it('a refused question leaves the access unknown to the screens and tells the store', async () => {
      core({ getAccessStatus: refuse(BUSY) });

      const store = showProbe();

      await vi.waitFor(() => {
        expect(store.getState().failure).toEqual({ kind: 'refused', refusal: BUSY });
      });
      expect(screen.getAllByText('Access: asking')).toHaveLength(2);
    });
  });

  describe('positive cases', () => {
    it('asks the core once, however many screens read it', async () => {
      let asked = 0;
      core({
        getAccessStatus: () => {
          asked += 1;

          return allFiles('unknown');
        },
      });

      showProbe();

      expect(await screen.findAllByText('Access: unknown')).toHaveLength(2);
      expect(asked).toBe(1);
    });
  });
});
