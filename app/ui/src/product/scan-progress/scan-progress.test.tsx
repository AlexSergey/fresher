import { fireEvent, render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { STORY_DICTIONARY } from '../scan-hero/story-data';
import { type ModuleProgress, ScanProgress } from './scan-progress';

const MODULES: readonly ModuleProgress[] = [
  { group: 'debug', module: 'demo', state: { kind: 'completed' } },
  {
    group: 'development',
    module: 'docker',
    state: { kind: 'failed', reason: { args: {}, key: 'docker.error.unreachable' } },
  },
  { group: 'development', module: 'xcode', state: { kind: 'running' } },
  { group: 'development', module: 'node', state: { kind: 'waiting' } },
  { group: 'development', module: 'logs', state: { kind: 'cancelled' } },
];

const chips = (): readonly HTMLElement[] =>
  within(screen.getByRole('group', { name: 'Parts of the scan' })).getAllByRole('listitem');

// The chip inside a list item: the element that carries its state and takes the focus.
// eslint-disable-next-line testing-library/no-node-access -- a chip is a styled span without a role of its own
const chipOf = (item: HTMLElement): HTMLElement | null => item.querySelector('[data-state]');

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('ScanProgress', () => {
  describe('negative cases', () => {
    it('shows no known progress: the core reports no total', () => {
      render(<ScanProgress entries={10} modules={[]} onStop={vi.fn()} />);

      expect(screen.getByRole('progressbar', { name: 'Scan progress' })).not.toHaveAttribute('aria-valuenow');
    });

    it('a chip never tells its state by its mark alone', () => {
      render(<ScanProgress entries={0} modules={MODULES} onStop={vi.fn()} />);

      expect(chips().map((chip) => chip.textContent)).toStrictEqual([
        'Docker, failed',
        'Xcode, scanning',
        'Node modules, waiting',
        'Logs, stopped',
        'Demo module, done',
      ]);
    });

    it('only a failed chip takes the focus', () => {
      render(<ScanProgress entries={0} modules={MODULES} onStop={vi.fn()} />);
      const focusable = chips()
        .map(chipOf)
        .filter((chip) => chip?.tabIndex === 0);

      expect(focusable.map((chip) => chip?.textContent)).toStrictEqual(['Docker, failed']);
    });
  });

  describe('positive cases', () => {
    it('a module not run for what it lacks shows as not available', () => {
      const unavailable: ModuleProgress = {
        group: 'debug',
        module: 'demo',
        state: { kind: 'unavailable', missing: [{ kind: 'all-files' }] },
      };
      render(<ScanProgress entries={0} modules={[unavailable]} onStop={vi.fn()} />);

      expect(chips().map((chip) => chip.textContent)).toStrictEqual(['Demo module, not available']);
    });

    it('names each group once, before its chips, Development first', () => {
      render(<ScanProgress entries={0} modules={MODULES} onStop={vi.fn()} />);

      expect(
        within(screen.getByRole('group', { name: 'Parts of the scan' }))
          .getAllByRole('list')
          .map((list) => list.getAttribute('aria-label')),
      ).toStrictEqual(['Development', 'Debug']);
    });

    it('Stop calls onStop', () => {
      const onStop = vi.fn();
      render(<ScanProgress entries={0} modules={MODULES} onStop={onStop} />);

      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));

      expect(onStop).toHaveBeenCalledOnce();
    });

    it('says how many items it looked at', () => {
      render(<ScanProgress entries={12_345} modules={[]} onStop={vi.fn()} />);

      const line = screen.getByText(/^Looked at/u);

      expect(line).toHaveTextContent(/^Looked at .* items$/u);
      expect(within(line).getByText('12,345', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('module chips show done and failed with the reason', () => {
      render(<ScanProgress entries={0} modules={MODULES} onStop={vi.fn()} />);
      const [failed, , , , done] = chips().map(chipOf);

      // eslint-disable-next-line testing-library/no-node-access -- the mark is a decorative icon, without a role
      expect(done?.querySelector('svg')).not.toBeNull();

      fireEvent.focusIn(failed ?? document.body);

      expect(screen.getByRole('tooltip')).toHaveTextContent('Docker could not be reached.');
      expect(failed).toHaveAccessibleDescription('Docker could not be reached.');
    });
  });
});
