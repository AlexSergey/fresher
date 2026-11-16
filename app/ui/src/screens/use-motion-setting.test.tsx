import { act, renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { SettingsView } from '../ipc/bindings.generated';

import { mockCore, pushEvents } from '../ipc/mock';
import { connectCore } from '../state/connect';
import { CoreContext, type CoreStore, createCoreStore } from '../state/store';
import { DEMO_DICTIONARY, IDLE } from '../test/demo-scan';
import { useMotionSetting } from './use-motion-setting';

const view = (motion: string): SettingsView => ({
  file: 'loaded',
  problems: [],
  settings: [],
  values: { 'core.motion': { kind: 'choice', value: motion } },
});

const mount = (store: CoreStore): void => {
  renderHook(useMotionSetting, {
    wrapper: ({ children }) => <CoreContext value={store}>{children}</CoreContext>,
  });
};

afterEach(() => {
  delete document.documentElement.dataset.motion;
});

describe('useMotionSetting', () => {
  describe('negative cases', () => {
    it('leaves the page to the system when the core does not answer or the setting follows the system', async () => {
      mockCore({ getDictionary: () => DEMO_DICTIONARY, getOverview: () => IDLE });
      const store = createCoreStore();

      mount(store);
      await vi.waitFor(() => {
        expect(store.getState().settings).toBeNull();
      });
      expect(document.documentElement.dataset.motion).toBeUndefined();
    });
  });

  describe('positive cases', () => {
    it('reduces motion on the page, and gives it back to the system when the setting changes', async () => {
      mockCore({ getDictionary: () => DEMO_DICTIONARY, getOverview: () => IDLE, getSettings: () => view('reduced') });
      const store = createCoreStore();
      connectCore(store);

      mount(store);
      await vi.waitFor(() => {
        expect(document.documentElement.dataset.motion).toBe('reduced');
      });
      act(() => {
        pushEvents({ key: 'core.motion', type: 'setting-changed', value: { kind: 'choice', value: 'system' } });
      });

      expect(document.documentElement.dataset.motion).toBeUndefined();
    });
  });
});
