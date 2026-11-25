import { renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { useNotices } from './use-notices';

describe('useNotices', () => {
  describe('negative cases', () => {
    it('reads nothing until the notices are wanted', () => {
      const { result } = renderHook(() => useNotices(false));

      expect(result.current).toBeNull();
    });

    it('drops the notices read after it is gone', () => {
      const { result, unmount } = renderHook(() => useNotices(true));
      unmount();

      expect(result.current).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('reads the generated notices once wanted, the MPL-2.0 parts with their source', async () => {
      const { result } = renderHook(() => useNotices(true));

      await waitFor(() => {
        expect(result.current).not.toBeNull();
      });
      expect(result.current?.licences.length).toBeGreaterThan(0);
      expect(result.current?.sources.map(({ name }) => name)).toEqual([
        'cssparser',
        'cssparser-macros',
        'dtoa-short',
        'option-ext',
        'selectors',
      ]);
    });
  });
});
