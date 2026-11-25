import { globSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

// The contrast of the design's pairs, computed from the token files (WCAG 2.2: 4.5 to 1 for text, 3 to 1 for control
// outlines and icons). A colour with alpha is composited over the surface it is used on.

type Pair = {
  readonly bar: number;
  readonly fg: string;
  readonly on: string;
};

type Rgba = readonly [number, number, number, number];

const TEXT = 4.5;
const NON_TEXT = 3;

const tokensFolder = import.meta.dirname;
const sourceFolder = path.join(tokensFolder, '..', '..');
const read = (file: string): string => readFileSync(path.join(tokensFolder, file), 'utf8');

// Removes the blocks of `@media` rules: the tokens are checked as they are by default.
const withoutMedia = (css: string): string => {
  let result = css;
  let start = result.indexOf('@media');

  while (start !== -1) {
    let depth = 0;
    let end = result.indexOf('{', start);

    for (; end < result.length; end += 1) {
      if (result[end] === '{') {
        depth += 1;
      } else if (result[end] === '}') {
        depth -= 1;

        if (depth === 0) {
          break;
        }
      }
    }

    result = result.slice(0, start) + result.slice(end + 1);
    start = result.indexOf('@media');
  }

  return result;
};

const tokensOf = (css: string): ReadonlyMap<string, string> =>
  new Map(
    [...withoutMedia(css.replaceAll(/\/\*[\s\S]*?\*\//gu, '')).matchAll(/(--[\w-]+):([^;]+);/gu)].map(
      ([, name, value]) => [name ?? '', (value ?? '').trim()],
    ),
  );

const parseColour = (value: string): null | Rgba => {
  const hex = /^#([\da-f]{3}|[\da-f]{6})$/iu.exec(value)?.[1];

  if (hex !== undefined) {
    const full = hex.length === 3 ? hex.replaceAll(/[\da-f]/giu, '$&$&') : hex;

    return [
      Number.parseInt(full.slice(0, 2), 16),
      Number.parseInt(full.slice(2, 4), 16),
      Number.parseInt(full.slice(4, 6), 16),
      1,
    ];
  }

  const rgb = /^rgb\((\d+) (\d+) (\d+) \/ ([\d.]+)%\)$/u.exec(value);

  if (rgb === null) {
    return null;
  }

  return [Number(rgb[1]), Number(rgb[2]), Number(rgb[3]), Number(rgb[4]) / 100];
};

// The colour a token stands for, following `var()` references.
const resolve = (tokens: ReadonlyMap<string, string>, name: string): Rgba => {
  const value = tokens.get(name);

  if (value === undefined) {
    throw new Error(`${name} is not defined`);
  }

  const reference = /^var\((--[\w-]+)\)$/u.exec(value)?.[1];

  if (reference !== undefined) {
    return resolve(tokens, reference);
  }

  const colour = parseColour(value);

  if (colour === null) {
    throw new Error(`${name} is not a colour: ${value}`);
  }

  return colour;
};

const over = ([r, g, b, alpha]: Rgba, [br, bg, bb]: Rgba): Rgba => [
  r * alpha + br * (1 - alpha),
  g * alpha + bg * (1 - alpha),
  b * alpha + bb * (1 - alpha),
  1,
];

const luminance = ([r, g, b]: Rgba): number => {
  const linear = (channel: number): number => {
    const value = channel / 255;

    return value <= 0.040_45 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  };

  return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
};

const contrast = (first: Rgba, second: Rgba): number => {
  const [light, dark] = [luminance(first), luminance(second)].toSorted((a, b) => b - a) as [number, number];

  return (light + 0.05) / (dark + 0.05);
};

// The pairs below their bar, each with its ratio.
const failingPairs = (tokens: ReadonlyMap<string, string>, pairs: readonly Pair[]): readonly string[] =>
  pairs.flatMap(({ bar, fg, on }) => {
    const surface = resolve(tokens, on);
    const ratio = contrast(over(resolve(tokens, fg), surface), surface);

    return ratio >= bar ? [] : [`${fg} on ${on}: ${ratio.toFixed(2)} : 1, below ${bar} : 1`];
  });

// Colour values written outside primitive.css, as `file:line`.
const colourValuesIn = (file: string, css: string): readonly string[] =>
  css
    .replaceAll(/\/\*[\s\S]*?\*\//gu, (comment) => comment.replaceAll(/[^\n]/gu, ' '))
    .split('\n')
    .flatMap((line, index) =>
      /:[^;{}]*(?:#[\da-f]{3,8}\b|rgba?\(|hsla?\()/iu.test(line) ? [`${file}:${index + 1}`] : [],
    );

const LEVELS = ['safe', 'low', 'medium', 'high'] as const;
const TEXT_SURFACES = ['--surface-window', '--surface-raised', '--surface-overlay', '--surface-hover'] as const;

const DESIGN_PAIRS: readonly Pair[] = [
  ...['--text-primary', '--text-secondary', '--text-tertiary'].flatMap((fg) =>
    TEXT_SURFACES.map((on) => ({ bar: TEXT, fg, on })),
  ),
  // The active sidebar row's trailing text, on its pill.
  { bar: TEXT, fg: '--text-secondary', on: '--surface-pressed' },
  { bar: TEXT, fg: '--accent-text', on: '--surface-raised' },
  { bar: TEXT, fg: '--accent-text', on: '--surface-overlay' },
  { bar: TEXT, fg: '--text-on-fill', on: '--accent-fill-top' },
  { bar: TEXT, fg: '--text-on-fill', on: '--danger-fill-top' },
  ...LEVELS.flatMap((level) => [
    { bar: TEXT, fg: `--level-${level}-fg`, on: `--level-${level}-tint` },
    { bar: TEXT, fg: `--level-${level}-fg`, on: '--surface-raised' },
  ]),
  { bar: NON_TEXT, fg: '--border-control', on: '--surface-raised' },
  { bar: TEXT, fg: '--text-on-fill', on: '--menu-item-active' },
  { bar: TEXT, fg: '--text-legend', on: '--gray-900' },
  { bar: TEXT, fg: '--tile-line-medium', on: '--amber-950' },
  { bar: TEXT, fg: '--text-tertiary', on: '--gray-875' },
  // The scanning card's "Looking at" line at the dimmest frame of its shimmer, on the card's lightest stop.
  { bar: TEXT, fg: '--text-tertiary', on: '--night-800' },
];

const FIXTURE = tokensOf(`
  :root {
    --surface: #15171b;
    --dim-text: #5a616d;
    --faint-outline: rgb(255 255 255 / 16%);
  }
`);

describe('the contrast of the tokens', () => {
  describe('negative cases', () => {
    it('fails a text pair below 4.5 to 1', () => {
      expect(failingPairs(FIXTURE, [{ bar: TEXT, fg: '--dim-text', on: '--surface' }])).toStrictEqual([
        '--dim-text on --surface: 2.88 : 1, below 4.5 : 1',
      ]);
    });

    it('fails a control outline below 3 to 1', () => {
      expect(failingPairs(FIXTURE, [{ bar: NON_TEXT, fg: '--faint-outline', on: '--surface' }])).toStrictEqual([
        '--faint-outline on --surface: 1.63 : 1, below 3 : 1',
      ]);
    });

    it('refuses a colour value outside primitive.css', () => {
      expect(colourValuesIn('a.css', '.a {\n  color: #fff;\n  background: rgb(0 0 0 / 50%);\n}')).toStrictEqual([
        'a.css:2',
        'a.css:3',
      ]);
    });

    it('refuses a token that is not defined', () => {
      expect(() => failingPairs(FIXTURE, [{ bar: TEXT, fg: '--missing', on: '--surface' }])).toThrow(
        '--missing is not defined',
      );
    });
  });

  describe('positive cases', () => {
    it('passes every pair of the design', () => {
      const tokens = new Map([...tokensOf(read('primitive.css')), ...tokensOf(read('semantic.css'))]);

      expect(failingPairs(tokens, DESIGN_PAIRS)).toStrictEqual([]);
    });

    it('writes colour values only in primitive.css', () => {
      const others = globSync('**/*.css', { cwd: sourceFolder }).filter(
        (file) => path.basename(file) !== 'primitive.css',
      );

      expect(others.length).toBeGreaterThan(0);
      expect(
        others.flatMap((file) => colourValuesIn(file, readFileSync(path.join(sourceFolder, file), 'utf8'))),
      ).toStrictEqual([]);
    });
  });
});
