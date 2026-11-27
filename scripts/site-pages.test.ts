import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import type { Category, Levels } from './readme-catalogue.ts';

import {
  buildDangerBar,
  buildKinds,
  counterTag,
  escapeHtml,
  parseCounterCode,
  parseRelease,
  renderDownload,
  renderIndex,
} from './site-pages.ts';

// GitHub's own field names.
const URL_KEY = 'browser_download_url';
const TAG_KEY = 'tag_name';

const IMAGE = 'Fresher_0.2.0_universal.dmg';
const IMAGE_URL = `https://github.com/AlexSergey/fresher/releases/download/v0.2.0/${IMAGE}`;

const releaseJson = (overrides: Readonly<Record<string, unknown>> = {}): Readonly<Record<string, unknown>> => ({
  assets: [
    {
      name: 'SHA256SUMS',
      [URL_KEY]: 'https://github.com/AlexSergey/fresher/releases/download/v0.2.0/SHA256SUMS',
    },
    { name: IMAGE, [URL_KEY]: IMAGE_URL },
  ],
  draft: false,
  prerelease: false,
  [TAG_KEY]: 'v0.2.0',
  ...overrides,
});

const problemOf = (json: unknown): string | undefined => {
  const parsed = parseRelease(json);

  return 'problem' in parsed ? parsed.problem.kind : undefined;
};

const LEVELS: Levels = new Map([
  ['High', 'Only if you are sure'],
  ['Low', 'Usually safe'],
  ['Medium', 'Look before removing'],
  ['Safe', 'Safe to remove'],
]);

const category = (title: string, danger: Category['danger']): Category => ({
  category: `test.${title}`,
  danger,
  example: '`~/x`',
  file: `test-${title}.md`,
  title,
});

const CATEGORIES: readonly Category[] = [
  category("npm's download cache", 'Low'),
  category('Xcode archives', 'High'),
  category('Xcode build data', 'Safe'),
  category('Old app logs', 'Safe'),
];

const TEMPLATE = [
  '<head><!-- site:counter --></head>',
  '<h2>{{kind-count}} kinds</h2>',
  '<!-- site:danger-bar -->',
  '<div><!-- site:kinds --></div>',
  '<span>Version {{version}}</span>',
].join('\n');

const VALUES = { categories: CATEGORIES, counterCode: 'getfresher', levels: LEVELS, version: '0.2.0' } as const;

const RELEASE = { downloadUrl: IMAGE_URL, version: '0.2.0' } as const;

describe('parseRelease', () => {
  describe('negative cases', () => {
    it('refuses what is not a release object', () => {
      assert.equal(problemOf(null), 'bad-release');
      assert.equal(problemOf([]), 'bad-release');
      assert.equal(problemOf({ [TAG_KEY]: 'v0.2.0' }), 'bad-release');
    });

    it('refuses a draft or a pre-release', () => {
      assert.equal(problemOf(releaseJson({ draft: true })), 'not-published');
      assert.equal(problemOf(releaseJson({ prerelease: true })), 'not-published');
    });

    it('refuses a release without the universal disk image', () => {
      assert.equal(problemOf(releaseJson({ assets: [] })), 'no-asset');
      assert.equal(
        problemOf(releaseJson({ assets: [{ name: 'Fresher_0.2.0_aarch64.dmg', [URL_KEY]: IMAGE_URL }] })),
        'no-asset',
      );
    });

    it('refuses two universal disk images', () => {
      const second = { name: 'Fresher_0.1.0_universal.dmg', [URL_KEY]: IMAGE_URL };

      assert.equal(problemOf(releaseJson({ assets: [{ name: IMAGE, [URL_KEY]: IMAGE_URL }, second] })), 'two-assets');
    });

    it("refuses an image whose version is not the release's tag", () => {
      assert.equal(problemOf(releaseJson({ [TAG_KEY]: 'v0.1.0' })), 'tag-mismatch');
    });

    it("refuses an image whose address is not the repository's release download", () => {
      for (const url of [
        `https://example.com/${IMAGE}`,
        `https://github.com/someone/fresher/releases/download/v0.2.0/${IMAGE}`,
        `javascript:alert(1)//${IMAGE}`,
      ]) {
        assert.equal(problemOf(releaseJson({ assets: [{ name: IMAGE, [URL_KEY]: url }] })), 'asset-url');
      }
    });
  });

  describe('positive cases', () => {
    it('takes the version and the address of the universal disk image', () => {
      assert.deepEqual(parseRelease(releaseJson()), { release: RELEASE });
    });
  });
});

describe('parseCounterCode', () => {
  describe('negative cases', () => {
    it('refuses a code that is not a GoatCounter site code', () => {
      for (const code of ['Get Fresher', 'a.b', '"><script>', '-fresher']) {
        assert.ok('problem' in parseCounterCode(code), code);
      }
    });

    it('leaves the counter out while the placeholder or nothing is set', () => {
      assert.deepEqual(parseCounterCode('GOATCOUNTER_CODE'), { code: undefined });
      assert.deepEqual(parseCounterCode(''), { code: undefined });
      assert.equal(counterTag(undefined), '');
    });
  });

  describe('positive cases', () => {
    it("takes a site code and loads GoatCounter's script for it", () => {
      assert.deepEqual(parseCounterCode('getfresher'), { code: 'getfresher' });
      assert.equal(
        counterTag('getfresher'),
        '<script data-goatcounter="https://getfresher.goatcounter.com/count" async src="https://gc.zgo.at/count.js"></script>',
      );
    });
  });
});

