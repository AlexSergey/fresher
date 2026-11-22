import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import type { Build, Text } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { BUILD_DICTIONARY, SYSTEM_TRASH } from '../../test/sandbox-build';
import { BuildBanner } from './build-banner';

const OWN_TRASH = "Cleanups move items to the sandbox's Trash: /Users/dev/box/home/.Trash";

const OWN: Text = {
  args: { path: { kind: 'path', value: '/Users/dev/box/home/.Trash' } },
  key: 'core.build.trash.sandbox',
};

const sandbox = (simulated: Extract<Build, { readonly kind: 'sandbox' }>['simulated'], trash = OWN): Build => ({
  kind: 'sandbox',
  path: '/Users/dev/box',
  simulated,
  trash,
});

beforeEach(() => {
  setCoreDictionary(BUILD_DICTIONARY);
});

describe('BuildBanner', () => {
  describe('negative cases', () => {
    it('a_release_build_shows_no_banner', () => {
      const { container } = render(<BuildBanner build={{ kind: 'release' }} />);

      expect(container).toBeEmptyDOMElement();
    });

    it('a_development_build_shows_no_trash_line', () => {
      render(<BuildBanner build={{ kind: 'development' }} />);

      expect(screen.getByRole('note')).not.toHaveTextContent('Trash');
    });

    it('a_sandbox_that_simulates_nothing_does_not_say_anything_is_simulated', () => {
      render(<BuildBanner build={sandbox([])} />);

      expect(screen.getByRole('note').textContent).toBe(`Sandbox · /Users/dev/box · ${OWN_TRASH}`);
    });
  });

  describe('positive cases', () => {
    it('the_sandbox_banner_names_its_folder_its_trash_and_says_its_open_apps_are_simulated', () => {
      render(<BuildBanner build={sandbox(['running'])} />);

      expect(screen.getByRole('note').textContent).toBe(
        `Sandbox · /Users/dev/box · ${OWN_TRASH} · Simulated: open apps.`,
      );
    });

    it('the_sandbox_banner_lists_every_simulated_part_in_the_cores_order', () => {
      render(<BuildBanner build={sandbox(['running', 'spotlight', 'access', 'installs', 'tools'])} />);

      expect(screen.getByRole('note').textContent).toBe(
        `Sandbox · /Users/dev/box · ${OWN_TRASH} · Simulated: open apps, Spotlight, access, installs, tools.`,
      );
    });

    it('a_sandbox_on_the_real_trash_says_so_in_the_cores_words', () => {
      render(<BuildBanner build={sandbox([], SYSTEM_TRASH)} />);

      expect(screen.getByRole('note').textContent).toBe(
        "Sandbox · /Users/dev/box · Cleanups move the sandbox's items to the Trash of this Mac.",
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
