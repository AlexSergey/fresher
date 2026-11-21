import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CleanupReport } from '../../ipc/bindings.generated';

import { DURATION } from '../../design/tokens/motion';
import { setCoreDictionary } from '../../i18n';
import { fakeFrames, fakeReducedMotion, setMotion } from '../../test/animation';
import {
  CLEANUP_DICTIONARY,
  DELETED_REPORT,
  MOVED_REPORT,
  reportOf,
  STOPPED_REPORT,
  TOOL_PLAN,
  UNMEASURED_REPORT,
} from '../../test/cleanup-plan';
import { DEMO_DICTIONARY } from '../../test/demo-scan';
import { ResultSummary, type ResultSummaryProps } from './result-summary';

const title = (name: string): ResultSummaryProps['problems'][number]['title'] => ({
  args: { name: { kind: 'text', value: name } },
  key: 'demo.finding.title',
});

const PROBLEMS: ResultSummaryProps['problems'] = [
  { outcome: { kind: 'skipped', reason: { app: 'Safari', kind: 'in-use' } }, title: title('editor-cache') },
  { outcome: { kind: 'failed', reason: { args: {}, key: 'test.failed' } }, title: title('installer.dmg') },
  { outcome: { kind: 'skipped', reason: { kind: 'vanished' } }, title: null },
];

const TRASH_NOTE = "Cleanups move items to the sandbox's Trash: /Users/dev/box/home/.Trash";

const renderSummary = (props: Partial<ResultSummaryProps> = {}): ReturnType<typeof render> =>
  render(
    <ResultSummary
      onDone={vi.fn()}
      onSeeCleanup={vi.fn()}
      onShowInTrash={vi.fn()}
      problems={[]}
      report={MOVED_REPORT}
      {...props}
    />,
  );

// The counting copy of the freed size, hidden from assistive technology.
const counting = (): string =>
  // eslint-disable-next-line testing-library/no-node-access -- the copy has no role, by design
  document.querySelector('[data-numeric] > [aria-hidden]')?.textContent ?? '';

// eslint-disable-next-line testing-library/no-node-access -- the wave is decoration, hidden from assistive technology
const wave = (): Element | null => document.querySelector('[data-light-wave]');

beforeEach(() => {
  fakeReducedMotion(false);
  setCoreDictionary({ ...DEMO_DICTIONARY, ...CLEANUP_DICTIONARY, 'test.failed': 'macOS refused to move it.' });
});

afterEach(async () => {
  vi.useRealTimers();
  setCoreDictionary({});
  await setMotion(null);
});

