import { type Category, type Danger, DANGERS, type Levels } from './readme-catalogue.ts';

// The pages of getfresher.app (plan 046): the kinds of files by danger level from docs/categories, the counter's tag,
// the version, and /download from the latest release. Pure: no file system, no network.

export const REPOSITORY = 'AlexSergey/fresher';

// Each kind links to its page of the catalogue.
const CATEGORIES_URL = `https://github.com/${REPOSITORY}/blob/main/docs/categories/`;

export const COUNTER_PLACEHOLDER = 'GOATCOUNTER_CODE';

export type Release = {
  readonly downloadUrl: string;
  readonly version: string;
};

export type SiteProblem = {
  readonly kind:
    | 'asset-url'
    | 'bad-counter-code'
    | 'bad-release'
    | 'no-asset'
    | 'not-published'
    | 'slot'
    | 'tag-mismatch'
    | 'two-assets';
  readonly message: string;
};

type DangerLook = { readonly className: string; readonly note: string };

// The site's short note under each level's name; the name itself is the catalogue's "Shown as".
const LOOKS: ReadonlyMap<Danger, DangerLook> = new Map([
  ['High', { className: 'high', note: 'Cannot be made again; its own tick each time.' }],
  ['Low', { className: 'low', note: 'Downloads or builds again, at the cost of time.' }],
  ['Medium', { className: 'medium', note: 'Settings or work you may still want.' }],
  ['Safe', { className: 'safe', note: 'Comes back by itself when needed.' }],
]);

const ASSET_NAME = /^Fresher_(\d+\.\d+\.\d+)_universal\.dmg$/u;
const DOWNLOAD_PREFIX = `https://github.com/${REPOSITORY}/releases/download/`;
const COUNTER_CODE = /^[a-z\d][a-z\d-]{0,62}$/u;

export const escapeHtml = (text: string): string =>
  text
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const stringField = (record: Readonly<Record<string, unknown>>, key: string): string | undefined => {
  const value = record[key];

  return typeof value === 'string' ? value : undefined;
};

// GitHub's `repos/<owner>/<repo>/releases/latest`: the one universal disk image of a published release.
export const parseRelease = (json: unknown): { readonly problem: SiteProblem } | { readonly release: Release } => {
  const problem = (kind: SiteProblem['kind'], message: string): { readonly problem: SiteProblem } => ({
    problem: { kind, message },
  });

  if (!isRecord(json) || !Array.isArray(json.assets)) {
    return problem('bad-release', 'the release is not an object with a list of assets');
  }
  if (json.draft === true || json.prerelease === true) {
    return problem('not-published', 'the release is a draft or a pre-release');
  }

  const images = json.assets.filter(isRecord).flatMap((asset) => {
    const name = stringField(asset, 'name') ?? '';
    const version = ASSET_NAME.exec(name)?.[1];

    return version === undefined ? [] : [{ name, url: stringField(asset, 'browser_download_url') ?? '', version }];
  });

  const [image] = images;
  if (image === undefined) {
    return problem('no-asset', 'the release has no asset named Fresher_<version>_universal.dmg');
  }
  if (images.length > 1) {
    return problem('two-assets', 'the release has more than one Fresher_<version>_universal.dmg');
  }
  if (stringField(json, 'tag_name') !== `v${image.version}`) {
    return problem('tag-mismatch', `the image ${image.name} is not on the tag v${image.version}`);
  }
  if (image.url !== `${DOWNLOAD_PREFIX}v${image.version}/${image.name}`) {
    return problem('asset-url', `the image's address is not ${DOWNLOAD_PREFIX}v${image.version}/${image.name}`);
  }

  return { release: { downloadUrl: image.url, version: image.version } };
};

// The counter's site code: undefined while the placeholder is left, so no page loads a counter that does not exist.
export const parseCounterCode = (
  code: string,
): { readonly code: string | undefined } | { readonly problem: SiteProblem } => {
  if (code === COUNTER_PLACEHOLDER || code === '') {
    return { code: undefined };
  }
  if (!COUNTER_CODE.test(code)) {
    return {
      problem: {
        kind: 'bad-counter-code',
        message: `the counter code "${code}" is not lowercase letters, digits and -`,
      },
    };
  }

  return { code };
};

const counterEndpoint = (code: string): string => `https://${code}.goatcounter.com/count`;

// GoatCounter's script counts the view and binds every `data-goatcounter-click` link; it sets no cookie.
export const counterTag = (code: string | undefined): string =>
  code === undefined
    ? ''
    : `<script data-goatcounter="${counterEndpoint(code)}" async src="https://gc.zgo.at/count.js"></script>`;

const sortedByDanger = (categories: readonly Category[]): readonly Category[] =>
  categories.toSorted((a, b) => DANGERS.indexOf(a.danger) - DANGERS.indexOf(b.danger));

const countOf = (categories: readonly Category[], danger: Danger): number =>
  categories.filter((category) => category.danger === danger).length;

export const buildDangerBar = (categories: readonly Category[], levels: Levels): string => {
  const parts = DANGERS.map((danger) => ({ count: countOf(categories, danger), danger })).filter(
    (part) => part.count > 0,
  );
  const label = parts
    .map((part) => `${String(part.count)} ${(levels.get(part.danger) ?? part.danger).toLowerCase()}`)
    .join(', ');

  return [
    `<div class="danger-bar" role="img" aria-label="${escapeHtml(`The ${String(categories.length)} kinds by danger level: ${label}`)}">`,
    ...parts.map(
      (part) =>
        `  <span class="${LOOKS.get(part.danger)?.className ?? ''}" style="flex: ${String(part.count)}"></span>`,
    ),
    '</div>',
  ].join('\n');
};

