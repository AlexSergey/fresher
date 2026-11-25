import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  buildBlock,
  type CatalogueRow,
  type Category,
  END_MARKER,
  type Levels,
  parseCatalogue,
  parseLevels,
  type Problem,
  readCategories,
  replaceBlock,
  START_MARKER,
} from './readme-catalogue.ts';

const LEVELS_TABLE = [
  '## Danger levels',
  '',
  'Some words.',
  '',
  '| Level | Meaning | Examples |',
  '| --- | --- | --- |',
  '| **Safe** | Rebuilt automatically. | logs |',
  '| **Low** | Rebuilt on demand. | caches |',
  '| **Medium** | Restored by hand. | settings |',
  '| **High** | May hold the only copy. | backups |',
].join('\n');

const LEVELS: Levels = new Map([
  ['High', 'May hold the only copy.'],
  ['Low', 'Rebuilt on demand.'],
  ['Medium', 'Restored by hand.'],
  ['Safe', 'Rebuilt automatically.'],
]);

const categoryFile = ({
  danger = 'Low',
  example = '`~/.npm/_cacache`',
  title = "npm's download cache",
}: { readonly danger?: string; readonly example?: string; readonly title?: string } = {}): string =>
  [
    `# ${title}`,
    '',
    `**Danger level:** ${danger} · **Rule:** none · **Since:** 0.1.0`,
    '',
    `**Example:** ${example}`,
    '',
    '## What it is',
  ].join('\n');

const NPM_ROW: CatalogueRow = { category: 'dev-caches.npm', danger: 'Low', file: 'dev-caches-npm.md' };

const kindsOf = (rows: readonly CatalogueRow[], files: ReadonlyMap<string, string>): readonly Problem['kind'][] =>
  readCategories(rows, files).problems.map((problem) => problem.kind);

const kindsOfFile = (text: string, row: CatalogueRow = NPM_ROW): readonly Problem['kind'][] =>
  kindsOf([row], new Map([[row.file, text]]));

describe('readCategories', () => {
  describe('negative cases', () => {
    it('refuses a row whose file does not exist', () => {
      assert.deepEqual(kindsOf([NPM_ROW], new Map()), ['missing-file']);
    });

    it('refuses a file no row names', () => {
      const files = new Map([
        ['dev-caches-npm.md', categoryFile()],
        ['dev-caches-yarn.md', categoryFile()],
      ]);

      assert.deepEqual(kindsOf([NPM_ROW], files), ['unlisted-file']);
    });

    it('refuses a category named twice', () => {
      const files = new Map([
        ['dev-caches-npm.md', categoryFile()],
        ['other.md', categoryFile()],
      ]);

      assert.deepEqual(kindsOf([NPM_ROW, { ...NPM_ROW, file: 'other.md' }], files), ['duplicate-row']);
      assert.match(
        readCategories([NPM_ROW, { ...NPM_ROW, file: 'other.md' }], files).problems[0]?.message ?? '',
        /category dev-caches\.npm/u,
      );
    });

    it('refuses a file named twice', () => {
      const files = new Map([['dev-caches-npm.md', categoryFile()]]);

      assert.deepEqual(kindsOf([NPM_ROW, { ...NPM_ROW, category: 'dev-caches.other' }], files), ['duplicate-row']);
    });

    it('refuses a file without a title line', () => {
      assert.deepEqual(kindsOfFile(categoryFile().replace("# npm's download cache", 'npm cache')), ['no-title']);
    });

    it('refuses a file without a danger level line', () => {
      assert.deepEqual(kindsOfFile(categoryFile().replace('**Danger level:**', 'Danger:')), ['danger-mismatch']);
    });

    it('refuses a danger level that is not one of the four', () => {
      assert.deepEqual(kindsOfFile(categoryFile({ danger: 'Severe' })), ['danger-mismatch']);
    });

    it("refuses a danger level that differs from the row's", () => {
      assert.deepEqual(kindsOfFile(categoryFile(), { ...NPM_ROW, danger: 'Safe' }), ['danger-mismatch']);
    });

    it('refuses a file without an example line', () => {
      assert.deepEqual(kindsOfFile(categoryFile().replace('**Example:**', 'Example:')), ['no-example']);
    });

    it('refuses an empty example line', () => {
      assert.deepEqual(kindsOfFile(categoryFile({ example: '' })), ['no-example']);
    });

    it('refuses an example over 80 characters', () => {
      assert.deepEqual(kindsOfFile(categoryFile({ example: 'x'.repeat(81) })), ['bad-example']);
    });

    it('refuses an example holding a pipe', () => {
      assert.deepEqual(kindsOfFile(categoryFile({ example: 'a | b' })), ['bad-example']);
    });

    it('names the category file in every problem', () => {
      const [problem] = readCategories([NPM_ROW], new Map()).problems;

      assert.equal(problem?.where, 'docs/categories/dev-caches-npm.md');
    });
  });

  describe('positive cases', () => {
    it('reads the title, the danger level and the example', () => {
      const { categories, problems } = readCategories([NPM_ROW], new Map([[NPM_ROW.file, categoryFile()]]));

      assert.deepEqual(problems, []);
      assert.deepEqual(categories, [
        {
          category: 'dev-caches.npm',
          danger: 'Low',
          example: '`~/.npm/_cacache`',
          file: 'dev-caches-npm.md',
          title: "npm's download cache",
        },
      ]);
    });

    it('accepts an example of exactly 80 characters', () => {
      assert.deepEqual(kindsOfFile(categoryFile({ example: 'x'.repeat(80) })), []);
    });

    it("keeps the catalogue's row order", () => {
      const yarn: CatalogueRow = { category: 'dev-caches.yarn', danger: 'Low', file: 'dev-caches-yarn.md' };
      const files = new Map([
        [NPM_ROW.file, categoryFile({ title: 'npm' })],
        [yarn.file, categoryFile({ title: 'Yarn' })],
      ]);

      const titles = readCategories([yarn, NPM_ROW], files).categories.map((category) => category.title);

      assert.deepEqual(titles, ['Yarn', 'npm']);
    });
  });
});

