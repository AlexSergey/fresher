import { render, screen, within } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import type { Segment } from '../segment';

import { UsageBar } from './usage-bar';

const css = readFileSync(path.join(import.meta.dirname, 'usage-bar.module.css'), 'utf8');

const SEGMENTS: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', level: 'safe', size: '42 GB', value: 42 },
  { key: 'docker', label: 'Docker', level: 'medium', size: '18 GB', value: 18 },
  { key: 'none', label: 'Nothing', level: 'high', size: '0 B', value: 0 },
];

// eslint-disable-next-line testing-library/no-node-access -- the segments are decoration inside the bar's picture
const parts = (): readonly HTMLElement[] => [...screen.getByRole('img').querySelectorAll<HTMLElement>('[data-level]')];

describe('UsageBar', () => {
  describe('negative cases', () => {
    it('an empty bar draws only its track and says its name', () => {
      render(<UsageBar label="Space found" segments={[]} total={100} />);

      expect(screen.getByRole('img', { name: 'Space found' })).toBeInTheDocument();
      expect(parts()).toHaveLength(0);
    });

    it('a segment of 0 is not drawn, nor listed', () => {
      render(<UsageBar label="Space found" legend segments={SEGMENTS} total={100} />);

      expect(parts()).toHaveLength(2);
      expect(screen.queryByText('Nothing')).not.toBeInTheDocument();
    });

    it('shows no legend unless asked', () => {
      render(<UsageBar label="Space found" segments={SEGMENTS} total={100} />);

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('never animates a width', () => {
      expect(css).not.toMatch(/transition/u);
      expect(css).toMatch(/from \{\s*transform: scaleX\(0\);\s*\}/u);
    });
  });

  describe('positive cases', () => {
    it('each segment takes its share of the bar, in order and colour', () => {
      render(<UsageBar label="Space found" segments={SEGMENTS} total={120} />);

      expect(parts().map((part) => [part.dataset.level, part.style.width])).toStrictEqual([
        ['safe', '35%'],
        ['medium', '15%'],
      ]);
    });

    it("the bar's label lists every segment", () => {
      render(<UsageBar label="Space found" segments={SEGMENTS} total={100} />);

      expect(screen.getByRole('img', { name: 'Space found: Xcode 42 GB, Docker 18 GB' })).toBeInTheDocument();
    });

    it("the usage bar's legend matches its segments", () => {
      render(<UsageBar label="Space found" legend segments={SEGMENTS} total={100} />);
      const items = within(screen.getByRole('list')).getAllByRole('listitem');

      expect(items.map((item) => item.textContent)).toStrictEqual(['Xcode42 GB', 'Docker18 GB']);
      // eslint-disable-next-line testing-library/no-node-access -- the dot is decoration beside the label
      expect(items.map((item) => item.querySelector('[data-level]')?.getAttribute('data-level'))).toStrictEqual(
        parts().map((part) => part.dataset.level),
      );
    });
  });
});
