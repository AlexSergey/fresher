import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { DURATION, EASING, STAGGER } from './motion';

// The tokens of the first `:root` block of motion.css (the full motion), by name.
const cssTokens = (css: string): ReadonlyMap<string, string> => {
  const root = /:root\s*\{([^}]*)\}/u.exec(css)?.[1] ?? '';

  return new Map(
    [...root.matchAll(/(--[\w-]+):([^;]+);/gu)].map(([, name, value]) => [name ?? '', (value ?? '').trim()]),
  );
};

// The tokens motion.ts stands for, as CSS would write them.
const mirrored = (): ReadonlyMap<string, string> =>
  new Map([
    ...Object.entries(DURATION).map(([name, ms]) => [`--duration-${name}`, `${ms}ms`] as const),
    ...Object.entries(EASING).map(([name, easing]) => [`--ease-${name}`, easing] as const),
    ['--stagger', `${STAGGER.step}ms`],
    ['--stagger-steps', String(STAGGER.steps)],
  ]);

const motionCss = readFileSync(path.join(import.meta.dirname, 'motion.css'), 'utf8');

describe('the motion tokens', () => {
  describe('negative cases', () => {
    it('notices a duration that differs', () => {
      const css = ':root { --duration-fast: 150ms; }';

      expect(cssTokens(css).get('--duration-fast')).not.toBe(mirrored().get('--duration-fast'));
    });
  });

  describe('positive cases', () => {
    it('motion.ts has the values of motion.css', () => {
      const css = cssTokens(motionCss);

      for (const [name, value] of mirrored()) {
        expect({ name, value: css.get(name) }).toStrictEqual({ name, value });
      }
    });

    it('motion.css has no duration or easing that motion.ts lacks', () => {
      const timing = [...cssTokens(motionCss).keys()].filter(
        (name) => name.startsWith('--duration-') || name.startsWith('--ease-') || name.startsWith('--stagger'),
      );

      expect(timing.toSorted()).toStrictEqual([...mirrored().keys()].toSorted());
    });
  });
});
