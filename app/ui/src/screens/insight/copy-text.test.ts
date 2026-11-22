import { afterEach, describe, expect, it, vi } from 'vitest';

import { copyText } from './copy-text';

describe('copyText', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('negative cases', () => {
    it('answers false without the clipboard API', async () => {
      vi.stubGlobal('navigator', {});

      expect(await copyText('a line')).toBe(false);
    });

    it('answers false when writing is refused', async () => {
      vi.stubGlobal('navigator', { clipboard: { writeText: vi.fn().mockRejectedValue(new Error('refused')) } });

      expect(await copyText('a line')).toBe(false);
    });
  });

  describe('positive cases', () => {
    it('answers true when the text is written', async () => {
      const writeText = vi.fn().mockResolvedValue(undefined);
      vi.stubGlobal('navigator', { clipboard: { writeText } });

      expect(await copyText('a line')).toBe(true);
      expect(writeText).toHaveBeenCalledWith('a line');
    });
  });
});
