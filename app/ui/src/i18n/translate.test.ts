import { describe, expect, it } from 'vitest';

import type { Dictionary, Text, TextArgs } from './types';

import { translate } from './translate';

const text = (key: string, args: TextArgs = {}): Text => ({ args, key });

const dictionary: Dictionary = {
  'demo.cleaned': 'Cleaned on {date}',
  'demo.found': 'Found {size} in {app}',
  'demo.items.one': '{count} item',
  'demo.items.other': '{count} items',
  'demo.located': 'In {path}',
  'demo.only-other.other': '{count} things',
};

describe('translate', () => {
  describe('negative cases', () => {
    it('returns the key when it is missing', () => {
      expect(translate(dictionary, text('demo.gone'))).toBe('demo.gone');
      expect(translate(dictionary, text('demo.gone', { count: { kind: 'count', value: 2 } }))).toBe('demo.gone');
    });

    it('leaves an unknown placeholder visible', () => {
      expect(translate(dictionary, text('demo.found', { app: { kind: 'text', value: 'Editor' } }))).toBe(
        'Found {size} in Editor',
      );
    });

    it('falls back to .other when .one is missing', () => {
      expect(translate(dictionary, text('demo.only-other', { count: { kind: 'count', value: 1 } }))).toBe('1 things');
    });

    it('ignores a count arg that is not a count', () => {
      expect(translate(dictionary, text('demo.items', { count: { kind: 'text', value: 'one' } }))).toBe('demo.items');
    });
  });

  describe('positive cases', () => {
    it('fills placeholders', () => {
      const found = text('demo.found', {
        app: { kind: 'text', value: 'Editor' },
        size: { kind: 'size', value: 42_000_000_000 },
      });

      expect(translate(dictionary, found)).toBe('Found 42 GB in Editor');
      expect(translate(dictionary, text('demo.located', { path: { kind: 'path', value: '/tmp/a' } }))).toBe(
        'In /tmp/a',
      );
    });

    it('picks the plural form', () => {
      expect(translate(dictionary, text('demo.items', { count: { kind: 'count', value: 1 } }))).toBe('1 item');
      expect(translate(dictionary, text('demo.items', { count: { kind: 'count', value: 12_345 } }))).toBe(
        '12,345 items',
      );
    });

    it('formats a date in the medium style', () => {
      // Noon UTC, the same calendar day in every time zone from UTC-11 to UTC+11.
      const noon = Date.UTC(2026, 0, 15, 12) / 1000;

      expect(translate(dictionary, text('demo.cleaned', { date: { kind: 'date', value: noon } }))).toBe(
        'Cleaned on Jan 15, 2026',
      );
    });
  });
});
