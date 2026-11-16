import { globSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

// The CSS guard (docs/plans/003-base-ui/00-design.md, "The platform" and "Performance budget"): every stylesheet of
// `src/` stays inside the Safari 16.0 baseline and the motion budget.

// The only properties a transition or a keyframe may change: compositor-only, plus the named paint-only exceptions.
export const MOTION_PROPERTIES: readonly string[] = [
  'transform',
  'opacity',
  'filter',
  'stroke-dashoffset',
  'background-color',
  'border-color',
  'color',
];

// Features newer than Safari 16.0, with the version that brought them.
const OUTSIDE_BASELINE: readonly (readonly [RegExp, string])[] = [
  [/color-mix\(/u, 'color-mix() (Safari 16.2)'],
  [/@property\b/u, '@property (Safari 16.4)'],
  [/(?<![\w-])linear\(/u, 'linear() easing (Safari 17.2)'],
  [/@starting-style\b/u, '@starting-style (Safari 17.5)'],
  [/transition-behavior\b/u, 'transition-behavior (Safari 17.4)'],
  [/view-transition/u, 'View Transitions (Safari 18.0)'],
  [/content-visibility\b/u, 'content-visibility (Safari 18.0)'],
  [/\[popover\b|:popover-open\b/u, 'the Popover API (Safari 17.0)'],
];

// Words of a `transition` item that are not the property it names.
const TRANSITION_KEYWORDS = new Set([
  'allow-discrete',
  'ease',
  'ease-in',
  'ease-in-out',
  'ease-out',
  'linear',
  'normal',
  'step-end',
  'step-start',
]);

// Descriptors a keyframe step may hold besides the properties it animates.
const KEYFRAME_DESCRIPTORS = new Set(['animation-timing-function']);

type Block = {
  readonly body: string;
  readonly bodyStart: number;
  readonly parent: Block | null;
  readonly prelude: string;
  readonly preludeStart: number;
};

// A problem found at an offset of the source, before its line is known.
type Found = { readonly message: string; readonly offset: number };

type Problem = { readonly line: number; readonly message: string };

// Comments become spaces of the same length, so every offset (and line) stays where it was.
const stripComments = (source: string): string =>
  source.replaceAll(/\/\*[\s\S]*?\*\//gu, (comment) => comment.replaceAll(/[^\n]/gu, ' '));

const lineAt = (source: string, offset: number): number => source.slice(0, offset).split('\n').length;

// Every `{ … }` block with its prelude (the selector or at-rule before it) and its parent block.
const blocksOf = (source: string): readonly Block[] => {
  const blocks: Block[] = [];
  const open: { readonly block: Omit<Block, 'body'>; readonly start: number }[] = [];
  let preludeStart = 0;

  for (let index = 0; index < source.length; index += 1) {
    const char = source[index];

    if (char === '{') {
      const parent = open.at(-1);
      const prelude = source.slice(preludeStart, index);
      const block = {
        bodyStart: index + 1,
        parent: parent === undefined ? null : (blocks.find((done) => done.bodyStart === parent.start) ?? null),
        prelude: prelude.trim(),
        preludeStart: preludeStart + prelude.length - prelude.trimStart().length,
      };
      open.push({ block, start: index + 1 });
      // The block is recorded now so its children can find it; its body is filled when it closes.
      blocks.push({ ...block, body: '' });
      preludeStart = index + 1;
    } else if (char === '}') {
      const closing = open.pop();

      if (closing !== undefined) {
        const position = blocks.findIndex((done) => done.bodyStart === closing.start);
        const recorded = blocks[position];

        if (recorded !== undefined) {
          blocks[position] = { ...recorded, body: source.slice(closing.start, index) };
        }
      }

      preludeStart = index + 1;
    } else if (char === ';') {
      preludeStart = index + 1;
    }
  }

  return blocks;
};

const isAtRule = (block: Block): boolean => block.prelude.startsWith('@');

type Declaration = { readonly offset: number; readonly property: string; readonly value: string };

// The declarations written directly in a block (not inside its child blocks).
const declarationsOf = (block: Block): readonly Declaration[] => {
  const declarations: Declaration[] = [];
  let depth = 0;
  let start = 0;

  const take = (end: number): void => {
    const text = block.body.slice(start, end);
    const colon = text.indexOf(':');

    if (depth === 0 && colon > 0 && !text.trim().startsWith('@')) {
      const leading = text.length - text.trimStart().length;
      declarations.push({
        offset: block.bodyStart + start + leading,
        property: text.slice(0, colon).trim().toLowerCase(),
        value: text.slice(colon + 1).trim(),
      });
    }
  };

  for (let index = 0; index < block.body.length; index += 1) {
    const char = block.body[index];

    if (char === '{') {
      depth += 1;
    } else if (char === '}') {
      depth -= 1;
      start = index + 1;
    } else if (char === ';' && depth === 0) {
      take(index);
      start = index + 1;
    }
  }

  take(block.body.length);

  return declarations;
};

// Splits a value on the commas outside parentheses.
const splitTopLevel = (value: string, separator: RegExp): readonly string[] => {
  const parts: string[] = [];
  let depth = 0;
  let current = '';

  for (const char of value) {
    if (char === '(') {
      depth += 1;
    } else if (char === ')') {
      depth -= 1;
    }

    if (depth === 0 && separator.test(char)) {
      parts.push(current);
      current = '';
    } else {
      current += char;
    }
  }

  parts.push(current);

  return parts.map((part) => part.trim()).filter((part) => part !== '');
};

const isTime = (word: string): boolean => /^-?[\d.]+m?s$/u.test(word);

// The property one item of a `transition` shorthand names; `all` when it names none (the shorthand's default),
// `null` when it cannot be told (a custom property stands for it).
const transitionedProperty = (item: string): null | string => {
  const words = splitTopLevel(item, /\s/u);
  const named = words.find(
    (word) => !word.includes('(') && !isTime(word) && !TRANSITION_KEYWORDS.has(word.toLowerCase()),
  );

  if (named !== undefined) {
    return named.toLowerCase();
  }

  return words.some((word) => word.startsWith('var(')) ? null : 'all';
};

const transitionProblems = (declaration: Declaration): readonly string[] => {
  const { property, value } = declaration;

  if (property !== 'transition' && property !== 'transition-property') {
    return [];
  }

  if (value.trim().toLowerCase() === 'none') {
    return [];
  }

  const items = splitTopLevel(value, /,/u);
  const named =
    property === 'transition'
      ? items.map((item) => transitionedProperty(item))
      : items.map((item) => item.toLowerCase());

  return named.flatMap((name) => {
    if (name === null) {
      return [`${property} names no property it can be checked by: write the property out`];
    }

    if (name === 'all') {
      return [`${property} of all is refused: name the properties`];
    }

    return MOTION_PROPERTIES.includes(name) ? [] : [`${property} of ${name} is outside the motion budget`];
  });
};

const keyframeProblems = (block: Block): readonly Found[] => {
  if (block.parent === null || !/^@(?:-webkit-)?keyframes\b/u.test(block.parent.prelude)) {
    return [];
  }

  return declarationsOf(block)
    .filter(({ property }) => !MOTION_PROPERTIES.includes(property) && !KEYFRAME_DESCRIPTORS.has(property))
    .map(({ offset, property }) => ({ message: `a keyframe sets ${property}, outside the motion budget`, offset }));
};

const blockProblems = (block: Block): readonly Found[] => {
  const problems: Found[] = [];
  const declarations = declarationsOf(block);

  if (block.prelude.startsWith('&')) {
    problems.push({ message: 'nesting is not in the baseline (Safari 16.5)', offset: block.preludeStart });
  }

  if (!isAtRule(block) && block.body.includes('{')) {
    problems.push({ message: 'nesting is not in the baseline (Safari 16.5)', offset: block.bodyStart });
  }

  const properties = new Set(declarations.map(({ property }) => property));

  for (const declaration of declarations) {
    if (declaration.property === 'backdrop-filter' && !properties.has('-webkit-backdrop-filter')) {
      problems.push({
        message: 'backdrop-filter needs -webkit-backdrop-filter in the same block (Safari 16)',
        offset: declaration.offset,
      });
    }

    for (const message of transitionProblems(declaration)) {
      problems.push({ message, offset: declaration.offset });
    }
  }

  return [...problems, ...keyframeProblems(block)];
};

// Every problem of one stylesheet, with its line.
const checkCss = (source: string): readonly Problem[] => {
  const text = stripComments(source);
  const features = OUTSIDE_BASELINE.flatMap(([pattern, name]) =>
    [...text.matchAll(new RegExp(pattern, 'gu'))].map((match) => ({
      message: `${name} is not in the baseline`,
      offset: match.index,
    })),
  );
  const found = [...features, ...blocksOf(text).flatMap((block) => blockProblems(block))];

  return found
    .map(({ message, offset }) => ({ line: lineAt(text, offset), message }))
    .toSorted((first, second) => first.line - second.line);
};

const report = (file: string, problems: readonly Problem[]): readonly string[] =>
  problems.map(({ line, message }) => `${file}:${line}: ${message}`);

// Every stylesheet of `src/`, read from the disk: Vitest empties a CSS import, even with `?raw`.
const sourceFolder = path.join(import.meta.dirname, '..');
const stylesheets = globSync('**/*.css', { cwd: sourceFolder }).map(
  (file) => [file, readFileSync(path.join(sourceFolder, file), 'utf8')] as const,
);

describe('the CSS guard', () => {
  describe('negative cases', () => {
    it('refuses color-mix', () => {
      expect(report('a.css', checkCss('.a {\n  color: color-mix(in srgb, red, blue);\n}'))).toStrictEqual([
        'a.css:2: color-mix() (Safari 16.2) is not in the baseline',
      ]);
    });

    it('refuses @property', () => {
      expect(checkCss("@property --angle {\n  syntax: '<angle>';\n}")).toContainEqual({
        line: 1,
        message: '@property (Safari 16.4) is not in the baseline',
      });
    });

    it('refuses linear() as an easing, not a linear gradient', () => {
      expect(checkCss('.a { background: linear-gradient(red, blue); }')).toStrictEqual([]);
      expect(checkCss('.a { animation-timing-function: linear(0, 1); }')).toStrictEqual([
        { line: 1, message: 'linear() easing (Safari 17.2) is not in the baseline' },
      ]);
    });

    it('refuses the features newer than Safari 16', () => {
      const messages = checkCss(
        [
          '@starting-style { .a { opacity: 0; } }',
          '.a { transition-behavior: allow-discrete; }',
          '::view-transition-old(root) { opacity: 0; }',
          '.a { content-visibility: auto; }',
          '[popover] { opacity: 1; }',
        ].join('\n'),
      ).map(({ line }) => line);

      expect(messages).toStrictEqual([1, 2, 3, 4, 5]);
    });

    it('refuses nesting', () => {
      expect(checkCss('.a {\n  color: red;\n  .b { color: blue; }\n}')).toStrictEqual([
        { line: 1, message: 'nesting is not in the baseline (Safari 16.5)' },
      ]);
      expect(checkCss('.a {\n  &:hover { color: blue; }\n}').map(({ message }) => message)).toContain(
        'nesting is not in the baseline (Safari 16.5)',
      );
    });

    it('refuses a backdrop-filter without its prefixed twin', () => {
      expect(checkCss('.a {\n  backdrop-filter: blur(16px);\n}')).toStrictEqual([
        { line: 2, message: 'backdrop-filter needs -webkit-backdrop-filter in the same block (Safari 16)' },
      ]);
    });

    it('refuses a transition of width', () => {
      expect(checkCss('.a { transition: width 200ms ease; }')).toStrictEqual([
        { line: 1, message: 'transition of width is outside the motion budget' },
      ]);
      expect(checkCss('.a { transition-property: opacity, box-shadow; }')).toStrictEqual([
        { line: 1, message: 'transition-property of box-shadow is outside the motion budget' },
      ]);
    });

    it('refuses transition all', () => {
      expect(checkCss('.a { transition: all 200ms; }')).toStrictEqual([
        { line: 1, message: 'transition of all is refused: name the properties' },
      ]);
      expect(checkCss('.a { transition: 200ms ease-out; }')).toStrictEqual([
        { line: 1, message: 'transition of all is refused: name the properties' },
      ]);
    });

    it('refuses a transition whose property is hidden in a custom property', () => {
      expect(checkCss('.a { transition: var(--move); }')).toStrictEqual([
        { line: 1, message: 'transition names no property it can be checked by: write the property out' },
      ]);
    });

    it('refuses a keyframe that moves top', () => {
      expect(checkCss('@keyframes fall {\n  from { top: 0; }\n  to { top: 10px; }\n}')).toStrictEqual([
        { line: 2, message: 'a keyframe sets top, outside the motion budget' },
        { line: 3, message: 'a keyframe sets top, outside the motion budget' },
      ]);
    });
  });

  describe('positive cases', () => {
    it('accepts a transition of transform and opacity', () => {
      expect(
        checkCss(
          '.a { transition: transform var(--duration-fast) var(--ease-standard), opacity 120ms cubic-bezier(0, 0, 0.2, 1); }',
        ),
      ).toStrictEqual([]);
      expect(checkCss('.a { transition: none; }')).toStrictEqual([]);
    });

    it('accepts the prefixed backdrop-filter pair', () => {
      expect(checkCss('.a {\n  -webkit-backdrop-filter: blur(16px);\n  backdrop-filter: blur(16px);\n}')).toStrictEqual(
        [],
      );
    });

    it('accepts keyframes inside the budget and group rules around rules', () => {
      expect(
        checkCss(
          [
            '@layer components {',
            '  @media (prefers-reduced-motion: reduce) { .a { opacity: 1; } }',
            '  @keyframes rise { from { opacity: 0; transform: translateY(4px); animation-timing-function: ease; } }',
            '}',
          ].join('\n'),
        ),
      ).toStrictEqual([]);
    });

    it('ignores what is written in comments', () => {
      expect(checkCss('/* no color-mix( here */ .a { color: red; }')).toStrictEqual([]);
    });

    it('accepts the current styles', () => {
      expect(stylesheets.length).toBeGreaterThan(0);
      expect(stylesheets.flatMap(([file, source]) => report(file, checkCss(source)))).toStrictEqual([]);
    });
  });
});
