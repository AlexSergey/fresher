import { render, screen, within } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import type { Segment } from '../segment';

import { UsageBar } from './usage-bar';

const css = readFileSync(path.join(import.meta.dirname, 'usage-bar.module.css'), 'utf8');

const SEGMENTS: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', size: '42 GB', tone: 'safe', value: 42 },
  { key: 'docker', label: 'Docker', size: '18 GB', tone: 'medium', value: 18 },
  { key: 'none', label: 'Nothing', size: '0 B', tone: 'high', value: 0 },
];

// eslint-disable-next-line testing-library/no-node-access -- the segments are decoration inside the bar's picture
const parts = (): readonly HTMLElement[] => [...screen.getByRole('img').querySelectorAll<HTMLElement>('[data-tone]')];

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

    it('a tiny segment keeps its legend row', () => {
      const tiny: readonly Segment[] = [
        { key: 'big', label: 'Big', size: '99 GB', tone: 'plain', value: 1000 },
        { key: 'tiny', label: 'Tiny', size: '1 MB', tone: 'system', value: 1 },
      ];

      render(<UsageBar label="Space found" legend segments={tiny} total={1001} />);

      expect(screen.getByText('Tiny')).toBeInTheDocument();
      expect(parts().filter((part) => part.dataset.tone === 'system')).toHaveLength(1);
    });

    it('free has no fill', () => {
      render(
        <UsageBar
          label="Disk"
          segments={[{ key: 'f', label: 'Free', size: '1 GB', tone: 'free', value: 1 }]}
          total={1}
        />,
      );

      expect(parts()[0]?.dataset.tone).toBe('free');
      expect(css).toMatch(/\[data-tone='free'\][^}]*background-color: transparent/u);
    });

    it('never animates a width', () => {
      expect(css).not.toMatch(/transition/u);
      expect(css).toMatch(/from \{\s*transform: scaleX\(0\);\s*\}/u);
    });
  });

  describe('positive cases', () => {
    it('each segment takes its share of the bar, in order and colour', () => {
      render(<UsageBar label="Space found" segments={SEGMENTS} total={120} />);

      expect(parts().map((part) => [part.dataset.tone, part.style.width])).toStrictEqual([
        ['safe', '35%'],
        ['medium', '15%'],
      ]);
    });

    it('each tone is drawn with its token', () => {
      for (const tone of ['plain', 'system', 'unknown'] as const) {
        expect(css).toMatch(new RegExp(`\\[data-tone='${tone}'\\][^}]*var\\(--tone-${tone}\\)`, 'u'));
      }
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
      expect(items.map((item) => item.querySelector('[data-tone]')?.getAttribute('data-tone'))).toStrictEqual(
        parts().map((part) => part.dataset.tone),
      );
    });
  });
});
