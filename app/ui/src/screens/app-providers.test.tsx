import { act, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { Availability, ModuleInfo } from '../ipc/bindings.generated';

import { mockCore, pushEvents, sampleStorageAnswers } from '../ipc/mock';
import { createCoreStore } from '../state/store';
import { allFiles, DEMO_DICTIONARY, DEMO_MODULES, IDLE } from '../test/demo-scan';
import { AppProviders } from './app-providers';
import { accessOfCategories } from './permission/category-access';
import { useModules } from './use-modules';

const [DEMO] = DEMO_MODULES as readonly [ModuleInfo];

const withAvailability = (availability: Availability): readonly ModuleInfo[] => [
  { ...DEMO, categories: DEMO.categories.map((category) => ({ ...category, availability })) },
];

// The first category's state as the cards would read it.
const Probe = (): React.JSX.Element => {
  const kind = accessOfCategories(useModules(), null).get('demo.caches')?.kind ?? 'ready';

  return <p>caches: {kind}</p>;
};

describe('AppProviders', () => {
  describe('negative cases', () => {
    it('keeps the last list when a refresh fails', async () => {
      let calls = 0;
      mockCore({
        getDictionary: () => DEMO_DICTIONARY,
        getModules: () => {
          calls += 1;
          if (calls === 2) {
            throw new Error('busy');
          }
          if (calls > 2) {
            // Never answers: the list on screen stays the one the failed refresh left.
            return new Promise<never>(() => undefined);
          }

          return withAvailability({ kind: 'unavailable', missing: [{ kind: 'all-files' }] });
        },
        getOverview: () => IDLE,
        ...sampleStorageAnswers,
      });
      render(
        <AppProviders store={createCoreStore()}>
          <Probe />
        </AppProviders>,
      );
      await screen.findByText('caches: unavailable');

      act(() => {
        pushEvents({ status: allFiles('granted'), type: 'access-changed' });
      });
      act(() => {
        pushEvents({ status: allFiles('denied'), type: 'access-changed' });
      });
      // The third request starts only once the failed one has settled, so its failure has been handled.
      await waitFor(() => {
        expect(calls).toBe(3);
      });

      expect(screen.getByText('caches: unavailable')).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('asks for the modules again when the access changes, and an unavailable category becomes available', async () => {
      let granted = false;
      let calls = 0;
      mockCore({
        getDictionary: () => DEMO_DICTIONARY,
        getModules: () => {
          calls += 1;

          return withAvailability(
            granted ? { kind: 'ready' } : { kind: 'unavailable', missing: [{ kind: 'all-files' }] },
          );
        },
        getOverview: () => IDLE,
        ...sampleStorageAnswers,
      });
      render(
        <AppProviders store={createCoreStore()}>
          <Probe />
        </AppProviders>,
      );
      await screen.findByText('caches: unavailable');

      granted = true;
      act(() => {
        pushEvents({ status: allFiles('granted'), type: 'access-changed' });
      });

      expect(await screen.findByText('caches: ready')).toBeInTheDocument();
      expect(calls).toBe(2);
    });
  });
});