describe('buildKinds and buildDangerBar', () => {
  describe('negative cases', () => {
    it('shows no column and no bar part for a level without kinds', () => {
      assert.doesNotMatch(buildKinds(CATEGORIES, LEVELS), /Look before removing/u);
      assert.doesNotMatch(buildDangerBar(CATEGORIES, LEVELS), /class="medium"/u);
    });

    it('escapes the titles', () => {
      assert.match(buildKinds([category('<b>&', 'Safe')], LEVELS), />&lt;b&gt;&amp;<\/a><\/li>/u);
      assert.equal(escapeHtml(`"'`), '&quot;&#39;');
    });
  });

  describe('positive cases', () => {
    it('lists the kinds by danger level, safest first, in catalogue order inside a level', () => {
      const kinds = buildKinds(CATEGORIES, LEVELS);
      const order = ['Safe to remove', 'Xcode build data', 'Old app logs', 'Usually safe', 'npm&#39;s', 'Only if'].map(
        (text) => kinds.indexOf(text),
      );

      assert.ok(order.every((index) => index !== -1));
      assert.deepEqual(
        order,
        order.toSorted((a, b) => a - b),
      );
      assert.match(kinds, /<span class="level-count">2<\/span>/u);
      assert.ok(
        kinds.includes(
          '<a href="https://github.com/AlexSergey/fresher/blob/main/docs/categories/test-Xcode archives.md">',
        ),
      );
    });

    it('draws a bar part per level, as wide as its count, with a label', () => {
      const bar = buildDangerBar(CATEGORIES, LEVELS);

      assert.match(
        bar,
        /aria-label="The 4 kinds by danger level: 2 safe to remove, 1 usually safe, 1 only if you are sure"/u,
      );
      assert.match(bar, /<span class="safe" style="flex: 2"><\/span>/u);
    });
  });
});

describe('renderIndex', () => {
  describe('negative cases', () => {
    it('keeps a replacement pattern in a title as text', () => {
      const result = renderIndex(TEMPLATE, { ...VALUES, categories: [category('$& cache', 'Low')] });

      assert.ok('page' in result);
      assert.match(result.page, />\$&amp; cache<\/a><\/li>/u);
      assert.doesNotMatch(result.page, /<!-- site:kinds --> cache/u);
    });

    it('refuses a template without a slot or with one twice', () => {
      const missing = renderIndex(TEMPLATE.replace('<!-- site:kinds -->', ''), VALUES);
      const twice = renderIndex(`${TEMPLATE}{{version}}`, VALUES);

      assert.ok('problem' in missing && missing.problem.kind === 'slot');
      assert.ok('problem' in twice && twice.problem.kind === 'slot');
    });

    it('refuses a template with a value the build does not know', () => {
      const result = renderIndex(`${TEMPLATE}{{total}}`, VALUES);

      assert.ok('problem' in result && result.problem.kind === 'slot');
    });

    it('loads no counter without a code', () => {
      const result = renderIndex(TEMPLATE, { ...VALUES, counterCode: undefined });

      assert.ok('page' in result);
      assert.doesNotMatch(result.page, /goatcounter/u);
    });
  });

  describe('positive cases', () => {
    it('fills the counter, the count, the bar, the kinds and the version', () => {
      const result = renderIndex(TEMPLATE, VALUES);

      assert.ok('page' in result);
      assert.match(result.page, /getfresher\.goatcounter\.com/u);
      assert.match(result.page, /<h2>4 kinds<\/h2>/u);
      assert.match(result.page, /class="danger-bar"/u);
      assert.match(result.page, />Xcode archives<\/a><\/li>/u);
      assert.match(result.page, /Version 0\.2\.0/u);
    });
  });
});

describe('renderDownload', () => {
  describe('negative cases', () => {
    it('without a counter, leaves at once and loads nothing else', () => {
      const page = renderDownload(RELEASE, undefined);

      assert.doesNotMatch(page, /goatcounter/u);
      assert.ok(page.includes(`<script>location.replace(${JSON.stringify(IMAGE_URL)});</script>`));
    });
  });

  describe('positive cases', () => {
    it('sends the browser on by script, by meta refresh and by a plain link', () => {
      const page = renderDownload(RELEASE, 'getfresher');

      assert.ok(page.includes(`var target = ${JSON.stringify(IMAGE_URL)};`));
      assert.ok(page.includes('location.replace(target)'));
      assert.ok(page.includes(`<noscript><meta http-equiv="refresh" content="0; url=${IMAGE_URL}" /></noscript>`));
      assert.equal(page.split('http-equiv="refresh"').length, 2);
      assert.ok(page.includes(`<a class="btn-primary" href="${IMAGE_URL}">Download ${IMAGE}</a>`));
      assert.match(page, /<title>Downloading Fresher 0\.2\.0<\/title>/u);
    });

    it('counts its own view, with a pixel when scripts are off', () => {
      const page = renderDownload(RELEASE, 'getfresher');

      assert.ok(page.includes('window.goatcounter = { no_onload: true };'));
      assert.ok(page.includes("window.goatcounter.count({ path: '/download' });"));
      assert.ok(page.includes('https://getfresher.goatcounter.com/count?p=%2Fdownload'));
    });
  });
});
