import { beforeEach, describe, expect, it } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { levelMeaning, levelName } from './texts';

beforeEach(() => {
  setCoreDictionary(FINDING_DICTIONARY);
});

describe('the texts of a finding', () => {
  describe('positive cases', () => {
    it("a level's word and sentence come from the core's dictionary", () => {
      expect(levelName('high')).toBe('High');
      expect(levelMeaning('high')).toBe('May hold the only copy of your data. Removing it can lose work.');
    });
  });
});