describe('parseLevels', () => {
  describe('negative cases', () => {
    it('refuses a levels table without High', () => {
      const { problems } = parseLevels(LEVELS_TABLE.replace('| **High** | May hold the only copy. | backups |', ''));

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['no-levels'],
      );
      assert.match(problems[0]?.message ?? '', /High/u);
    });
  });

  describe('positive cases', () => {
    it('reads the meaning of each level', () => {
      const { levels, problems } = parseLevels(LEVELS_TABLE);

      assert.deepEqual(problems, []);
      assert.deepEqual(levels, LEVELS);
    });
  });
});

describe('parseCatalogue', () => {
  describe('positive cases', () => {
    it('reads an aligned and an unaligned row', () => {
      const readme = [
        '## Catalogue',
        '',
        '| Category         | Danger level | Rule | File                                       |',
        '| ---------------- | ------------ | ---- | ------------------------------------------ |',
        '| `dev-caches.npm` | Low          | none | [`dev-caches-npm.md`](./dev-caches-npm.md) |',
        '| `temp.old-files` | Safe | none | [`temp-old-files.md`](./temp-old-files.md) |',
        '',
        'The next ones come later.',
      ].join('\n');

      assert.deepEqual(parseCatalogue(`${LEVELS_TABLE}\n\n${readme}`), [
        { category: 'dev-caches.npm', danger: 'Low', file: 'dev-caches-npm.md' },
        { category: 'temp.old-files', danger: 'Safe', file: 'temp-old-files.md' },
      ]);
    });
  });
});

describe('buildBlock', () => {
  describe('positive cases', () => {
    it('folds the levels and the rows, Safe before Low, under a summary with the count', () => {
      const categories: readonly Category[] = [
        {
          category: 'dev-caches.npm',
          danger: 'Low',
          example: '`~/.npm/_cacache`',
          file: 'dev-caches-npm.md',
          title: 'npm',
        },
        {
          category: 'logs.app-logs',
          danger: 'Safe',
          example: '`~/Library/Logs`',
          file: 'logs-app-logs.md',
          title: 'Logs',
        },
      ];

      assert.equal(
        buildBlock(categories, LEVELS),
        [
          '<details>',
          '<summary><b>All 2 kinds of files, with their danger level</b></summary>',
          '',
          '- **Safe** — Rebuilt automatically.',
          '- **Low** — Rebuilt on demand.',
          '- **Medium** — Restored by hand.',
          '- **High** — May hold the only copy.',
          '',
          '| What | Where, for example | Danger |',
          '| --- | --- | --- |',
          '| [Logs](docs/categories/logs-app-logs.md) | `~/Library/Logs` | Safe |',
          '| [npm](docs/categories/dev-caches-npm.md) | `~/.npm/_cacache` | Low |',
          '',
          '</details>',
        ].join('\n'),
      );
    });

    it('keeps the catalogue order within one level', () => {
      const category = (title: string, danger: Category['danger']): Category => ({
        category: title,
        danger,
        example: 'x',
        file: `${title}.md`,
        title,
      });

      const rows = buildBlock([category('b', 'High'), category('a', 'Low'), category('c', 'Low')], LEVELS)
        .split('\n')
        .filter((line) => line.startsWith('| ['));

      assert.deepEqual(
        rows.map((row) => row.slice(3, 4)),
        ['a', 'c', 'b'],
      );
    });
  });
});

describe('replaceBlock', () => {
  const kindOf = (readme: string): string | undefined => {
    const result = replaceBlock(readme, 'block');

    return 'problem' in result ? result.problem.kind : undefined;
  };

  describe('negative cases', () => {
    it('refuses a README without markers', () => {
      assert.equal(kindOf('# Fresher\n'), 'markers');
    });

    it('refuses a README with two start markers', () => {
      assert.equal(kindOf([START_MARKER, START_MARKER, END_MARKER].join('\n')), 'markers');
    });

    it('refuses a README whose end marker comes before the start', () => {
      assert.equal(kindOf([END_MARKER, START_MARKER].join('\n')), 'markers');
    });
  });

  describe('positive cases', () => {
    it('changes only the text between the markers', () => {
      const before = '# Fresher\n\nIntro  with  spaces.\n';
      const after = '\n## Next\n\nTrailing.\n';
      const readme = `${before}${START_MARKER}\nold block\n${END_MARKER}${after}`;

      assert.deepEqual(replaceBlock(readme, 'new block'), {
        text: `${before}${START_MARKER}\n\nnew block\n\n${END_MARKER}${after}`,
      });
    });
  });
});
