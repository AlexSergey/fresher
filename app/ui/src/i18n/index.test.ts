import { afterEach, describe, expect, it } from 'vitest';

import { setCoreDictionary, t, tText } from '.';

describe('t and tText', () => {
  afterEach(() => {
    setCoreDictionary({});
  });

  describe('negative cases', () => {
    it('shows the key of a core text before the dictionary is read', () => {
      expect(tText({ args: {}, key: 'core.error.busy' })).toBe('core.error.busy');
    });
  });

  describe('positive cases', () => {
    it('reads the chrome texts of the UI', () => {
      expect(t('ui.app.title')).toBe('Mac Fresher');
      expect(t('ui.app.version', { version: { kind: 'text', value: '1.2.3' } })).toBe('Version 1.2.3');
    });

    it('reads a core text from the dictionary it was given', () => {
      setCoreDictionary({ 'core.error.busy': 'The app is already running a {running}.' });

      expect(tText({ args: { running: { kind: 'text', value: 'scan' } }, key: 'core.error.busy' })).toBe(
        'The app is already running a scan.',
      );
    });
  });
});
