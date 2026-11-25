import { describe, expect, it } from 'vitest';

import { shortPath } from './short-path';

describe('shortPath', () => {
  describe('negative cases', () => {
    it('leaves the path as it is without a home', () => {
      expect(shortPath('/Users/a/Library/Caches', null)).toBe('/Users/a/Library/Caches');
      expect(shortPath('/Users/a/Library/Caches', '')).toBe('/Users/a/Library/Caches');
    });

    it('does not shorten a folder whose name only starts with the home', () => {
      expect(shortPath('/Users/ab/x', '/Users/a')).toBe('/Users/ab/x');
    });

    it('leaves a path outside the home as it is', () => {
      expect(shortPath('/Library/Caches/x', '/Users/a')).toBe('/Library/Caches/x');
      expect(shortPath('/Users', '/Users/a')).toBe('/Users');
    });

    it('never treats the root as a home', () => {
      expect(shortPath('/Library/x', '/')).toBe('/Library/x');
    });
  });

  describe('positive cases', () => {
    it('shows a path below the home from ~', () => {
      expect(shortPath('/Users/a/Library/Caches/com.example', '/Users/a')).toBe('~/Library/Caches/com.example');
    });

    it('shows the home itself as ~', () => {
      expect(shortPath('/Users/a', '/Users/a')).toBe('~');
    });

    it('takes a home written with a trailing slash', () => {
      expect(shortPath('/Users/a/x', '/Users/a/')).toBe('~/x');
    });
  });
});
