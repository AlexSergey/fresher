import { describe, expect, it } from 'vitest';

import { placeMenu } from './place';

const VIEWPORT = { height: 720, width: 1100 };
const MENU = { height: 100, width: 200 };

describe('placeMenu', () => {
  describe('negative cases', () => {
    it('never leaves the window on either side', () => {
      const left = placeMenu({ anchor: { bottom: 40, right: 50, top: 10 }, menu: MENU, viewport: VIEWPORT });
      const right = placeMenu({ anchor: { bottom: 40, right: 1300, top: 10 }, menu: MENU, viewport: VIEWPORT });

      expect(left.left).toBe(8);
      expect(right.left).toBe(VIEWPORT.width - 8 - MENU.width);
    });

    it('stays below when neither side fits', () => {
      const place = placeMenu({
        anchor: { bottom: 60, right: 500, top: 30 },
        menu: { height: 700, width: 200 },
        viewport: VIEWPORT,
      });

      expect(place.side).toBe('below');
    });
  });

  describe('positive cases', () => {
    it("opens below, aligned to the anchor's right edge", () => {
      const place = placeMenu({ anchor: { bottom: 40, right: 500, top: 10 }, menu: MENU, viewport: VIEWPORT });

      expect(place).toEqual({ left: 300, side: 'below', top: 44 });
    });

    it('opens above when only above fits', () => {
      const place = placeMenu({ anchor: { bottom: 700, right: 500, top: 670 }, menu: MENU, viewport: VIEWPORT });

      expect(place).toEqual({ left: 300, side: 'above', top: 566 });
    });
  });
});
