import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, describe, it } from 'node:test';
import * as prettier from 'prettier';

import { END_MARKER, START_MARKER } from './readme-catalogue.ts';
import { formatMarkdown, run } from './readme.ts';

const PRETTIER_CONFIG = JSON.stringify({ endOfLine: 'lf', printWidth: 120, singleQuote: true, trailingComma: 'all' });

const CATALOGUE = `# Categories

## Danger levels

| Level | Meaning | Examples |
| --- | --- | --- |
| **Safe** | Rebuilt automatically. | logs |
| **Low** | Rebuilt on demand. | caches |
| **Medium** | Restored by hand. | settings |
| **High** | May hold the only copy. | backups |

## Catalogue

| Category | Danger level | Rule | File |
| --- | --- | --- | --- |
| \`dev-caches.npm\` | Low | none | [\`dev-caches-npm.md\`](./dev-caches-npm.md) |
| \`logs.app-logs\` | Safe | none | [\`logs-app-logs.md\`](./logs-app-logs.md) |
`;

const NPM = `# npm's download cache

**Danger level:** Low · **Rule:** none · **Since:** 0.1.0

**Example:** \`~/.npm/_cacache\`
`;

const LOGS = `# Logs

**Danger level:** Safe · **Rule:** none · **Since:** 0.1.0

**Example:** \`~/Library/Logs\`
`;

const README = `# Fresher

## What it finds

${START_MARKER}
${END_MARKER}

## Licence
`;

const folders: string[] = [];

const makeRepository = (files: Readonly<Record<string, string>> = {}): string => {
  const root = mkdtempSync(path.join(tmpdir(), 'fresher-readme-'));
  folders.push(root);
  const all: Record<string, string> = {
    '.prettierrc': PRETTIER_CONFIG,
    'docs/categories/dev-caches-npm.md': NPM,
    'docs/categories/logs-app-logs.md': LOGS,
    'docs/categories/README.md': CATALOGUE,
    'README.md': README,
    ...files,
  };
  for (const [name, text] of Object.entries(all)) {
    mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
    writeFileSync(path.join(root, name), text);
  }

  return root;
};

const readReadme = (root: string): string => readFileSync(path.join(root, 'README.md'), 'utf8');

after(() => {
  for (const folder of folders) {
    rmSync(folder, { force: true, recursive: true });
  }
});

describe('run', () => {
  describe('negative cases', () => {
    it('reports a dead link and no stale block in a freshly generated README', async () => {
      const root = makeRepository();
      await run('generate', root);
      writeFileSync(path.join(root, 'README.md'), `${readReadme(root)}\n[gone](docs/missing.md)\n`);

      const problems = await run('check', root);

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['dead-link'],
      );
    });

    it('reports a stale block when a table cell was edited', async () => {
      const root = makeRepository();
      await run('generate', root);
      writeFileSync(path.join(root, 'README.md'), readReadme(root).replace('`~/Library/Logs`', '`~/Logs`'));

      const problems = await run('check', root);

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['stale-block'],
      );
      assert.match(problems[0]?.message ?? '', /differs from what `npm run readme:generate` writes/u);
    });

    it('reports a link whose case differs from the file on disk', async () => {
      const root = makeRepository({ 'docs/Goal.md': '# Goal\n' });
      writeFileSync(path.join(root, 'README.md'), `${README}\n[goal](docs/goal.md)\n`);

      const problems = await run('generate', root);

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['dead-link'],
      );
    });

    it('reports a licence link whose case differs from the file on disk', async () => {
      const root = makeRepository({ LICENSE: 'text\n' });
      writeFileSync(path.join(root, 'README.md'), `${README}\n[licence](License)\n`);

      const problems = await run('generate', root);

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['dead-link'],
      );
    });

    it('reports an image whose case differs from the file on disk', async () => {
      const root = makeRepository({ 'assets/Logo.png': 'png' });
      writeFileSync(path.join(root, 'README.md'), `${README}\n![logo](assets/logo.png)\n`);

      const problems = await run('generate', root);

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['missing-image'],
      );
    });

    it('writes nothing when a category file has no example', async () => {
      const root = makeRepository({ 'docs/categories/logs-app-logs.md': LOGS.replace('**Example:**', 'Example:') });

      const problems = await run('generate', root);

      assert.deepEqual(
        problems.map((problem) => problem.kind),
        ['no-example'],
      );
      assert.equal(readReadme(root), README);
    });
  });

  describe('positive cases', () => {
    it('finds no problem in a README it has just generated', async () => {
      const root = makeRepository();

      assert.deepEqual(await run('generate', root), []);
      assert.deepEqual(await run('check', root), []);
      assert.match(readReadme(root), /\[Logs\]\(docs\/categories\/logs-app-logs\.md\)/u);
    });

    it('writes the same bytes when run twice', async () => {
      const root = makeRepository();
      await run('generate', root);
      const first = readReadme(root);

      await run('generate', root);

      assert.equal(readReadme(root), first);
    });
  });
});

describe('formatMarkdown', () => {
  describe('positive cases', () => {
    it('aligns an unaligned table the way Prettier does', async () => {
      const root = makeRepository();
      const filePath = path.join(root, 'README.md');
      const text = '| What | Danger |\n| --- | --- |\n| npm cache | Low |\n| Logs | Safe |\n';
      const config = await prettier.resolveConfig(filePath);

      const formatted = await formatMarkdown(text, filePath);

      assert.equal(formatted, await prettier.format(text, { ...config, filepath: filePath }));
      assert.match(formatted, /^\| What {6}\| Danger \|$/mu);
    });
  });
});
