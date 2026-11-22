import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { DEMO_INSIGHT, DEMO_INSIGHT_DICTIONARY, DEMO_INSIGHT_UNKNOWN } from '../../test/demo-insight';
import { InsightCard } from './insight-card';

describe('InsightCard', () => {
  beforeEach(() => {
    setCoreDictionary(DEMO_INSIGHT_DICTIONARY);
  });

  describe('negative cases', () => {
    it('shows no figure list when the insight has no figure', () => {
      render(<InsightCard insight={DEMO_INSIGHT_UNKNOWN} onDetails={vi.fn()} />);

      expect(screen.queryByRole('term')).toBeNull();
      expect(screen.getByText('Nothing is known about it.')).toBeVisible();
    });

    it('has no checkbox', () => {
      render(<InsightCard insight={DEMO_INSIGHT} onDetails={vi.fn()} />);

      expect(screen.queryByRole('checkbox')).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('shows the title, the summary and every figure with its size', () => {
      render(<InsightCard insight={DEMO_INSIGHT} onDetails={vi.fn()} />);

      expect(screen.getByRole('heading', { level: 3, name: 'Example tool' })).toBeVisible();
      expect(screen.getByText('Made-up data of a made-up tool: a disk file and what is inside it.')).toBeVisible();
      expect(screen.getAllByRole('term').map((term) => term.textContent)).toStrictEqual([
        'Largest it can grow',
        'Takes on the disk',
        'Used inside',
      ]);
      expect(screen.getAllByRole('definition').map((size) => size.textContent)).toStrictEqual([
        expect.stringContaining('68.7 GB'),
        expect.stringContaining('21.5 GB'),
        expect.stringContaining('9.7 GB'),
      ]);
    });

    it('calls onDetails from Details', () => {
      const onDetails = vi.fn();
      render(<InsightCard insight={DEMO_INSIGHT} onDetails={onDetails} />);

      fireEvent.click(screen.getByRole('button', { name: 'Details' }));

      expect(onDetails).toHaveBeenCalledTimes(1);
    });
  });
});
