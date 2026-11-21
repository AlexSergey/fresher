import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Overview } from '../../ipc/bindings.generated';

import { Announcer } from '../../design/feedback/announcer/announcer';
import { setCoreDictionary } from '../../i18n';
import { setMotion } from '../../test/animation';
import { ScanHero, type ScanHeroProps } from './scan-hero';
import { DEMO_FINISHED, DEMO_RUNNING, HERO_LEVELS, HERO_VOLUME, STORY_DICTIONARY } from './story-data';

const props = (overrides: Partial<ScanHeroProps> = {}): ScanHeroProps => ({
  failure: null,
  levels: HERO_LEVELS,
  onScan: vi.fn(),
  onStop: vi.fn(),
  overview: null,
  phase: 'idle',
  progress: null,
  volume: HERO_VOLUME,
  ...overrides,
});

const mountHero = (overrides: Partial<ScanHeroProps> = {}): ((next: Partial<ScanHeroProps>) => void) => {
  const { rerender } = render(
    <Announcer>
      <ScanHero {...props(overrides)} />
    </Announcer>,
  );

  return (next) => {
    rerender(
      <Announcer>
        <ScanHero {...props({ ...overrides, ...next })} />
      </Announcer>,
    );
  };
};

const running = (allocated: number): Overview => ({
  ...DEMO_RUNNING,
  found: { allocated, completeness: 'complete', logical: allocated, reclaimable: allocated },
});

// The ring's drawing has no roles: its arcs and the sweep are found by their markers.
/* eslint-disable testing-library/no-node-access -- the ring's layers are decoration, without roles */
const arcs = (): readonly Element[] => [...document.querySelectorAll('[data-ring="arcs"] [data-tone]')];
const sweep = (): Element | null => document.querySelector('[data-sweep]');
const rowOf = (key: string): HTMLElement | null => document.querySelector(`[data-key="${key}"]`);
/* eslint-enable testing-library/no-node-access */

// What the polite live region says, without the no-break space that makes a repeated text change.
const spoken = (): string => screen.getByRole('status').textContent.trim();

const wait = (ms: number): void => {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
};

