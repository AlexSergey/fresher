import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import type { Problem } from './readme-catalogue.ts';

import { checkReferences, findReferences, headingSlugs, slugOf } from './readme-links.ts';

const GOAL_HEADING = '### 8. Transparent: it does only what the user asks';
const GOAL_SLUG = '8-transparent-it-does-only-what-the-user-asks';

const check = (
  markdown: string,
  options: {
    readonly existing?: readonly string[];
    readonly file?: string;
    readonly slugs?: Record<string, string[]>;
  } = {},
): readonly Problem[] => {
  const { existing = [], file = 'README.md', slugs = {} } = options;

  return checkReferences({
    exists: (repoPath) => existing.includes(repoPath),
    file,
    markdown,
    slugsOf: (repoPath) => {
      const found = slugs[repoPath];

      return found === undefined ? null : new Set(found);
    },
  });
};

const kinds = (...args: Parameters<typeof check>): readonly string[] => check(...args).map((problem) => problem.kind);

describe('checkReferences', () => {
  describe('negative cases', () => {
    it('reports a link to a file that does not exist', () => {
      assert.deepEqual(kinds('[x](docs/none.md)'), ['dead-link']);
    });

    it('reports a link to an anchor the file does not have', () => {
      assert.deepEqual(kinds('# Title\n\n[x](#nowhere)'), ['dead-anchor']);
    });

    it('reports an anchor into a Markdown file whose headings lack it', () => {
      const options = { existing: ['docs/goal.md'], slugs: { 'docs/goal.md': ['goal'] } };

      assert.deepEqual(kinds('[x](docs/goal.md#nowhere)', options), ['dead-anchor']);
    });

    it('reports an image outside assets', () => {
      assert.deepEqual(kinds('![x](docs/pic.png)', { existing: ['docs/pic.png'] }), ['image-outside-assets']);
    });

    it('reports an image under assets that does not exist', () => {
      assert.deepEqual(kinds('![x](assets/none.png)'), ['missing-image']);
    });

    it('reports a remote image that is not a badge', () => {
      assert.deepEqual(kinds('<img src="https://example.com/a.png">'), ['remote-image']);
    });

    it('reports a claim of something not built', () => {
      const problems = check('# Fresher\n\nWatch mode tells you when it grows.');

      assert.deepEqual(problems, [
        { kind: 'unbuilt-claim', message: 'watch mode is not built (plan 010)', where: 'README.md:3' },
      ]);
    });

    it('reports a reference-style link definition to a file that does not exist', () => {
      assert.deepEqual(kinds('[x][ref]\n\n[ref]: docs/none.md'), ['dead-link']);
    });

    it('reports a link that climbs out of the repository', () => {
      assert.deepEqual(kinds('[x](../outside.md)', { existing: ['../outside.md', 'outside.md'] }), ['dead-link']);
    });

    it('reports the line of each problem', () => {
      const problems = check('one\n\ntwo [x](docs/none.md)');

      assert.deepEqual(
        problems.map((problem) => problem.where),
        ['README.md:3'],
      );
    });

    it('sorts problems of different kinds by their line number', () => {
      const markdown = ['watch mode', ...Array.from({ length: 9 }, () => ''), '[x](none.md)', 'watch mode'].join('\n');

      assert.deepEqual(
        check(markdown).map((problem) => problem.where),
        ['README.md:1', 'README.md:11', 'README.md:12'],
      );
    });

    it('reports a single-quoted HTML image outside assets', () => {
      assert.deepEqual(kinds("<img src='docs/pic.png'>", { existing: ['docs/pic.png'] }), ['image-outside-assets']);
    });

    it('reports a single-quoted HTML link to a file that does not exist', () => {
      assert.deepEqual(kinds("<a href='docs/none.md'>x</a>"), ['dead-link']);
    });
  });

  describe('positive cases', () => {
    it('ignores a link inside a fenced code block', () => {
      assert.deepEqual(kinds('```\n[x](docs/none.md)\nwatch mode\n```'), []);
    });

    it('accepts an anchor into a Markdown file that has the heading', () => {
      const options = { existing: ['docs/goal.md'], slugs: { 'docs/goal.md': [GOAL_SLUG] } };

      assert.deepEqual(kinds(`[x](docs/goal.md#${GOAL_SLUG})`, options), []);
    });

    it('accepts a link to the top of the page', () => {
      assert.deepEqual(kinds('# Title\n\n[top](#) <a href="#">top</a>'), []);
    });

    it('accepts an anchor to a heading of its own file', () => {
      assert.deepEqual(kinds('## Install\n\n[x](#install)'), []);
    });

    it('accepts an image under assets that exists', () => {
      assert.deepEqual(kinds('<img src="assets/logo.png">', { existing: ['assets/logo.png'] }), []);
    });

    it('accepts a reference-style link definition to a file that exists', () => {
      assert.deepEqual(kinds('[x][ref]\n\n[ref]: docs/goal.md "Goal"', { existing: ['docs/goal.md'] }), []);
    });

    it('accepts a badge from shields.io', () => {
      assert.deepEqual(kinds('![licence](https://img.shields.io/badge/licence-MIT-blue)'), []);
    });

    it('does not check https and mailto links', () => {
      assert.deepEqual(kinds('[a](https://example.com/none) [b](mailto:someone@example.com)'), []);
    });

    it('resolves a link from .github against the repository root', () => {
      const options = { existing: ['docs/rules.md'], file: '.github/SECURITY.md' };

      assert.deepEqual(kinds('[x](../docs/rules.md)', options), []);
      assert.deepEqual(kinds('[x](rules.md)', options), ['dead-link']);
    });

    it('ignores the query of a link and a title after its target', () => {
      assert.deepEqual(kinds('[x](docs/a.md?plain=1 "A title")', { existing: ['docs/a.md'] }), []);
    });
  });
});

