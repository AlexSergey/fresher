import { afterEach, describe, expect, it } from 'vitest';

import { inApp } from '.';

describe('inApp', () => {
  afterEach(() => {
    Reflect.deleteProperty(globalThis, 'isTauri');
  });

  describe('negative cases', () => {
    it('is false outside the app', () => {
      expect(inApp()).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('is true when the page runs in the app', () => {
      Reflect.set(globalThis, 'isTauri', true);

      expect(inApp()).toBe(true);
    });
  });
});
