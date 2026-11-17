import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { App } from './app';
import { mockCore, sampleStorageAnswers } from './ipc/mock';
import { createCoreStore } from './state/store';
import { DEMO_DICTIONARY, DEMO_MODULES, GRANTED, IDLE } from './test/demo-scan';

describe('App', () => {
  describe('negative cases', () => {
    it('shows no screen before the core has sent its texts', () => {
      mockCore({
        getDictionary: () => new Promise(() => undefined),
        getModules: () => DEMO_MODULES,
        getOverview: () => IDLE,
      });

      render(<App store={createCoreStore()} />);

      expect(screen.queryByRole('heading', { level: 1 })).toBeNull();
      expect(screen.getByRole('navigation', { name: 'Screens' })).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('opens on the overview in the frame', async () => {
      mockCore({
        getAccessStatus: () => GRANTED,
        getDictionary: () => DEMO_DICTIONARY,
        getModules: () => DEMO_MODULES,
        getOverview: () => IDLE,
        ...sampleStorageAnswers,
      });

      render(<App store={createCoreStore()} />);

      expect(await screen.findByRole('heading', { level: 1, name: 'Overview' })).toBeInTheDocument();
      expect(await screen.findByRole('button', { name: /^Demo module/ })).toBeInTheDocument();
    });
  });
});
