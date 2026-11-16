import { beforeEach, describe, expect, it } from 'vitest';

import { setCoreDictionary } from '../i18n';
import { DEMO_DICTIONARY } from '../test/demo-scan';
import { formatCount, honestSize, sizeLabel } from './sizes';

beforeEach(() => {
  setCoreDictionary({ ...DEMO_DICTIONARY, 'core.completeness.partial': 'Some files could not be read.' });
});

describe('sizes', () => {
  describe('negative cases', () => {
    it('an unknown size has no text and is never 0', () => {
      expect(honestSize({ allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 })).toStrictEqual({
        accessibleText: 'Size not known',
        completeness: 'unknown',
      });
      expect(sizeLabel({ allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 })).toBe('—');
    });

    it('a partial size says it is a lower bound, with the core reason', () => {
      expect(
        honestSize({ allocated: 42_000_000_000, completeness: 'partial', logical: 1, reclaimable: 42_000_000_000 }),
      ).toStrictEqual({
        accessibleText: 'at least 42 GB',
        completeness: 'partial',
        hint: 'Some files could not be read.',
        text: '42 GB',
      });
    });
  });

  describe('positive cases', () => {
    it('a complete size shows its allocated bytes, not its logical ones', () => {
      expect(
        honestSize({ allocated: 1_500_000_000, completeness: 'complete', logical: 9, reclaimable: 1_500_000_000 }),
      ).toStrictEqual({
        accessibleText: '1.5 GB',
        completeness: 'complete',
        text: '1.5 GB',
      });
      expect(
        sizeLabel({ allocated: 1_500_000_000, completeness: 'partial', logical: 9, reclaimable: 1_500_000_000 }),
      ).toBe('1.5 GB+');
    });

    it('formats a count with grouping and no fraction', () => {
      expect(formatCount(12_345.6)).toBe('12,346');
    });
  });
});
