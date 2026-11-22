import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { createNavigation, type Navigation } from '../../state/navigation';
import { createCoreStore } from '../../state/store';
import { DEMO_INSIGHT, DEMO_INSIGHT_DICTIONARY } from '../../test/demo-insight';
import { AppProviders } from '../app-providers';
import { InsightScreen } from './insight-screen';

const renderScreen = (): Navigation => {
  const navigation = createNavigation();

  render(
    <AppProviders navigation={navigation} store={createCoreStore()}>
      <InsightScreen insight={DEMO_INSIGHT} module="demo" />
    </AppProviders>,
  );

  return navigation;
};

const partOf = (name: string): HTMLElement => screen.getByRole('listitem', { name });

describe('InsightScreen', () => {
  beforeEach(() => {
    setCoreDictionary(DEMO_INSIGHT_DICTIONARY);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('negative cases', () => {
    it('a part without a command has no Copy button', () => {
      renderScreen();

      expect(within(partOf('Unused volumes')).queryByRole('button')).toBeNull();
      expect(screen.getAllByRole('button', { name: 'Copy' })).toHaveLength(2);
    });

    it('has no checkbox or selection bar', () => {
      renderScreen();

      expect(screen.queryByRole('checkbox')).toBeNull();
      expect(screen.queryByText(/selected/iu)).toBeNull();
    });

    it('says it could not copy when the clipboard refuses', async () => {
      vi.stubGlobal('navigator', { clipboard: { writeText: vi.fn().mockRejectedValue(new Error('refused')) } });
      renderScreen();

      fireEvent.click(within(partOf('Build cache')).getByRole('button', { name: 'Copy' }));

      expect(
        await within(partOf('Build cache')).findByText('Could not copy. Select the command and copy it.'),
      ).toBeVisible();
    });
  });

  describe('positive cases', () => {
    it('shows each part with its level, size, detail and effect', () => {
      renderScreen();

      const images = partOf('Unused images');
      expect(within(images).getByRole('heading', { level: 2, name: 'Unused images' })).toBeVisible();
      expect(images).toHaveTextContent('Low');
      expect(images).toHaveTextContent('3.2 GB');
      expect(images).toHaveTextContent('Made-up images no container uses.');
      expect(images).toHaveTextContent('They are downloaded again when needed.');
      expect(partOf('Unused volumes')).toHaveTextContent('High');
    });

    it('shows the command line as the core gave it', () => {
      renderScreen();

      expect(screen.getByText('demo-tool cache prune --all').tagName).toBe('CODE');
      expect(screen.getByText('demo-tool image prune --all')).toBeVisible();
    });

    it('Copy writes the line and says Copied', async () => {
      const writeText = vi.fn().mockResolvedValue(undefined);
      vi.stubGlobal('navigator', { clipboard: { writeText } });
      renderScreen();

      fireEvent.click(within(partOf('Unused images')).getByRole('button', { name: 'Copy' }));

      expect(await within(partOf('Unused images')).findByText('Copied.')).toBeVisible();
      expect(writeText).toHaveBeenCalledWith('demo-tool image prune --all');
    });

    it('Back returns to the scan screen, to the insights card', () => {
      const { getScreen, navigate } = renderScreen();
      navigate({ identity: 'demo-tool', kind: 'insight', module: 'demo' });

      fireEvent.click(screen.getByRole('button', { name: 'Back' }));

      expect(getScreen()).toStrictEqual({ fromInsight: 'demo/demo-tool', kind: 'scan' });
    });

    it('shows the notes', () => {
      renderScreen();

      expect(screen.getByText('Freeing space inside does not shrink the file on the disk at once.')).toBeVisible();
      expect(screen.getByText('Fresher never runs these commands. You run them yourself.')).toBeVisible();
    });
  });
});