describe('ResultSummary', () => {
  describe('negative cases', () => {
    it('the number does not count under reduced motion', async () => {
      await setMotion('reduced');
      fakeFrames();
      renderSummary();

      expect(counting()).toBe('2.6 GB');
      expect(wave()).not.toBeInTheDocument();
    });

    it('the light wave plays once', () => {
      fakeFrames();
      const { rerender } = renderSummary();
      const first = wave();

      expect(first).toBeInTheDocument();

      fireEvent.animationEnd(first as Element);

      expect(wave()).not.toBeInTheDocument();

      rerender(<ResultSummary onDone={vi.fn()} problems={[]} report={STOPPED_REPORT} />);

      expect(wave()).not.toBeInTheDocument();
    });

    it('the ambient light turns back from green after the wave', () => {
      fakeFrames();
      renderSummary();
      // eslint-disable-next-line testing-library/no-node-access -- the light is decoration
      const safe = (): Element | null => document.querySelector('[data-tone="safe"]');

      expect(safe()).toHaveAttribute('data-shown');

      fireEvent.animationEnd(wave() as Element);

      expect(safe()).not.toHaveAttribute('data-shown');
    });

    it('a trash result never says freed for the trash', () => {
      renderSummary();

      expect(screen.getByRole('region', { name: 'Moved to the Trash' })).toBeInTheDocument();
      expect(screen.queryByRole('heading', { name: 'Freed' })).toBeNull();
      expect(screen.getByText('Moved 3 items to the Trash')).toBeInTheDocument();
      expect(screen.getByText('You get the space back when you empty the Trash.')).toBeInTheDocument();
      expect(screen.getByText('Space freed on the disk now: about 100 MB (measured).')).toBeInTheDocument();
    });

    it('an unmeasured space says so', () => {
      renderSummary({ report: UNMEASURED_REPORT });

      expect(screen.getByText('The space freed on the disk now could not be measured.')).toBeInTheDocument();
      expect(screen.queryByText(/about/u)).toBeNull();
    });

    it('a stopped cleanup says how many were untouched', () => {
      renderSummary({ report: STOPPED_REPORT });

      expect(screen.getByText('Stopped. 2 items were not touched.')).toBeInTheDocument();
      expect(screen.getByText('Moved 1 item to the Trash')).toBeInTheDocument();
    });

    it('a cleanup stopped by the history says the core’s text', () => {
      renderSummary({ report: { ...STOPPED_REPORT, ended: 'stopped' } });

      expect(screen.queryByText(/^Stopped\. 2/u)).toBeNull();
      expect(screen.getByText(/could not be saved/u)).toBeInTheDocument();
    });

    it('Show in the Trash is absent after a permanent delete and when nothing moved', () => {
      const { rerender } = renderSummary({ report: DELETED_REPORT });

      expect(screen.queryByRole('button', { name: 'Show in the Trash' })).toBeNull();
      expect(screen.getByText('Deleted 3 items permanently')).toBeInTheDocument();
      expect(screen.queryByText('You get the space back when you empty the Trash.')).toBeNull();

      const nothingMoved: CleanupReport = { ...MOVED_REPORT, counts: { ...MOVED_REPORT.counts, moved: 0 } };
      rerender(<ResultSummary onDone={vi.fn()} onShowInTrash={vi.fn()} problems={[]} report={nothingMoved} />);

      expect(screen.queryByRole('button', { name: 'Show in the Trash' })).toBeNull();
      expect(screen.queryByRole('button', { name: 'See this cleanup' })).toBeNull();
    });

    it('a permanent cleanup shows no trash note, even when one is given', () => {
      renderSummary({ report: DELETED_REPORT, trashNote: TRASH_NOTE });

      expect(screen.queryByText(TRASH_NOTE)).toBeNull();
    });

    it('a trash cleanup without a trash note shows none', () => {
      renderSummary();

      expect(screen.queryByText(/sandbox/u)).toBeNull();
    });

    it('shows no list when every item was removed', () => {
      renderSummary();

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('a trash cleanup shows its trash note under its lines', () => {
      renderSummary({ trashNote: TRASH_NOTE });

      expect(screen.getByText(TRASH_NOTE)).toBeInTheDocument();
    });

    it('the size reaches its value', () => {
      const frames = fakeFrames();
      renderSummary();

      expect(counting()).toBe('0 bytes');

      // One frame hands the number its value; the count starts on the next.
      frames.advance(16);
      frames.advance(16 + DURATION.deliberate / 2);

      expect(counting()).not.toBe('2.6 GB');

      frames.advance(DURATION.deliberate);

      expect(counting()).toBe('2.6 GB');
      expect(screen.getByText('2.6 GB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('marks a partial size as a lower bound', async () => {
      await setMotion('off');
      renderSummary({ report: { ...MOVED_REPORT, moved: { ...MOVED_REPORT.moved, completeness: 'partial' } } });

      expect(counting()).toBe('2.6 GB+');
    });

    it('a permanent cleanup says freed, with the measured space next to the estimate', () => {
      renderSummary({ report: DELETED_REPORT });

      expect(screen.getByRole('region', { name: 'Freed' })).toBeInTheDocument();
      expect(screen.getByText('Measured on the disk: about 2.4 GB.')).toBeInTheDocument();
    });

    it('the result lists skipped items with their reasons', () => {
      renderSummary({ problems: PROBLEMS });

      const items = within(screen.getByRole('list', { name: '3 items were not removed' })).getAllByRole('listitem');

      expect(items).toHaveLength(3);
      expect(items[0]).toHaveTextContent('Example: editor-cache');
      expect(items[0]).toHaveTextContent('An app is using it, so Fresher left it alone. Quit Safari and try again.');
      expect(items[1]).toHaveTextContent('Example: installer.dmg');
      expect(items[1]).toHaveTextContent('macOS refused to move it.');
      expect(items[2]).toHaveTextContent('An item of this cleanup');
      expect(items[2]).toHaveTextContent('It is no longer there.');
    });

    it('a failed reason with a detail shows the disclosure, closed', () => {
      const reason = { args: { detail: { kind: 'text', value: 'raw tool line' } }, key: 'test.failed' } as const;

      renderSummary({ problems: [{ outcome: { kind: 'failed', reason }, title: title('installer.dmg') }] });

      expect(screen.getByRole('button', { name: 'Details' })).toBeInTheDocument();
      expect(screen.getByText('raw tool line')).not.toBeVisible();
    });

    it('items a tool removed are counted apart', () => {
      renderSummary({ report: reportOf(TOOL_PLAN, { outcomes: [{ kind: 'moved' }, { kind: 'tool-done' }] }) });

      expect(screen.getByText('Moved 1 item to the Trash')).toBeInTheDocument();
      expect(screen.getByText('1 item was removed by its own tool.')).toBeInTheDocument();
    });

    it('every way on calls its own handler', () => {
      const [onDone, onSeeCleanup, onShowInTrash] = [vi.fn(), vi.fn(), vi.fn()];
      renderSummary({ onDone, onSeeCleanup, onShowInTrash });

      fireEvent.click(screen.getByRole('button', { name: 'Show in the Trash' }));
      fireEvent.click(screen.getByRole('button', { name: 'See this cleanup' }));
      fireEvent.click(screen.getByRole('button', { name: 'Done' }));

      expect(onShowInTrash).toHaveBeenCalledOnce();
      expect(onSeeCleanup).toHaveBeenCalledOnce();
      expect(onDone).toHaveBeenCalledOnce();
    });
  });
});
