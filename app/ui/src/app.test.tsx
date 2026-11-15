import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { App } from './app';
import { mockCore } from './ipc/mock';

describe('App', () => {
  describe('negative cases', () => {
    it('keeps waiting for the core when the version cannot be read', async () => {
      mockCore({});

      render(<App />);

      expect(await screen.findByText('Connecting to the core…')).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the version the core returns', async () => {
      mockCore({ appVersion: () => '1.2.3' });

      render(<App />);

      expect(await screen.findByText('Version 1.2.3')).toBeInTheDocument();
    });
  });
});
