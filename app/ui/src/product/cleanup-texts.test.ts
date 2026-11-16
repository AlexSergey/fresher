import { beforeEach, describe, expect, it } from 'vitest';

import { setCoreDictionary } from '../i18n';
import { CLEANUP_DICTIONARY } from '../test/cleanup-plan';
import { skipCodeText, skipReasonText } from './cleanup-texts';

beforeEach(() => {
  setCoreDictionary({
    ...CLEANUP_DICTIONARY,
    'core.cleanup.skip.not-removable': 'Fresher cannot remove it.',
    'core.removability.locked.how': 'Unlock it in Finder.',
  });
});

describe('the cleanup texts', () => {
  describe('negative cases', () => {
    it('a reason without an instruction is shown alone', () => {
      expect(skipReasonText({ kind: 'vanished' })).toBe('It is no longer there.');
      expect(skipCodeText('vanished')).toBe('It is no longer there.');
    });

    it('the history does not show an instruction that names an app it did not keep', () => {
      expect(skipCodeText('in-use')).toBe('An app is using it, so Fresher left it alone.');
    });
  });

  describe('positive cases', () => {
    it('an item in use names the app to quit', () => {
      expect(skipReasonText({ app: 'Safari', kind: 'in-use' })).toBe(
        'An app is using it, so Fresher left it alone. Quit Safari and try again.',
      );
    });

    it('an item that cannot be removed says what its removability asks', () => {
      expect(skipReasonText({ kind: 'not-removable', removability: 'locked' })).toBe(
        'Fresher cannot remove it. Unlock it in Finder.',
      );
    });
  });
});
