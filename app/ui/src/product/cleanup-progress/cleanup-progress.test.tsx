import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';

import { CleanupProgress, type CleanupProgressProps } from './cleanup-progress';

const here = import.meta.dirname;
const sources = ['cleanup-progress.tsx', 'cleanup-progress.module.css'].map((file) =>
  readFileSync(path.join(here, file), 'utf8'),
);

const renderProgress = (props: Partial<CleanupProgressProps> = {}): ReturnType<typeof render> =>
  render(
    <CleanupProgress
      done={11}
      method="trash"
      now="Example: editor-cache"
      onStop={vi.fn()}
      stopping={false}
      total={128}
      {...props}
    />,
  );

describe('CleanupProgress', () => {
  describe('negative cases', () => {
    it('the progress has no motion', () => {
      renderProgress();

      for (const source of sources) {
        expect(source).not.toMatch(/AnimatedNumber|--stagger|--ease-settle|--glow|-glow\)|ScanSweep|AmbientLight/u);
      }
      expect(sources[1]).toMatch(/\[role='progressbar'\] > \* \{\s*transition: none;/u);
      expect(screen.getByRole('progressbar')).not.toHaveAttribute('data-indeterminate');
    });

    it('stop says stopping and cannot be pressed again', () => {
      const onStop = vi.fn();
      renderProgress({ onStop, stopping: true });

      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));

      expect(onStop).not.toHaveBeenCalled();
      expect(screen.getByRole('status')).toHaveTextContent('Stopping after this item…');
    });

    it('an item in hand that is not known shows no title', () => {
      renderProgress({ now: null });

      expect(screen.queryByText(/^Now:/u)).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('progress follows the items', () => {
      const { rerender } = renderProgress();

      expect(screen.getByRole('region', { name: 'Moving 12 of 128 to the Trash…' })).toBeInTheDocument();
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '9');
      expect(screen.getByText('Now: Example: editor-cache')).toBeInTheDocument();

      rerender(
        <CleanupProgress done={128} method="permanent" now={null} onStop={vi.fn()} stopping={false} total={128} />,
      );

      expect(screen.getByText('Deleting 128 of 128…')).toBeInTheDocument();
      expect(screen.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');
    });

    it('Stop asks once', () => {
      const onStop = vi.fn();
      renderProgress({ onStop });

      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));

      expect(onStop).toHaveBeenCalledOnce();
    });
  });
});
