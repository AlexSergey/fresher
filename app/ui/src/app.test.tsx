import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { CoreAnswers } from './ipc/mock';

import { App } from './app';
import { mockCore, pushEvents, refuse } from './ipc/mock';
import { createCoreStore } from './state/store';
import { BUSY, DEMO_DICTIONARY, DEMO_FINDINGS, DEMO_SCAN, FIRST_BATCH, IDLE, SCAN_STARTED } from './test/demo-scan';

const core = (answers: Partial<CoreAnswers> = {}): void => {
  mockCore({ appVersion: () => '1.2.3', getDictionary: () => DEMO_DICTIONARY, getOverview: () => IDLE, ...answers });
};

// The app, once it shows the version and the dictionary's texts.
const renderApp = async (): Promise<void> => {
  render(<App store={createCoreStore()} />);
  await screen.findByText('Version 1.2.3');
};

describe('App', () => {
  describe('negative cases', () => {
    it('keeps waiting for the core when the version cannot be read', async () => {
      mockCore({});

      render(<App store={createCoreStore()} />);

      expect(await screen.findByText('Connecting to the core…')).toBeInTheDocument();
    });

    it('shows the refusal text when a scan is busy', async () => {
      core({ startScan: refuse(BUSY) });
      await renderApp();

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(await screen.findByRole('alert')).toHaveTextContent(
        'The app is already running a scan. Wait for it to finish, or cancel it.',
      );
    });

    it('shows its own text when the core cannot be reached', async () => {
      core();
      await renderApp();

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(await screen.findByRole('alert')).toHaveTextContent('The app could not reach its core. Restart the app.');
    });

    it('shows a cancelled scan as partial, with what it found', async () => {
      core();
      await renderApp();

      act(() => {
        pushEvents(SCAN_STARTED, FIRST_BATCH, { outcome: { kind: 'partial' }, scan: 1, type: 'scan-finished' });
      });

      expect(screen.getByText(/The scan stopped early/)).toBeInTheDocument();
      expect(screen.getByText(/Example: editor-cache/)).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the version the core returns', async () => {
      core();

      await renderApp();

      expect(screen.getByText('Version 1.2.3')).toBeInTheDocument();
      expect(screen.getByText('Press Scan to see what takes the space.')).toBeInTheDocument();
    });

    it('shows the scan while it runs', async () => {
      core({ startScan: () => 1 });
      await renderApp();

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));
      act(() => {
        pushEvents(SCAN_STARTED);
      });

      expect(screen.getByText('Scanning…')).toBeInTheDocument();
    });

    it('shows every label from the dictionary', async () => {
      core();
      await renderApp();

      act(() => {
        pushEvents(...DEMO_SCAN);
      });

      expect(screen.getByRole('heading', { name: 'Example caches' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { name: 'Example downloads' })).toBeInTheDocument();
      expect(screen.getByText('The scan is complete.')).toBeInTheDocument();
      const caches = screen.getByRole('region', { name: 'Example caches' });
      expect(caches).toHaveTextContent(
        'Example: editor-cache · 1.5 GB · Safe · Example data from the demo module: it does not exist on your disk.',
      );
      expect(screen.getByRole('region', { name: 'Example downloads' })).toHaveTextContent(
        'Example: installer.dmg · 750 MB · Low',
      );
      expect(screen.getAllByRole('listitem')).toHaveLength(DEMO_FINDINGS.length);
      expect(screen.queryByText(/demo\.|core\./)).toBeNull();
    });

    it('shows the reason of a failed scan', async () => {
      core();
      await renderApp();

      act(() => {
        pushEvents(SCAN_STARTED, {
          outcome: { kind: 'failed', reason: { args: {}, key: 'demo.finding.reason' } },
          scan: 1,
          type: 'scan-finished',
        });
      });

      expect(screen.getByText(/does not exist on your disk/)).toBeInTheDocument();
    });
  });
});