beforeEach(async () => {
  setCoreDictionary(STORY_DICTIONARY);
  await setMotion('off');
});

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('ScanHero', () => {
  describe('negative cases', () => {
    it('idle shows the Scan button and no number', () => {
      mountHero();

      expect(screen.getByRole('button', { name: 'Scan' })).toBeInTheDocument();
      expect(screen.queryByText(/^in \d/u)).not.toBeInTheDocument();
      expect(screen.queryByText(/bytes|GB/u)).not.toBeInTheDocument();
    });

    it('shows no legend before a scan', () => {
      mountHero();

      expect(screen.queryByRole('list', { name: 'Parts of the disk' })).not.toBeInTheDocument();
    });

    it('a highlighted key that is no longer among the parts dims nothing', () => {
      const update = mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      fireEvent.pointerEnter(rowOf('free-6') as Element);
      update({ volume: { ...HERO_VOLUME, parts: HERO_VOLUME.parts.slice(0, 3) } });

      expect(arcs().filter((arc) => arc.hasAttribute('data-dimmed'))).toHaveLength(0);
    });

    it('shows no caption without a volume', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished', volume: null });

      expect(screen.queryByText(/^Space on .*disk$/u)).not.toBeInTheDocument();
      expect(screen.getByRole('img', { name: /^Space on the disk/u })).toBeInTheDocument();
    });

    it('the sweep is off when not scanning', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(sweep()).toBeNull();
    });

    it('a failed scan shows the core text and no ring segments', () => {
      mountHero({ failure: { args: {}, key: 'demo.error.failed' }, overview: DEMO_FINISHED, phase: 'failed' });

      expect(screen.getByText('The demo module could not build its example findings.')).toBeInTheDocument();
      expect(arcs()).toHaveLength(0);
      expect(screen.queryByRole('list', { name: 'Parts of the disk' })).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Scan' })).not.toBeInTheDocument();
    });

    it('an unknown found size shows — and never 0', () => {
      mountHero({
        overview: { ...DEMO_FINISHED, found: { allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 } },
        phase: 'finished',
      });

      expect(screen.getAllByText('Size not known')).not.toHaveLength(0);
      expect(screen.queryByText('0 bytes')).not.toBeInTheDocument();
    });

    it('the counting number is hidden from assistive technology', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });
      const hidden = screen.getAllByText('2.6', { exact: false, selector: '[aria-hidden="true"]' });

      // The hero's count: the number and its smaller unit, both hidden.
      expect(hidden.map((element) => element.textContent)).toContain('2.6GB');
      expect(screen.getAllByText('2.6 GB', { selector: '.u-visually-hidden' })).toHaveLength(1);
    });

    it('the announcer is not fed more than once per 5 seconds', () => {
      vi.useFakeTimers();
      const update = mountHero({ overview: running(1_000_000_000), phase: 'scanning' });

      expect(spoken()).toBe('Found 1 GB so far');

      update({ overview: running(2_000_000_000) });
      wait(1000);
      update({ overview: running(3_000_000_000) });
      wait(1000);

      expect(spoken()).toBe('Found 1 GB so far');

      wait(3000);

      expect(spoken()).toBe('Found 3 GB so far');
    });

    it('the Scan button cannot be pressed while it fades out', () => {
      mountHero({ overview: running(0), phase: 'scanning' });

      expect(screen.queryByRole('button', { name: 'Scan' })).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('Scan calls onScan', () => {
      const onScan = vi.fn();
      mountHero({ onScan });

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(onScan).toHaveBeenCalledOnce();
    });

    it('Try again scans again after a failure', () => {
      const onScan = vi.fn();
      mountHero({ failure: { args: {}, key: 'demo.error.failed' }, onScan, phase: 'failed' });

      fireEvent.click(screen.getByRole('button', { name: 'Try again' }));

      expect(onScan).toHaveBeenCalledOnce();
    });

    it('Stop calls onStop', () => {
      const onStop = vi.fn();
      mountHero({
        onStop,
        overview: DEMO_RUNNING,
        phase: 'scanning',
        progress: { entries: 10, modules: [{ group: 'debug', module: 'demo', state: { kind: 'running' } }] },
      });

      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));

      expect(onStop).toHaveBeenCalledOnce();
      expect(sweep()).not.toBeNull();
    });

    it('the ring draws the levels, then the disk parts, and leaves Free as the gap', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(arcs().map((arc) => arc.getAttribute('data-tone'))).toStrictEqual([
        'safe',
        'low',
        'medium',
        'high',
        'your-folders',
        'unknown',
        'mac-os',
        'swap-and-sleep',
        'disk-own',
      ]);
      expect(
        screen.getByRole('img', { name: /^Space on Macintosh HD: Safe to remove 40 GB, Low risk 14 GB,/u }),
      ).toBeInTheDocument();
    });

    it('the caption names the volume and its kind', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(screen.getByText('Space on Macintosh HD, internal disk')).toBeInTheDocument();
    });

    it('the legend has a row for every part, Free included', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });
      const rows = within(screen.getByRole('list', { name: 'Parts of the disk' })).getAllByRole('listitem');

      expect(rows.map((row) => row.getAttribute('data-key'))).toStrictEqual([
        'level-safe',
        'level-low',
        'level-medium',
        'level-high',
        'your-folders-1',
        'remainder-2',
        'mac-os-3',
        'swap-and-sleep-4',
        'disk-own-5',
        'free-6',
      ]);
    });

    it('a hover on an arc marks its legend row, and leaving clears it', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });
      const [safe] = arcs();
      const row = (): HTMLElement | null => rowOf('level-safe');

      expect(row()).not.toHaveAttribute('data-highlighted');

      fireEvent.pointerEnter(safe as Element);

      expect(row()).toHaveAttribute('data-highlighted');

      fireEvent.pointerLeave(safe as Element);

      expect(row()).not.toHaveAttribute('data-highlighted');
    });

    it('a hover on a legend row dims the other arcs', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      fireEvent.pointerEnter(rowOf('mac-os-3') as Element);

      expect(arcs().filter((arc) => arc.hasAttribute('data-dimmed'))).toHaveLength(arcs().length - 1);
    });

    it('the disk-changed note shows under the legend', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished', volume: { ...HERO_VOLUME, diskChanged: true } });

      expect(screen.getByText(/^The disk changed during the scan/u)).toBeInTheDocument();
    });

    it('one item in one category is said in the singular', () => {
      mountHero({
        overview: { ...DEMO_FINISHED, totals: DEMO_FINISHED.totals.slice(1) },
        phase: 'finished',
      });

      expect(screen.getByText('in 1 item · 1 category')).toBeInTheDocument();
    });

    it('a partial size is a lower bound', () => {
      mountHero({
        overview: { ...DEMO_FINISHED, found: { ...DEMO_FINISHED.found, completeness: 'partial' } },
        phase: 'partial',
      });

      expect(screen.getAllByText('at least 2.6 GB')).not.toHaveLength(0);
    });

    it('a finished scan announces its outcome once', () => {
      const update = mountHero({ overview: running(0), phase: 'scanning' });

      update({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(spoken()).toBe('The scan is complete. Found 2.6 GB.');

      update({ overview: running(3_000_000_000), phase: 'finished' });

      expect(spoken()).toBe('The scan is complete. Found 2.6 GB.');
    });

    it('a failed scan announces the core reason', () => {
      const update = mountHero({ overview: running(0), phase: 'scanning' });

      update({ failure: { args: {}, key: 'demo.error.failed' }, phase: 'failed' });

      expect(spoken()).toBe('The scan could not finish. The demo module could not build its example findings.');
    });

    it('a new scan counts again from 0', () => {
      const update = mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      update({ overview: running(0), phase: 'scanning' });

      expect(screen.getAllByText('0 bytes', { selector: '.u-visually-hidden' })).not.toHaveLength(0);
    });
  });
});
