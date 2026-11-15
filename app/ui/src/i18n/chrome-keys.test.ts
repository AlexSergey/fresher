import { describe, expect, it } from 'vitest';

import chrome from '../locales/en.json';

// Every source file of the UI except the tests: a chrome key is used when one of them names it.
const sources = import.meta.glob<string>(['../**/*.{ts,tsx}', '!../**/*.test.{ts,tsx}'], {
  eager: true,
  import: 'default',
  query: '?raw',
});

const baseKey = (key: string): string => key.replace(/\.(?:one|other)$/, '');

const isNamed = (key: string): boolean =>
  Object.values(sources).some((source) => source.includes(`'${key}'`) || source.includes(`"${key}"`));

describe('the chrome dictionary', () => {
  describe('negative cases', () => {
    it('has no key that no file names', () => {
      const unused = [...new Set(Object.keys(chrome).map(baseKey))].filter((key) => !isNamed(key));

      expect(unused).toEqual([]);
    });
  });

  describe('positive cases', () => {
    it('reads the source files it checks', () => {
      expect(Object.keys(sources)).toContain('../app.tsx');
      expect(Object.keys(sources).some((path) => path.endsWith('.test.ts'))).toBe(false);
    });
  });
});
