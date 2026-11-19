import type { JSX } from 'react';

import { act, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { Access } from '../../ipc/bindings.generated';
import type { CoreAnswers } from '../../ipc/mock';

import { mockCore, pushEvents, refuse, sampleStorageAnswers } from '../../ipc/mock';
import { PermissionCard } from '../../product/permission-card/permission-card';
import { accessOf } from '../../state/access';
import { type CoreStore, createCoreStore } from '../../state/store';
import { allFiles, BUSY, DEMO_DICTIONARY, DEMO_MODULES, IDLE } from '../../test/demo-scan';
import { AppProviders } from '../app-providers';
import { useAccessStatus, useCoverage } from './use-access';

const core = (answers: Partial<CoreAnswers>): void => {
  mockCore({
    getDictionary: () => DEMO_DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => IDLE,
    ...sampleStorageAnswers,
    ...answers,
  });
};

const answering = (access: Access): Partial<CoreAnswers> => ({ getAccessStatus: () => allFiles(access) });

// What a screen does with the access: the card while Full Disk Access is missing.
const Probe = (): JSX.Element => {
  const access = accessOf(useAccessStatus(), { kind: 'all-files' });

  return (
    <>
      <p>{`Access: ${access ?? 'asking'}`}</p>
      {access === 'denied' || access === 'unknown' ? (
        <PermissionCard access={access} how={[]} onOpenSettings={() => undefined}>
          <p>Not seen.</p>
        </PermissionCard>
      ) : null}
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

const StatusProbe = (): JSX.Element => {
  const status = useAccessStatus();
  const coverage = useCoverage();

  return (
    <p>{`Status: ${status?.capabilities.length ?? 'asking'}, coverage: ${coverage === null ? 'none' : 'known'}`}</p>
  );
};

describe('useAccessStatus', () => {
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

    it('gives the whole status and the coverage of the result on screen', async () => {
      core(answering('granted'));

      render(
        <AppProviders store={createCoreStore()}>
          <StatusProbe />
        </AppProviders>,
      );

      expect(await screen.findByText('Status: 1, coverage: none')).toBeDefined();

      act(() => {
        pushEvents(
          { modules: ['demo'], scan: 1, type: 'scan-started' },
          { coverage: { notEntered: [], unreadable: [] }, scan: 1, type: 'scan-coverage' },
        );
      });

      expect(screen.getByText('Status: 1, coverage: known')).toBeDefined();
    });
  });
});