describe('findReferences', () => {
  describe('positive cases', () => {
    it('finds both the image and the link of an image wrapped in a link', () => {
      assert.deepEqual(findReferences('[![alt](assets/a.png)](docs/a.md)'), [
        { kind: 'image', line: 1, target: 'assets/a.png' },
        { kind: 'link', line: 1, target: 'docs/a.md' },
      ]);
    });

    it('finds HTML anchors and images with their line numbers', () => {
      assert.deepEqual(findReferences('text\n<a href="docs/a.md">a</a> <img src="assets/b.png">'), [
        { kind: 'image', line: 2, target: 'assets/b.png' },
        { kind: 'link', line: 2, target: 'docs/a.md' },
      ]);
    });

    it('finds single-quoted HTML attributes', () => {
      assert.deepEqual(findReferences("<a href='docs/a.md'><img src='assets/b.png'></a>"), [
        { kind: 'image', line: 1, target: 'assets/b.png' },
        { kind: 'link', line: 1, target: 'docs/a.md' },
      ]);
    });
  });
});

describe('slugOf', () => {
  describe('positive cases', () => {
    it('slugs a heading with a number, a colon and spaces', () => {
      assert.equal(slugOf(GOAL_HEADING), GOAL_SLUG);
    });

    it('drops inline code backticks', () => {
      assert.equal(slugOf('## `npm` caches'), 'npm-caches');
    });

    it('keeps underscores the way GitHub does', () => {
      assert.equal(slugOf('## `node_modules` folders'), 'node_modules-folders');
    });
  });
});

describe('headingSlugs', () => {
  describe('positive cases', () => {
    it('numbers a repeated heading and skips headings in code blocks', () => {
      const slugs = headingSlugs('# A\n\n## Same\n\n## Same\n\n```\n# Hidden\n```\n');

      assert.deepEqual([...slugs], ['a', 'same', 'same-1']);
    });
  });
});
