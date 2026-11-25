/* eslint-disable testing-library/no-container, testing-library/no-node-access -- the badge and its icon are decorative, without a role */
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import type { CategoryIcon } from '../../ipc/bindings.generated';

import { CATEGORY_ICONS, CategoryIconBadge } from './category-icon';

// Kept equal to the core's `CategoryIcon`; the bindings test guards that list.
const KEYS: readonly CategoryIcon[] = [
  'cache',
  'build',
  'download',
  'project',
  'toolchain',
  'log',
  'temporary',
  'app',
  'model',
  'editor',
  'browser',
  'device',
  'archive',
  'generic',
];

describe('CategoryIconBadge', () => {
  describe('negative cases', () => {
    it('a badge is hidden from assistive technology', () => {
      const { container } = render(<CategoryIconBadge icon="cache" level="safe" />);

      expect(container.firstElementChild).toHaveAttribute('aria-hidden', 'true');
    });
  });

  describe('positive cases', () => {
    it('every icon key has a shape', () => {
      expect(Object.keys(CATEGORY_ICONS).toSorted()).toEqual([...KEYS].toSorted());
      for (const key of KEYS) {
        expect(CATEGORY_ICONS[key]).toBeDefined();
      }
    });

    it('takes its level and is large by default', () => {
      const { container } = render(<CategoryIconBadge icon="log" level="low" />);

      expect(container.firstElementChild).toHaveAttribute('data-level', 'low');
      expect(container.firstElementChild).toHaveAttribute('data-size', 'large');
      expect(container.querySelector('svg')).toHaveAttribute('width', '20');
    });

    it('a small badge is smaller', () => {
      const { container } = render(<CategoryIconBadge icon="log" level="low" size="small" />);

      expect(container.firstElementChild).toHaveAttribute('data-size', 'small');
      expect(container.querySelector('svg')).toHaveAttribute('width', '18');
    });
  });
});
