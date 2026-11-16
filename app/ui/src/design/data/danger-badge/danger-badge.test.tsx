import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { DangerBadge, type DangerBadgeProps, type Level } from './danger-badge';

const css = readFileSync(path.join(import.meta.dirname, 'danger-badge.module.css'), 'utf8');

const WORDS: Readonly<Record<Level, string>> = { high: 'High', low: 'Low', medium: 'Medium', safe: 'Safe' };

describe('DangerBadge', () => {
  describe('negative cases', () => {
    it('a danger badge without a label is a type error', () => {
      // @ts-expect-error -- the level's word is required: meaning never rests on colour alone
      const props: DangerBadgeProps = { level: 'high' };

      expect(props).not.toHaveProperty('label');
    });

    it('a small or medium badge takes no sentence', () => {
      // @ts-expect-error -- only the large badge carries its sentence
      const props: DangerBadgeProps = { description: 'Hard to get back.', label: 'High', level: 'high', size: 'small' };

      expect(props.size).toBe('small');
    });

    it('no badge pulses', () => {
      expect(css).not.toMatch(/animation/u);
    });
  });

  describe('positive cases', () => {
    it('each level has its own icon and word', () => {
      const icons = (['safe', 'low', 'medium', 'high'] as const).map((level) => {
        const { unmount } = render(<DangerBadge label={WORDS[level]} level={level} />);

        const icon = screen.getByText(WORDS[level]).previousElementSibling?.getAttribute('class') ?? '';

        expect(screen.getByText(WORDS[level])).toBeInTheDocument();
        unmount();

        return icon;
      });

      expect(new Set(icons).size).toBe(4);
      expect(icons[0]).toMatch(/shield-check/u);
      expect(icons[1]).toMatch(/lucide-shield(?!-)/u);
      expect(icons[2]).toMatch(/triangle-alert/u);
      expect(icons[3]).toMatch(/octagon-alert/u);
    });

    it('takes the level and the size, medium by default', () => {
      render(<DangerBadge label="Medium" level="medium" />);

      // eslint-disable-next-line testing-library/no-node-access -- the badge wraps its word
      const badge = screen.getByText('Medium').parentElement;

      expect(badge).toHaveAttribute('data-level', 'medium');
      expect(badge).toHaveAttribute('data-size', 'medium');
    });

    it('the large badge says its sentence', () => {
      render(<DangerBadge description="Hard to get back." label="High" level="high" size="large" />);

      expect(screen.getByText('Hard to get back.')).toBeInTheDocument();
      expect(screen.getByText('High')).toBeInTheDocument();
    });

    it('the High badge glows at rest', () => {
      expect(css).toMatch(/\[data-level='high'\][^}]*box-shadow: var\(--glow-high\)/u);
    });
  });
});
