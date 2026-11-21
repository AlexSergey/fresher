import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { STORY_DICTIONARY } from '../scan-hero/story-data';
import { DangerZone } from './danger-zone';

const SELECT_ALL = 'Select all';

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('DangerZone', () => {
  describe('negative cases', () => {
    it('shows no warning icon for Safe and Low', () => {
      const { rerender } = render(<DangerZone level="safe">content</DangerZone>);

      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(document.querySelector('svg')).toBeNull();

      rerender(<DangerZone level="low">content</DangerZone>);

      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(document.querySelector('svg')).toBeNull();
    });

    it('shows no Select all without its prop', () => {
      render(<DangerZone level="safe">content</DangerZone>);

      expect(screen.queryByRole('checkbox')).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('names the zone by the core and says what the level means', () => {
      render(<DangerZone level="safe">content</DangerZone>);

      expect(screen.getByRole('region', { name: 'Safe to remove' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { level: 2, name: 'Safe to remove' })).toBeInTheDocument();
      expect(
        screen.getByText('Comes back by itself when needed. Removing it costs only a little time.'),
      ).toBeInTheDocument();
      expect(screen.getByText('content')).toBeInTheDocument();
    });

    it('shows the warning icon for Medium and High', () => {
      const { rerender } = render(<DangerZone level="medium">content</DangerZone>);

      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(document.querySelector('svg[aria-hidden="true"]')).not.toBeNull();

      rerender(<DangerZone level="high">content</DangerZone>);

      // eslint-disable-next-line testing-library/no-node-access -- the icon is decorative, without a role
      expect(document.querySelector('svg[aria-hidden="true"]')).not.toBeNull();
    });

    it('Select all reflects mixed and calls its handler', () => {
      const onChange = vi.fn();
      render(
        <DangerZone level="low" selectAll={{ checked: 'mixed', onChange }}>
          content
        </DangerZone>,
      );
      const checkbox = screen.getByRole('checkbox', { name: SELECT_ALL });

      expect(checkbox).toBePartiallyChecked();
      expect(checkbox).toHaveAccessibleDescription(/Ticks every category in this zone.*Low risk/u);

      fireEvent.click(checkbox);

      expect(onChange).toHaveBeenCalledOnce();
    });
  });
});
