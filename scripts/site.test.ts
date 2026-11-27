import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, describe, it } from 'node:test';

import { build } from './site.ts';

// GitHub's own field names.
const URL_KEY = 'browser_download_url';
const TAG_KEY = 'tag_name';

const IMAGE = 'Fresher_0.2.0_universal.dmg';
const RELEASE = {
  assets: [{ name: IMAGE, [URL_KEY]: `https://github.com/AlexSergey/fresher/releases/download/v0.2.0/${IMAGE}` }],
  [TAG_KEY]: 'v0.2.0',
};

const CATALOGUE = [
  '# Categories',
  '',
  '## Danger levels',
  '',
  '| Level | Shown as | Meaning | Examples |',
  '| --- | --- | --- | --- |',
  '| **Safe** | Safe to remove | a | b |',
  '| **Low** | Usually safe | a | b |',
  '| **Medium** | Look before removing | a | b |',
  '| **High** | Only if you are sure | a | b |',
  '',
  '## Catalogue',
  '',
  '| Category | Danger level | Rule | File |',
  '| --- | --- | --- | --- |',
  '| `xcode.archives` | High | none | [`xcode-archives.md`](./xcode-archives.md) |',
  '',
].join('\n');

const ARCHIVES = ['# Xcode archives', '', '**Danger level:** High · **Rule:** none', '', '**Example:** `~/x`', ''].join(
  '\n',
);

const TEMPLATE = '<!-- site:counter --><h2>{{kind-count}}</h2><!-- site:danger-bar --><!-- site:kinds -->{{version}}';

const temporary = mkdtempSync(path.join(tmpdir(), 'fresher-site-'));
after(() => {
  rmSync(temporary, { force: true, recursive: true });
});

let count = 0;

// A small repository: site/, the catalogue with one category, and the copied images as text stand-ins.
const fixtureRoot = ({ code = 'getfresher', template = TEMPLATE } = {}): string => {
  count += 1;
  const root = path.join(temporary, `root-${String(count)}`);
  const files: Readonly<Record<string, string>> = {
    'assets/demo/cleanup.gif': 'gif',
    'assets/logo.png': 'logo',
    'assets/social-preview.png': 'preview',
    'docs/categories/README.md': CATALOGUE,
    'docs/categories/xcode-archives.md': ARCHIVES,
    'site/cleanup-still.png': 'still',
    'site/config.json': JSON.stringify({ goatcounter: code }),
    'site/index.html': template,
    'site/styles.css': 'body {}',
  };
  for (const [file, text] of Object.entries(files)) {
    mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    writeFileSync(path.join(root, file), text);
  }

  return root;
};

describe('build', () => {
  describe('negative cases', () => {
    it('refuses an output folder that is the repository, site/ or holds them', () => {
      const root = fixtureRoot();

      for (const out of [root, path.join(root, 'site'), path.join(root, 'site', 'out'), path.dirname(root)]) {
        assert.equal(build({ counterOverride: undefined, out, release: RELEASE, root }).length, 1, out);
      }
      assert.ok(existsSync(path.join(root, 'site', 'index.html')));
    });

    it('refuses an output folder outside dist/ and the temporary folders, or the temporary folder itself', () => {
      const root = fixtureRoot();

      for (const out of [path.join(path.parse(root).root, 'fresher-site-out'), tmpdir()]) {
        assert.equal(build({ counterOverride: undefined, out, release: RELEASE, root }).length, 1, out);
      }
    });

    it('writes nothing when the release has no universal disk image', () => {
      const root = fixtureRoot();
      const out = path.join(root, 'dist', 'site');

      const problems = build({ counterOverride: undefined, out, release: { ...RELEASE, assets: [] }, root });

      assert.equal(problems.length, 1);
      assert.match(problems[0] ?? '', /no asset/u);
      assert.ok(!existsSync(out));
    });

    it('writes nothing when the counter code is not a site code', () => {
      const root = fixtureRoot({ code: 'not a code' });

      assert.equal(
        build({ counterOverride: undefined, out: path.join(root, 'dist'), release: RELEASE, root }).length,
        1,
      );
    });

    it('writes nothing when the page has a slot missing', () => {
      const root = fixtureRoot({ template: '{{version}}' });

      assert.equal(
        build({ counterOverride: undefined, out: path.join(root, 'dist'), release: RELEASE, root }).length,
        1,
      );
    });
  });

  describe('positive cases', () => {
    it('writes the page, /download, the style sheet and the images', () => {
      const root = fixtureRoot();
      const out = path.join(root, 'dist', 'site');

      assert.deepEqual(build({ counterOverride: undefined, out, release: RELEASE, root }), []);

      const index = readFileSync(path.join(out, 'index.html'), 'utf8');
      assert.match(index, /<h2>1<\/h2>/u);
      assert.match(index, /docs\/categories\/xcode-archives\.md">Xcode archives<\/a><\/li>/u);
      assert.match(index, /getfresher\.goatcounter\.com/u);
      assert.match(index, /0\.2\.0$/u);
      assert.match(readFileSync(path.join(out, 'download', 'index.html'), 'utf8'), new RegExp(IMAGE, 'u'));
      assert.equal(readFileSync(path.join(out, 'cleanup.gif'), 'utf8'), 'gif');
      assert.equal(readFileSync(path.join(out, 'styles.css'), 'utf8'), 'body {}');
    });

    it('takes the counter code from the override over the config', () => {
      const root = fixtureRoot();
      const out = path.join(root, 'dist', 'site');

      assert.deepEqual(build({ counterOverride: 'other-site', out, release: RELEASE, root }), []);
      assert.match(readFileSync(path.join(out, 'index.html'), 'utf8'), /other-site\.goatcounter\.com/u);
    });

    it('replaces an earlier build', () => {
      const root = fixtureRoot();
      const out = path.join(root, 'dist', 'site');
      mkdirSync(out, { recursive: true });
      writeFileSync(path.join(out, 'stale.html'), 'old');

      assert.deepEqual(build({ counterOverride: undefined, out, release: RELEASE, root }), []);
      assert.ok(!existsSync(path.join(out, 'stale.html')));
    });
  });
});
