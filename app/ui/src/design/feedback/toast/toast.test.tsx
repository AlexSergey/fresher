import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { Toast } from './toast';

const css = readFileSync(path.join(import.meta.dirname, 'toast.module.css'), 'utf8');

describe('Toast', () => {
  describe('negative cases', () => {
    it('has no action unless given', () => {
      render(<Toast closeLabel="Close" onClose={vi.fn()} text="Scan started." tone="info" />);

      expect(screen.getAllByRole('button')).toHaveLength(1);
    });

    it('is never blurred', () => {
      expect(css).not.toMatch(/backdrop-filter/u);
    });
  });

  describe('positive cases', () => {
    it('names its close button', () => {
      render(<Toast closeLabel="Close" onClose={vi.fn()} text="Scan started." tone="info" />);

      expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
    });

    it('rises --distance-2 and fades in', () => {
      expect(css).toMatch(/animation: toast-enter var\(--duration-base\) var\(--ease-enter\)/u);
      expect(css).toMatch(/transform: translateY\(var\(--distance-2\)\)/u);
    });
  });
});
