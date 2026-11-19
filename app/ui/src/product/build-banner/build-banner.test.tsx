import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { BuildBanner } from './build-banner';

describe('BuildBanner', () => {
  describe('negative cases', () => {
    it('a_release_build_shows_no_banner', () => {
      const { container } = render(<BuildBanner build={{ kind: 'release' }} />);

      expect(container).toBeEmptyDOMElement();
    });

    it('a_sandbox_that_simulates_nothing_does_not_say_anything_is_simulated', () => {
      render(<BuildBanner build={{ kind: 'sandbox', path: '/Users/dev/box', simulated: [] }} />);

      expect(screen.getByRole('note')).toHaveTextContent(/^Sandbox · \/Users\/dev\/box$/);
    });
  });

  describe('positive cases', () => {
    it('the_sandbox_banner_names_its_folder_and_says_its_open_apps_are_simulated', () => {
      render(<BuildBanner build={{ kind: 'sandbox', path: '/Users/dev/box', simulated: ['running'] }} />);

      expect(screen.getByRole('note')).toHaveTextContent(/^Sandbox · \/Users\/dev\/box · Simulated: open apps\.$/);
    });

    it('the_sandbox_banner_lists_every_simulated_part_in_the_cores_order', () => {
      render(
        <BuildBanner
          build={{ kind: 'sandbox', path: '/Users/dev/box', simulated: ['running', 'spotlight', 'access', 'installs'] }}
        />,
      );

      expect(screen.getByRole('note')).toHaveTextContent(
        /^Sandbox · \/Users\/dev\/box · Simulated: open apps, Spotlight, access, installs\.$/,
      );
    });

    it('the_development_banner_says_nothing_can_be_removed', () => {
      render(<BuildBanner build={{ kind: 'development' }} />);

      expect(screen.getByRole('note')).toHaveTextContent(
        'Development build · nothing can be removed outside a sandbox',
      );
    });
  });
});
