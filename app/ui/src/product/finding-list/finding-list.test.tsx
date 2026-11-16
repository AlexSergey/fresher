import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Finding } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { fakeFrames, fakeReducedMotion, recordAnimations } from '../../test/animation';
import { BUSY } from '../../test/demo-scan';
import { FINDING_DICTIONARY, HIGH_FINDING, LOW_FINDING, SAFE_FINDING } from '../../test/finding-fixtures';
import { largeScan } from '../../test/large-scan';
import { fakeViewport } from '../../test/viewport';
import { FindingList, type FindingListProps } from './finding-list';

const ROW = 52;
// Ten rows fit the view.
const VIEW = 520;

const FINDINGS: readonly Finding[] = [HIGH_FINDING, SAFE_FINDING, LOW_FINDING];

const list = (props: Partial<FindingListProps> = {}): React.JSX.Element => (
  <FindingList
    failure={null}
    isSelected={() => false}
    label="Example caches"
    onRangeChange={vi.fn()}
    onToggle={vi.fn()}
    rowAt={(index) => FINDINGS[index]}
    total={FINDINGS.length}
    {...props}
  />
);

const rows = (): readonly HTMLElement[] => screen.queryAllByRole('listitem');

const viewport = (): HTMLElement => screen.getByRole('list', { name: 'Example caches' });

// The row's motion layer: what glides and what enters.
// eslint-disable-next-line testing-library/no-node-access -- the layer is presentation, it has no role to query by
const layerOf = (row: HTMLElement): HTMLElement | null => row.parentElement;

beforeEach(() => {
  setCoreDictionary(FINDING_DICTIONARY);
  fakeViewport(VIEW);
  fakeReducedMotion(false);
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe('FindingList', () => {
  describe('negative cases', () => {
    it('a row whose page is loading is a skeleton, not an empty row', () => {
      render(list({ rowAt: (index) => (index === 0 ? HIGH_FINDING : undefined), total: 40 }));

      expect(rows()[0]).toHaveAccessibleName('Example: old-project, High, 2.3 GB');
      expect(rows()[1]).toHaveAccessibleName('Loading…');
      expect(screen.getAllByRole('checkbox')).toHaveLength(1);
    });

    it('before its first page it shows 8 skeleton rows and asks for them', () => {
      const onRangeChange = vi.fn();
      render(list({ onRangeChange, total: undefined }));

      expect(rows()).toHaveLength(8);
      expect(rows().every((row) => row.getAttribute('aria-label') === 'Loading…')).toBe(true);
      expect(onRangeChange).toHaveBeenLastCalledWith(0, 7);
    });

    it('an empty list shows the empty state, not an empty frame', () => {
      render(list({ total: 0 }));

      expect(screen.getByText('Nothing found here')).toBeInTheDocument();
      expect(screen.queryByRole('list')).not.toBeInTheDocument();
      expect(screen.queryByText('Size')).not.toBeInTheDocument();
    });

    it("a refused page shows the core's text, and Try again asks for the rows again", () => {
      const frames = fakeFrames();
      const onRangeChange = vi.fn();
      const { rerender } = render(list({ onRangeChange, total: 1_000 }));

      viewport().scrollTop = 200 * ROW;
      fireEvent.scroll(viewport());
      frames.advance(16);
      rerender(list({ failure: { kind: 'refused', refusal: BUSY }, onRangeChange, total: 1_000 }));
      onRangeChange.mockClear();

      expect(screen.getByText('The list could not be loaded')).toBeInTheDocument();
      expect(
        screen.getByText('The app is already running a scan. Wait for it to finish, or cancel it.'),
      ).toBeInTheDocument();
      expect(screen.queryByRole('list')).not.toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: 'Try again' }));

      expect(onRangeChange).toHaveBeenCalledWith(194, 215);
    });

    it("an unreachable core shows the app's own text", () => {
      render(list({ failure: { kind: 'unreachable' }, total: undefined }));

      expect(screen.getByText('The app could not reach its core. Restart the app.')).toBeInTheDocument();
    });

    it('100,000 findings render no more than about 60 rows', () => {
      const { findings } = largeScan({ categories: 1, count: 100_000 });
      render(list({ rowAt: (index) => findings[index], total: findings.length }));

      expect(rows().length).toBeGreaterThan(0);
      expect(rows().length).toBeLessThanOrEqual(60);
      expect(rows()[0]).toHaveAttribute('aria-setsize', '100000');
    });

    it('Space on a loading row toggles nothing', () => {
      const onToggle = vi.fn();
      render(list({ onToggle, total: undefined }));

      const [first] = rows();
      act(() => {
        first?.focus();
      });
      fireEvent.keyDown(first ?? document.body, { key: ' ' });

      expect(onToggle).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('rows read title, level and size, under the column names', () => {
      render(list());

      expect(rows().map((row) => row.getAttribute('aria-label'))).toStrictEqual([
        'Example: old-project, High, 2.3 GB',
        'Example: editor-cache, Safe, 1.5 GB',
        'Example: installer.dmg, Low, 750 MB',
      ]);
      expect(screen.getByText('Size')).toBeInTheDocument();
    });

    it('Space toggles the focused row', () => {
      const onToggle = vi.fn();
      render(list({ onToggle }));

      const second = rows()[1];
      act(() => {
        second?.focus();
      });
      fireEvent.keyDown(second ?? document.body, { key: ' ' });

      expect(onToggle).toHaveBeenCalledWith(SAFE_FINDING);
    });

    it('a selected finding shows its checkbox ticked', () => {
      render(list({ isSelected: (finding) => finding.id === HIGH_FINDING.id }));

      expect(screen.getByRole('checkbox', { name: 'Select Example: old-project' })).toBeChecked();
      expect(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' })).not.toBeChecked();
    });

    it('the range sent for paging follows the scroll', () => {
      const frames = fakeFrames();
      const onRangeChange = vi.fn();
      render(list({ onRangeChange, rowAt: () => undefined, total: 100_000 }));

      expect(onRangeChange).toHaveBeenLastCalledWith(0, 15);

      viewport().scrollTop = 500 * ROW;
      fireEvent.scroll(viewport());
      frames.advance(16);

      expect(onRangeChange).toHaveBeenLastCalledWith(494, 515);
    });

    it('rows of a page that arrives fade in', () => {
      const { rerender } = render(list({ total: undefined }));

      rerender(list());

      const [first] = rows();

      expect(first).toHaveAccessibleName('Example: old-project, High, 2.3 GB');
      expect(layerOf(first ?? document.body)).toHaveAttribute('data-entering');
    });

    it('a refreshed page moves a row to its new index with a glide', () => {
      const animations = recordAnimations();
      const { rerender } = render(list());

      const refreshed = [SAFE_FINDING, HIGH_FINDING, LOW_FINDING];
      rerender(list({ rowAt: (index) => refreshed[index] }));

      const moved = animations.find((animation) => animation.element.textContent.includes('Example: editor-cache'));

      expect(moved?.keyframes).toStrictEqual([
        { transform: `translate(0px, ${String(ROW)}px)` },
        { transform: 'none' },
      ]);
      expect(animations.find((animation) => animation.element.textContent.includes('installer.dmg'))).toBeUndefined();
    });
  });
});
