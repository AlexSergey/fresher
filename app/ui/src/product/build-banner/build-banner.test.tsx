import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { BuildBanner } from './build-banner';

describe('BuildBanner', () => {
  describe('negative cases', () => {
    it('a_release_build_shows_no_banner', () => {
      const { container } = render(<BuildBanner build={{ kind: 'release' }} />);

      expect(container).toBeEmptyDOMElement();
    });
  });

  describe('positive cases', () => {
    it('the_sandbox_banner_names_its_folder', () => {
      render(<BuildBanner build={{ kind: 'sandbox', path: '/Users/dev/box' }} />);

      expect(screen.getByRole('note')).toHaveTextContent('Sandbox · /Users/dev/box');
    });

    it('the_development_banner_says_nothing_can_be_removed', () => {
      render(<BuildBanner build={{ kind: 'development' }} />);

      expect(screen.getByRole('note')).toHaveTextContent(
        'Development build · nothing can be removed outside a sandbox',
      );
    });
  });
});