export const buildKinds = (categories: readonly Category[], levels: Levels): string =>
  DANGERS.flatMap((danger) => {
    const kinds = sortedByDanger(categories).filter((category) => category.danger === danger);
    if (kinds.length === 0) {
      return [];
    }
    const look = LOOKS.get(danger);

    return [
      [
        '<div class="level">',
        '  <div class="stack">',
        '    <div class="level-head">',
        `      <span class="dot ${look?.className ?? ''}" aria-hidden="true"></span>`,
        `      <h3>${escapeHtml(levels.get(danger) ?? danger)}</h3>`,
        `      <span class="level-count">${String(kinds.length)}</span>`,
        '    </div>',
        `    <p class="level-note">${escapeHtml(look?.note ?? '')}</p>`,
        '  </div>',
        '  <ul class="plain-list kinds">',
        ...kinds.map(
          (kind) => `    <li><a href="${escapeHtml(CATEGORIES_URL + kind.file)}">${escapeHtml(kind.title)}</a></li>`,
        ),
        '  </ul>',
        '</div>',
      ].join('\n'),
    ];
  }).join('\n');

export type IndexValues = {
  readonly categories: readonly Category[];
  readonly counterCode: string | undefined;
  readonly levels: Levels;
  readonly version: string;
};

const SLOTS = ['<!-- site:counter -->', '<!-- site:danger-bar -->', '<!-- site:kinds -->'] as const;
const VALUES = ['{{kind-count}}', '{{version}}'] as const;

// Every slot and value appears exactly once, and nothing like a value is left after the fill.
export const renderIndex = (
  template: string,
  values: IndexValues,
): { readonly page: string } | { readonly problem: SiteProblem } => {
  for (const mark of [...SLOTS, ...VALUES]) {
    const count = template.split(mark).length - 1;
    if (count !== 1) {
      return { problem: { kind: 'slot', message: `site/index.html holds ${mark} ${String(count)} times, not once` } };
    }
  }

  const page = template
    // Function replacers: a `$&` or `$'` in a title is text, not a replacement pattern.
    .replace('<!-- site:counter -->', () => counterTag(values.counterCode))
    .replace('<!-- site:danger-bar -->', () => buildDangerBar(values.categories, values.levels))
    .replace('<!-- site:kinds -->', () => buildKinds(values.categories, values.levels))
    .replace('{{kind-count}}', () => String(values.categories.length))
    .replace('{{version}}', () => escapeHtml(values.version));

  if (/\{\{[^}]*\}\}|<!-- site:/u.test(page)) {
    return { problem: { kind: 'slot', message: 'site/index.html holds a slot or value the build does not know' } };
  }

  return { page };
};

// How long the page waits for the counter before it leaves anyway, and after the count before it leaves.
export const COUNTER_WAIT_MS = 1500;
export const AFTER_COUNT_MS = 300;

const redirectScript = (downloadUrl: string, code: string | undefined): string => {
  const target = JSON.stringify(downloadUrl);
  if (code === undefined) {
    return `<script>location.replace(${target});</script>`;
  }

  // The view is counted by hand (`no_onload`), then the page leaves; a counter that fails or is blocked never holds
  // the download back.
  return [
    '<script>',
    '  (function () {',
    `    var target = ${target};`,
    '    var gone = false;',
    '    var go = function () {',
    '      if (!gone) {',
    '        gone = true;',
    '        location.replace(target);',
    '      }',
    '    };',
    '    window.goatcounter = { no_onload: true };',
    `    setTimeout(go, ${String(COUNTER_WAIT_MS)});`,
    "    var script = document.createElement('script');",
    `    script.dataset.goatcounter = ${JSON.stringify(counterEndpoint(code))};`,
    "    script.src = 'https://gc.zgo.at/count.js';",
    '    script.onload = function () {',
    '      try {',
    "        window.goatcounter.count({ path: '/download' });",
    '      } catch (error) {',
    '        go();',
    '      }',
    `      setTimeout(go, ${String(AFTER_COUNT_MS)});`,
    '    };',
    '    script.onerror = go;',
    '    document.head.appendChild(script);',
    '  })();',
    '</script>',
  ].join('\n');
};

// /download: counts its own view and sends the browser on to the disk image of the latest release. With scripts off,
// the meta refresh (inside <noscript>, so the download never starts twice) sends it on and the counter's pixel counts
// the view; the plain link is there if both fail.
export const renderDownload = (release: Release, counterCode: string | undefined): string => {
  const url = escapeHtml(release.downloadUrl);
  const version = escapeHtml(release.version);
  const pixel =
    counterCode === undefined
      ? ''
      : `\n    <noscript><img src="${counterEndpoint(counterCode)}?p=%2Fdownload" alt="" width="1" height="1" /></noscript>`;

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="robots" content="noindex" />
    <noscript><meta http-equiv="refresh" content="0; url=${url}" /></noscript>
    <title>Downloading Fresher ${version}</title>
    <link rel="icon" type="image/png" href="/logo.png" />
    <link rel="stylesheet" href="/styles.css" />
    ${redirectScript(release.downloadUrl, counterCode).replaceAll('\n', '\n    ')}
  </head>
  <body>
    <main class="wrap download-page">
      <a class="brand" href="/"><img src="/logo.png" alt="" width="22" height="22" />Fresher</a>
      <h1>Downloading Fresher ${version}</h1>
      <p class="lead">The disk image comes from the project's GitHub release. If the download does not start:</p>
      <a class="btn-primary" href="${url}">Download Fresher_${version}_universal.dmg</a>
      <p class="caption quiet">Open it and drag Fresher to Applications. <a href="/#install">The first launch</a></p>
    </main>${pixel}
  </body>
</html>
`;
};
