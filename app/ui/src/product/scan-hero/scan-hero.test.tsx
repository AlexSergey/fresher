import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Overview } from '../../ipc/bindings.generated';

import { Announcer } from '../../design/feedback/announcer/announcer';
import { setCoreDictionary } from '../../i18n';
import { setMotion } from '../../test/animation';
import { ScanHero, type ScanHeroProps } from './scan-hero';
import { DEMO_FINISHED, DEMO_RUNNING, STORY_DICTIONARY } from './story-data';

const props = (overrides: Partial<ScanHeroProps> = {}): ScanHeroProps => ({
  failure: null,
  onScan: vi.fn(),
  onStop: vi.fn(),
  overview: null,
  phase: 'idle',
  progress: null,
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
  found: { allocated, completeness: 'complete', logical: allocated },
});

// The ring's drawing has no roles: its arcs and the sweep are found by their markers.
/* eslint-disable testing-library/no-node-access -- the ring's layers are decoration, without roles */
const arcs = (): readonly Element[] => [...document.querySelectorAll('[data-ring="arcs"] [data-level]')];
const sweep = (): Element | null => document.querySelector('[data-sweep]');
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
      expect(screen.getAllByText('Not scanned yet')).toHaveLength(3);
    });

    it('the sweep is off when not scanning', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(sweep()).toBeNull();
    });

    it('a failed scan shows the core text and no ring segments', () => {
      mountHero({ failure: { args: {}, key: 'demo.error.failed' }, overview: DEMO_FINISHED, phase: 'failed' });

      expect(screen.getByText('The demo module could not build its example findings.')).toBeInTheDocument();
      expect(arcs()).toHaveLength(0);
      expect(screen.queryByRole('button', { name: 'Scan' })).not.toBeInTheDocument();
    });

    it('an unknown found size shows — and never 0', () => {
      mountHero({
        overview: { ...DEMO_FINISHED, found: { allocated: 0, completeness: 'unknown', logical: 0 } },
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
      // The hero and its stat tile each give the real value to assistive technology.
      expect(screen.getAllByText('2.6 GB', { selector: '.u-visually-hidden' })).toHaveLength(2);
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
        progress: { entries: 10, modules: [{ module: 'demo', state: { kind: 'running' } }] },
      });

      fireEvent.click(screen.getByRole('button', { name: 'Stop' }));

      expect(onStop).toHaveBeenCalledOnce();
      expect(sweep()).not.toBeNull();
    });

    it('segments follow the totals in the core order and their highest levels', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(arcs().map((arc) => arc.getAttribute('data-level'))).toStrictEqual(['safe', 'low']);
      expect(
        screen.getByRole('img', { name: 'Space found: Example caches 1.8 GB, Example downloads 750 MB' }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole('img', { name: 'Space found by category: Example caches 1.8 GB, Example downloads 750 MB' }),
      ).toBeInTheDocument();
    });

    it('the centre shows found with its plural line', () => {
      mountHero({ overview: DEMO_FINISHED, phase: 'finished' });

      expect(screen.getByText('in 3 items · 2 categories')).toBeInTheDocument();
      expect(screen.getByRole('group', { name: 'Items' })).toHaveTextContent('3');
      expect(screen.getByRole('group', { name: 'Categories' })).toHaveTextContent('2');
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
