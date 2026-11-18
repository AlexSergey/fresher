import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { BuildBanner } from './build-banner';

describe('BuildBanner', () => {
  describe('negative cases', () => {
    it('a_release_build_shows_no_banner', () => {
      const { container } = render(<BuildBanner build={{ kind: 'release' }} />);

      expect(container).toBeEmptyDOMElement();
    });

    it('a_sandbox_whose_open_apps_are_real_does_not_say_they_are_simulated', () => {
      render(
        <BuildBanner
          build={{ kind: 'sandbox', path: '/Users/dev/box', simulatedRunning: false, simulatedSpotlight: false }}
        />,
      );

      expect(screen.getByRole('note')).toHaveTextContent(/^Sandbox · \/Users\/dev\/box$/);
    });
  });

  describe('positive cases', () => {
    it('the_sandbox_banner_names_its_folder_and_says_its_open_apps_are_simulated', () => {
      render(
        <BuildBanner
          build={{ kind: 'sandbox', path: '/Users/dev/box', simulatedRunning: true, simulatedSpotlight: false }}
        />,
      );

      expect(screen.getByRole('note')).toHaveTextContent('Sandbox · /Users/dev/box · Open apps are simulated.');
    });

    it('the_sandbox_banner_says_its_open_apps_and_spotlight_are_simulated', () => {
      render(
        <BuildBanner
          build={{ kind: 'sandbox', path: '/Users/dev/box', simulatedRunning: true, simulatedSpotlight: true }}
        />,
      );

      expect(screen.getByRole('note')).toHaveTextContent(
        'Sandbox · /Users/dev/box · Open apps and Spotlight are simulated.',
      );
    });

    it('the_sandbox_banner_says_spotlight_alone_is_simulated', () => {
      render(
        <BuildBanner
          build={{ kind: 'sandbox', path: '/Users/dev/box', simulatedRunning: false, simulatedSpotlight: true }}
        />,
      );

      expect(screen.getByRole('note')).toHaveTextContent(/^Sandbox · \/Users\/dev\/box · Spotlight is simulated\.$/);
    });

    it('the_development_banner_says_nothing_can_be_removed', () => {
      render(<BuildBanner build={{ kind: 'development' }} />);

      expect(screen.getByRole('note')).toHaveTextContent(
        'Development build · nothing can be removed outside a sandbox',
      );
    });
  });
});
