import { describe, expect, it } from 'vitest';

import { truncateMiddle } from './truncate';

const LONG = '~/Library/Developer/Xcode/DerivedData/Fresher-abcdefghijklmnop/Build/Intermediates.noindex/Index.db';

describe('truncateMiddle', () => {
  describe('negative cases', () => {
    it('a short path is not truncated', () => {
      expect(truncateMiddle('~/Library/Caches', 40)).toBe('~/Library/Caches');
      expect(truncateMiddle('~/Library/Caches', 16)).toBe('~/Library/Caches');
    });

    it('keeps a last segment longer than the room whole', () => {
      expect(truncateMiddle('~/Library/a-very-long-file-name.log', 12)).toBe('…/a-very-long-file-name.log');
    });

    it('never cuts a path of one segment in the middle', () => {
      expect(truncateMiddle('a-very-long-file-name.log', 8)).toBe('a-very-long-file-name.log');
    });
  });

  describe('positive cases', () => {
    it('a long path keeps its first folder and its last segment', () => {
      const shown = truncateMiddle(LONG, 40);

      expect(shown).toHaveLength(40);
      expect(shown.startsWith('~/Library/')).toBe(true);
      expect(shown.endsWith('/Index.db')).toBe(true);
      expect(shown).toContain('…');
    });

    it('fills the room it is given', () => {
      expect(truncateMiddle('/Users/me/Downloads/old/archive.zip', 24)).toBe('/Users/me/D…/archive.zip');
    });
  });
});
