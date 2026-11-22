import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { VIDEO_STUDIO_DETAIL } from '../../test/app-detail';
import { APPS_DICTIONARY } from '../../test/apps-list';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { FootprintList } from './footprint-list';

beforeEach(() => {
  setCoreDictionary({ ...FINDING_DICTIONARY, ...APPS_DICTIONARY });
});

const sectionOf = (name: string): HTMLElement => screen.getByRole('region', { name });

describe('FootprintList', () => {
  describe('negative cases', () => {
    it('shows no path until it is asked for', () => {
      render(<FootprintList items={VIDEO_STUDIO_DETAIL.items} />);

      expect(screen.queryAllByText(/Applications\/Video Studio\.app/u)).toHaveLength(0);
      expect(within(sectionOf('The app')).getByRole('button', { name: 'Show the path' })).toHaveAttribute(
        'aria-expanded',
        'false',
      );
    });

    it('does not say a kept item goes with either choice', () => {
      render(<FootprintList items={VIDEO_STUDIO_DETAIL.items} />);

      const kept = sectionOf("Shared with its maker's other apps");

      expect(within(kept).queryByText('Goes with both choices')).not.toBeInTheDocument();
      expect(within(kept).queryByText('Only with everything')).not.toBeInTheDocument();
      expect(within(kept).getByText(/Other apps by the same maker may use this folder/u)).toBeVisible();
    });

    it('shows nothing for an app without items', () => {
      render(<FootprintList items={[]} />);

      expect(screen.queryByRole('region')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('groups the items by kind under the core’s names', () => {
      render(<FootprintList items={VIDEO_STUDIO_DETAIL.items} />);

      for (const name of ['The app', 'Caches', 'Logs', 'Saved windows', 'Starts at login', 'Settings']) {
        expect(sectionOf(name)).toBeInTheDocument();
      }
    });

    it('shows each item’s danger and size', () => {
      render(<FootprintList items={VIDEO_STUDIO_DETAIL.items} />);

      const bundle = sectionOf('The app');

      expect(within(bundle).getByText('Medium')).toBeVisible();
      expect(within(bundle).getAllByText('3 GB')).not.toHaveLength(0);
      expect(within(sectionOf('Files it keeps for you')).getByText('High')).toBeVisible();
    });

    it('says which choices take an item', () => {
      render(<FootprintList items={VIDEO_STUDIO_DETAIL.items} />);

      expect(within(sectionOf('Caches')).getByText('Goes with both choices')).toBeVisible();
      expect(within(sectionOf('Settings')).getByText('Only with everything')).toBeVisible();
    });

    it('shows the path of an item on demand', () => {
      render(<FootprintList items={VIDEO_STUDIO_DETAIL.items} />);

      fireEvent.click(within(sectionOf('The app')).getByRole('button', { name: 'Show the path' }));

      expect(screen.getAllByText('/Applications/Video Studio.app')).not.toHaveLength(0);
    });
  });
});
